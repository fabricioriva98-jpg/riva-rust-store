//! Tradução C/C++ → Rust com IA (API do Claude) e laço do `rustc`: o modelo traduz, o `rustc` compila,
//! os erros voltam para o modelo e o ciclo se repete até compilar ou acabarem as tentativas.
//!
//! Usa a chave de API do próprio usuário (`settings.anthropic_api_key` ou `ANTHROPIC_API_KEY`); cada
//! tradução é cobrada pela Anthropic. HTTP direto (não há SDK oficial para Rust): `POST /v1/messages`.
//! O histórico é só acrescentado: a resposta do modelo volta inteira (com os blocos de raciocínio) nas
//! rodadas seguintes, como a API exige.

use crate::translator::TranslateMode;
use anyhow::{bail, Context, Result};
use serde_json::{json, Value};
use std::future::Future;

pub const MODEL: &str = "claude-opus-5-5";
pub const API_URL: &str = "https://api.anthropic.com/v1/messages";
pub const API_VERSION: &str = "2026-10-21";
/// Recusa por política: a API refaz o pedido em outro modelo dentro da mesma chamada.
pub const FALLBACK_BETA: &str = "server-side-fallback-2026-07-01";
pub const SETTING_KEY: &str = "anthropic_api_key";
pub const DEFAULT_ROUNDS: usize = 4;
const MAX_ERROR_CHARS: usize = 6000;

#[derive(Debug, Clone)]
pub struct AiTranslation {
    pub code: String,
    pub rounds: usize,
    pub compiled: bool,
    pub unsafe_count: usize,
    /// Última saída do `rustc` quando não compilou.
    pub last_error: Option<String>,
}

/// Resposta do modelo: o array `content` inteiro (para reenviar sem alterar) e o motivo de parada.
#[derive(Debug, Clone)]
pub struct ModelReply {
    pub content: Value,
    pub stop_reason: String,
}

fn system_prompt(mode: TranslateMode) -> String {
    let base = "Você traduz código C ou C++ para Rust. Responda com o arquivo Rust completo em um único bloco ```rust. \
O resultado precisa compilar como biblioteca (rustc --crate-type=lib --edition=2021), sem crates externas.";
    match mode {
        TranslateMode::SafeRust => format!(
            "{base} Prefira Rust seguro e idiomático (Vec, String, slices, Option, Result, iteradores). \
Use `unsafe` só quando não houver alternativa, com um comentário curto explicando o motivo."
        ),
        TranslateMode::FfiBindings => format!(
            "{base} Gere código compatível com a ABI C: #[repr(C)] nas structs, extern \"C\" nas funções \
exportadas e ponteiros brutos onde o C usa ponteiros."
        ),
    }
}

/// Corpo do pedido (sem `temperature`: a versão 2026-10-21 da API recusa parâmetros de amostragem).
pub fn request_body(mode: TranslateMode, messages: &[Value]) -> Value {
    json!({
        "model": MODEL,
        "max_tokens": 16000,
        "system": system_prompt(mode),
        "messages": messages,
        "output_config": {"effort": "high"},
        "fallbacks": "default",
    })
}

/// Junta os blocos de texto da resposta (ignora blocos de raciocínio e de troca de modelo).
pub fn reply_text(content: &Value) -> String {
    content
        .as_array()
        .map(|blocks| {
            blocks
                .iter()
                .filter(|b| b["type"] == "text")
                .filter_map(|b| b["text"].as_str())
                .collect::<Vec<_>>()
                .join("\n")
        })
        .unwrap_or_default()
}

/// Primeiro bloco de código marcado como Rust da resposta (ou o primeiro bloco sem linguagem).
pub fn extract_rust_code(text: &str) -> Option<String> {
    let mut rest = text;
    let mut unlabeled: Option<String> = None;
    while let Some(start) = rest.find("```") {
        let after = &rest[start + 3..];
        let Some(nl) = after.find('\n') else { break };
        let label = after[..nl].trim().to_lowercase();
        let body = &after[nl + 1..];
        let Some(end) = body.find("```") else { break };
        let code = body[..end].trim_end();
        if !code.trim().is_empty() {
            if label == "rust" || label == "rs" {
                return Some(format!("{code}\n"));
            }
            if label.is_empty() && unlabeled.is_none() {
                unlabeled = Some(format!("{code}\n"));
            }
        }
        rest = &body[end + 3..];
    }
    unlabeled
}

/// Quantas vezes a palavra `unsafe` aparece fora de comentários de linha.
pub fn count_unsafe(code: &str) -> usize {
    code.lines()
        .map(|l| l.split("//").next().unwrap_or(""))
        .map(|l| {
            l.match_indices("unsafe")
                .filter(|(i, _)| {
                    let before = l[..*i].chars().last();
                    let after = l[i + 6..].chars().next();
                    !before.is_some_and(|c| c.is_alphanumeric() || c == '_')
                        && !after.is_some_and(|c| c.is_alphanumeric() || c == '_')
                })
                .count()
        })
        .sum()
}

