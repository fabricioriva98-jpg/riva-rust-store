//! Compartilhamento de biblioteca e texto de apps usando GitHub Gists e links diretos.

use crate::db::App;
use anyhow::{bail, Context, Result};
use serde_json::{json, Value};

pub const LIBRARY_GIST_FILE: &str = "rrs-biblioteca.json";

/// ID do gist a partir de um link (`https://gist.github.com/dono/ID`) ou do próprio ID.
pub fn gist_id_from_url(input: &str) -> Option<String> {
    let s = input.trim().trim_end_matches('/');
    let id = s.rsplit('/').next()?.split('#').next()?.split('?').next()?;
    let ok = id.len() >= 20 && id.len() <= 40 && id.chars().all(|c| c.is_ascii_hexdigit());
    ok.then(|| id.to_lowercase())
}

/// Texto para compartilhar um app com alguém (vai para a área de transferência).
pub fn share_text(app: &App, title: &str) -> String {
    format!(
        "{title} — {}\n{}\nNa RRS - Riva Rust Store: adicione \"{}\" no campo \"Adicionar dono/repo\" para ver a nota de confiança e baixar com SHA-256 conferido.",
        app.description.trim(),
        app.url,
        app.full_name
    )
}

fn http() -> Result<reqwest::Client> {
    Ok(reqwest::Client::builder().user_agent("gitstore/0.3").build()?)
}

fn auth(req: reqwest::RequestBuilder, token: &str) -> reqwest::RequestBuilder {
    req.header("Authorization", format!("Bearer {}", token.trim()))
        .header("Accept", "application/vnd.github+json")
        .header("X-GitHub-Api-Version", "2022-11-28")
}

async fn check(resp: reqwest::Response, what: &str) -> Result<Value> {
    let status = resp.status();
    if status == reqwest::StatusCode::NOT_FOUND {
        bail!("{what}: não encontrado (o gist pode ter sido apagado)");
    }
    if status == reqwest::StatusCode::UNAUTHORIZED || status == reqwest::StatusCode::FORBIDDEN {
        bail!("{what}: o login do GitHub não tem permissão (entre de novo; é preciso o escopo \"gist\")");
    }
    if !status.is_success() {
        bail!("{what}: o GitHub respondeu HTTP {status}");
    }
    if status == reqwest::StatusCode::NO_CONTENT {
        return Ok(Value::Null);
    }
    Ok(resp.json().await.unwrap_or(Value::Null))
}

/// Gist secreto com a biblioteca; devolve o link para mandar aos amigos.
pub async fn share_library(token: &str, library_json: &str) -> Result<String> {
    let body = json!({
        "description": "RRS - Riva Rust Store: biblioteca compartilhada",
        "public": false,
        "files": {LIBRARY_GIST_FILE: {"content": library_json}},
    });
    let resp = auth(http()?.post("https://api.github.com/gists"), token).json(&body).send().await?;
    let v = check(resp, "compartilhar biblioteca").await?;
    v["html_url"].as_str().map(String::from).context("resposta sem link do gist")
}

/// Conteúdo de um arquivo de um gist (gists secretos também abrem com o link).
pub async fn fetch_gist_file(gist_id: &str, file: &str, token: Option<&str>) -> Result<String> {
    let mut req = http()?.get(format!("https://api.github.com/gists/{gist_id}"));
    req = match token {
        Some(t) => auth(req, t),
        None => req.header("Accept", "application/vnd.github+json"),
    };
    let v = check(req.send().await?, "abrir gist").await?;
    let f = &v["files"][file];
    if f.is_null() {
        bail!("o gist não tem o arquivo {file}");
    }
    if f["truncated"].as_bool().unwrap_or(false) {
        let raw = f["raw_url"].as_str().context("gist sem raw_url")?;
        return Ok(http()?.get(raw).send().await?.text().await?);
    }
    f["content"].as_str().map(String::from).context("gist sem conteúdo")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn id_do_gist_a_partir_do_link() {
        assert_eq!(
            gist_id_from_url("https://gist.github.com/fulano/aa5a315d61ae9438b18d").as_deref(),
            Some("aa5a315d61ae9438b18d")
        );
        assert_eq!(gist_id_from_url("AA5A315D61AE9438B18D/").as_deref(), Some("aa5a315d61ae9438b18d"));
        assert!(gist_id_from_url("https://exemplo.com/../../etc").is_none());
        assert!(gist_id_from_url("").is_none());
    }
}