fn truncate_errors(s: &str) -> String {
    if s.chars().count() <= MAX_ERROR_CHARS {
        s.to_string()
    } else {
        format!("{}\n[... saída do rustc cortada ...]", s.chars().take(MAX_ERROR_CHARS).collect::<String>())
    }
}

/// Laço tradução → `rustc` → correção. `ask` envia o histórico e devolve a resposta do modelo;
/// `check` compila e devolve `Err(saída do rustc)` quando falha. Sem rede aqui, para dar para testar.
pub async fn translate_loop<F, Fut, C>(
    c_code: &str,
    max_rounds: usize,
    mut ask: F,
    check: C,
) -> Result<AiTranslation>
where
    F: FnMut(Vec<Value>) -> Fut,
    Fut: Future<Output = Result<ModelReply>>,
    C: Fn(&str) -> Result<String, String>,
{
    let mut messages = vec![json!({
        "role": "user",
        "content": format!("Traduza para Rust:\n```c\n{}\n```", c_code.trim_start_matches('\u{feff}')),
    })];
    let mut best: Option<(String, String)> = None;
    for round in 1..=max_rounds.max(1) {
        let reply = ask(messages.clone()).await?;
        match reply.stop_reason.as_str() {
            "refusal" => bail!("o modelo recusou o pedido (o código pode ter sido classificado como sensível)"),
            "max_tokens" => bail!("a resposta passou do limite de tamanho; traduza um arquivo menor por vez"),
            _ => {}
        }
        let text = reply_text(&reply.content);
        messages.push(json!({"role": "assistant", "content": reply.content}));
        let Some(code) = extract_rust_code(&text) else {
            messages.push(json!({
                "role": "user",
                "content": "Não encontrei um bloco ```rust na resposta. Responda com o arquivo Rust completo em um bloco ```rust.",
            }));
            continue;
        };
        match check(&code) {
            Ok(_) => {
                return Ok(AiTranslation {
                    unsafe_count: count_unsafe(&code),
                    code,
                    rounds: round,
                    compiled: true,
                    last_error: None,
                });
            }
            Err(errors) => {
                messages.push(json!({
                    "role": "user",
                    "content": format!(
                        "O rustc recusou o código com estes erros:\n```\n{}\n```\nCorrija e responda de novo com o arquivo Rust completo.",
                        truncate_errors(&errors)
                    ),
                }));
                best = Some((code, errors));
            }
        }
    }
    match best {
        Some((code, errors)) => Ok(AiTranslation {
            unsafe_count: count_unsafe(&code),
            code,
            rounds: max_rounds,
            compiled: false,
            last_error: Some(errors),
        }),
        None => bail!("o modelo não devolveu código Rust em {max_rounds} tentativa(s)"),
    }
}

/// Envia um pedido à API do Claude. Repete até 2 vezes em limite de uso (429) e erro do servidor (5xx).
pub async fn ask_claude(client: &reqwest::Client, api_key: &str, mode: TranslateMode, messages: Vec<Value>) -> Result<ModelReply> {
    let body = request_body(mode, &messages);
    let mut attempt = 0;
    loop {
        attempt += 1;
        let resp = client
            .post(API_URL)
            .header("x-api-key", api_key.trim())
            .header("anthropic-version", API_VERSION)
            .header("anthropic-beta", FALLBACK_BETA)
            .json(&body)
            .send()
            .await
            .context("sem conexão com a API do Claude")?;
        let status = resp.status();
        let payload: Value = resp.json().await.unwrap_or(Value::Null);
        if status.is_success() {
            return Ok(ModelReply {
                content: payload["content"].clone(),
                stop_reason: payload["stop_reason"].as_str().unwrap_or_default().to_string(),
            });
        }
        let api_msg = payload["error"]["message"].as_str().unwrap_or("").to_string();
        let retryable = status.as_u16() == 429 || status.is_server_error();
        if retryable && attempt <= 2 {
            tokio::time::sleep(std::time::Duration::from_secs(3 * attempt)).await;
            continue;
        }
        bail!("{}", describe_http_error(status.as_u16(), &api_msg));
    }
}

/// Mensagem para o usuário a partir do código HTTP da API.
pub fn describe_http_error(status: u16, api_msg: &str) -> String {
    let what = match status {
        400 => "pedido recusado pela API",
        401 => "chave de API inválida (confira em Configurar Chaves API)",
        403 => "a chave não tem permissão para este modelo",
        404 => "modelo ou endereço não encontrado",
        413 => "o código é grande demais para um pedido",
        429 => "limite de uso da API atingido; tente de novo em alguns minutos",
        529 => "API sobrecarregada; tente de novo em alguns minutos",
        s if s >= 500 => "erro no servidor da API; tente de novo",
        _ => "erro na API",
    };
    if api_msg.is_empty() {
        format!("{what} (HTTP {status})")
    } else {
        format!("{what} (HTTP {status}): {api_msg}")
    }
}

/// Tradução completa pela API, com o laço do `rustc`.
pub async fn translate_with_claude(api_key: &str, c_code: &str, mode: TranslateMode, max_rounds: usize) -> Result<AiTranslation> {
    if api_key.trim().is_empty() {
        bail!("defina a chave da API do Claude em Configurar Chaves API (ou a variável ANTHROPIC_API_KEY)");
    }
    let client = reqwest::Client::builder()
        .user_agent("gitstore/0.3")
        .timeout(std::time::Duration::from_secs(600))
        .build()?;
    translate_loop(
        c_code,
        max_rounds,
        |messages| {
            let client = client.clone();
            let key = api_key.to_string();
            async move { ask_claude(&client, &key, mode, messages).await }
        },
        crate::translator::verify_rust_syntax,
    )
    .await
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::cell::RefCell;

    fn reply(text: &str) -> ModelReply {
        ModelReply {
            content: json!([{"type": "thinking", "thinking": "", "signature": "x"}, {"type": "text", "text": text}]),
            stop_reason: "end_turn".into(),
        }
    }

    #[test]
    fn extrai_o_bloco_rust() {
        assert_eq!(extract_rust_code("Aqui:\n```rust\nfn a() {}\n```\nfim").as_deref(), Some("fn a() {}\n"));
        assert_eq!(extract_rust_code("```\npub fn b() {}\n```").as_deref(), Some("pub fn b() {}\n"));
        assert!(extract_rust_code("sem código").is_none());
        assert!(extract_rust_code("```rust\n```").is_none());
    }

    #[test]
    fn conta_unsafe_fora_de_comentarios() {
        let code = "unsafe fn a() {}\n// unsafe aqui não conta\nfn b() { unsafe { x() } }\nlet not_unsafe_name = 1;";
        assert_eq!(count_unsafe(code), 2);
    }

    #[test]
    fn corpo_do_pedido_segue_a_api_atual() {
        let body = request_body(TranslateMode::SafeRust, &[json!({"role": "user", "content": "x"})]);
        assert_eq!(body["model"], "claude-opus-5-5");
        assert_eq!(body["fallbacks"], "default");
        assert_eq!(body["output_config"]["effort"], "high");
        assert!(body.get("temperature").is_none(), "a API 2026-10-21 recusa temperature");
        assert!(body.get("thinking").is_none(), "no Opus 5.5 o raciocínio é sempre ligado; não enviar");
        assert_eq!(reply_text(&reply("oi").content), "oi");
    }

    #[test]
    fn erros_http_viram_mensagens_claras() {
        assert!(describe_http_error(401, "").contains("chave de API inválida"));
        assert!(describe_http_error(429, "rate").contains("limite de uso"));
        assert!(describe_http_error(400, "bad field").contains("bad field"));
    }

    #[test]
    fn laco_corrige_com_o_erro_do_rustc_e_mantem_a_resposta_inteira() {
        let sent: RefCell<Vec<Vec<Value>>> = RefCell::new(Vec::new());
        let answers = RefCell::new(vec![
            reply("```rust\npub fn soma(a: i32, b: i32) -> i32 { a + b + c }\n```"),
            reply("```rust\npub fn soma(a: i32, b: i32) -> i32 { a + b }\n```"),
        ]);
        let rt = tokio::runtime::Runtime::new().unwrap();
        let res = rt
            .block_on(translate_loop(
                "int soma(int a, int b) { return a + b; }",
                4,
                |msgs| {
                    sent.borrow_mut().push(msgs);
                    let next = answers.borrow_mut().remove(0);
                    async move { Ok(next) }
                },
                crate::translator::verify_rust_syntax,
            ))
            .unwrap();
        assert!(res.compiled);
        assert_eq!(res.rounds, 2);
        assert_eq!(res.unsafe_count, 0);
        let second = &sent.borrow()[1];
        assert_eq!(second.len(), 3, "user, assistant, user com o erro");
        assert_eq!(second[1]["content"][0]["type"], "thinking", "a resposta volta inteira, sem editar");
        assert!(second[2]["content"].as_str().unwrap().contains("cannot find value `c`"), "{}", second[2]);
    }

    #[test]
    fn laco_para_em_recusa_e_quando_nao_compila_devolve_o_melhor_esforco() {
        let rt = tokio::runtime::Runtime::new().unwrap();
        let refused = rt.block_on(translate_loop(
            "int x;",
            3,
            |_| async {
                Ok(ModelReply {
                    content: json!([]),
                    stop_reason: "refusal".into(),
                })
            },
            |_| Ok(String::new()),
        ));
        assert!(refused.unwrap_err().to_string().contains("recusou"));

        let never = rt
            .block_on(translate_loop(
                "int x;",
                2,
                |_| async { Ok(reply("```rust\nfn quebrado( {\n```")) },
                crate::translator::verify_rust_syntax,
            ))
            .unwrap();
        assert!(!never.compiled);
        assert_eq!(never.rounds, 2);
        assert!(never.last_error.is_some());
    }
}
