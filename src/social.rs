//! Perfil público, amigos e compartilhamento, usando o GitHub como "servidor" (sem servidor próprio).
//!
//! - Perfil público: só quando o usuário liga "Publicar perfil". Vai para um Gist **público** na conta
//!   dele, com o arquivo `rrs-perfil.json` (nível, conquistas e os apps que ele mais usa).
//! - Amigos: quem o usuário segue no GitHub e também publicou o `rrs-perfil.json`.
//! - Compartilhar biblioteca: Gist **secreto** (só quem tem o link vê) com o `rrs-biblioteca.json`.
//!
//! Perfis de amigos são dados de terceiros: só são mostrados como texto, com tamanho limitado.

use crate::achievements::{self, Progress};
use crate::db::App;
use anyhow::{bail, Context, Result};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

pub const PROFILE_FILE: &str = "rrs-perfil.json";
pub const PROFILE_FORMAT: &str = "rrs-perfil";
pub const LIBRARY_GIST_FILE: &str = "rrs-biblioteca.json";
pub const SETTING_PROFILE_GIST: &str = "perfil_gist_id";
const MAX_FRIENDS: usize = 50;
const MAX_TEXT: usize = 80;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct PublicApp {
    pub full_name: String,
    #[serde(default)]
    pub horas: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct PublicProfile {
    pub formato: String,
    pub versao: u32,
    pub login: String,
    pub nivel: u32,
    #[serde(default)]
    pub titulo: String,
    pub xp: u64,
    #[serde(default)]
    pub conquistas: Vec<String>,
    #[serde(default)]
    pub jogando: Vec<PublicApp>,
    #[serde(default)]
    pub atualizado_em: String,
}

#[derive(Debug, Clone)]
pub struct Friend {
    pub login: String,
    pub avatar_url: String,
    /// `None` = segue no GitHub, mas não publicou perfil da RRS.
    pub profile: Option<PublicProfile>,
}

fn clip(s: &str) -> String {
    s.chars().take(MAX_TEXT).collect()
}

/// Perfil público a partir do progresso local: só nível, conquistas e os 10 apps mais jogados.
pub fn build_profile(login: &str, progress: &Progress, apps: &[App]) -> PublicProfile {
    let mut playing: Vec<PublicApp> = apps
        .iter()
        .filter_map(|a| {
            a.installed.as_ref().map(|i| PublicApp {
                full_name: a.full_name.clone(),
                horas: (i.playtime_seconds as f64 / 360.0).round() / 10.0,
            })
        })
        .collect();
    playing.sort_by(|a, b| b.horas.partial_cmp(&a.horas).unwrap_or(std::cmp::Ordering::Equal));
    playing.truncate(10);
    PublicProfile {
        formato: PROFILE_FORMAT.into(),
        versao: 1,
        login: login.to_string(),
        nivel: progress.level,
        titulo: achievements::title_for(progress.level).to_string(),
        xp: progress.xp,
        conquistas: progress.unlocked.iter().map(|(id, _)| id.clone()).collect(),
        jogando: playing,
        atualizado_em: chrono::Utc::now().to_rfc3339(),
    }
}

/// Lê e limpa um perfil de terceiros (tamanhos limitados, só conquistas conhecidas e nomes de repositório válidos).
pub fn parse_profile(text: &str) -> Result<PublicProfile> {
    let mut p: PublicProfile = serde_json::from_str(text.trim_start_matches('\u{feff}')).context("perfil inválido")?;
    if p.formato != PROFILE_FORMAT {
        bail!("não é um perfil da RRS");
    }
    p.login = clip(&p.login);
    p.titulo = clip(&p.titulo);
    p.nivel = p.nivel.min(999);
    p.conquistas.retain(|id| achievements::find(id).is_some());
    p.conquistas.dedup();
    p.jogando.retain(|a| crate::forge::RepoRef::parse(&a.full_name).is_some());
    p.jogando.truncate(10);
    for a in &mut p.jogando {
        a.full_name = clip(&a.full_name);
        a.horas = a.horas.clamp(0.0, 100_000.0);
    }
    Ok(p)
}

/// Endereço bruto do `rrs-perfil.json` na lista de gists de um usuário.
pub fn find_profile_raw_url(gists: &Value) -> Option<String> {
    gists.as_array()?.iter().find_map(|g| {
        g["files"][PROFILE_FILE]["raw_url"]
            .as_str()
            .filter(|u| u.starts_with("https://gist.githubusercontent.com/"))
            .map(String::from)
    })
}

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

/// Cria (ou atualiza) o gist público do perfil. Devolve o ID do gist.
pub async fn publish_profile(token: &str, existing_gist: Option<&str>, profile: &PublicProfile) -> Result<String> {
    let body = json!({
        "description": "RRS - Riva Rust Store: perfil público",
        "public": true,
        "files": {PROFILE_FILE: {"content": serde_json::to_string_pretty(profile)?}},
    });
    let client = http()?;
    let resp = match existing_gist {
        Some(id) => auth(client.patch(format!("https://api.github.com/gists/{id}")), token).json(&body).send().await?,
        None => auth(client.post("https://api.github.com/gists"), token).json(&body).send().await?,
    };
    let v = check(resp, "publicar perfil").await?;
    v["id"].as_str().map(String::from).context("resposta sem id do gist")
}

/// Apaga o gist do perfil (o perfil deixa de ser público).
pub async fn unpublish_profile(token: &str, gist_id: &str) -> Result<()> {
    let resp = auth(http()?.delete(format!("https://api.github.com/gists/{gist_id}")), token)
        .send()
        .await?;
    check(resp, "despublicar perfil").await.map(|_| ())
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

/// Quem o usuário segue no GitHub, com o perfil da RRS de quem publicou.
pub async fn friends(token: &str) -> Result<Vec<Friend>> {
    let client = http()?;
    let following = check(
        auth(client.get(format!("https://api.github.com/user/following?per_page={MAX_FRIENDS}")), token)
            .send()
            .await?,
        "lista de quem você segue",
    )
    .await?;
    let mut out = Vec::new();
    for u in following.as_array().cloned().unwrap_or_default().iter().take(MAX_FRIENDS) {
        let Some(login) = u["login"].as_str() else { continue };
        let avatar_url = u["avatar_url"].as_str().unwrap_or_default().to_string();
        let gists = auth(client.get(format!("https://api.github.com/users/{login}/gists?per_page=30")), token)
            .send()
            .await
            .ok();
        let mut profile = None;
        if let Some(resp) = gists.filter(|r| r.status().is_success()) {
            if let Some(raw) = find_profile_raw_url(&resp.json().await.unwrap_or(Value::Null)) {
                if let Ok(text) = client.get(&raw).send().await.and_then(|r| r.error_for_status()) {
                    profile = parse_profile(&text.text().await.unwrap_or_default()).ok();
                }
            }
        }
        out.push(Friend { login: clip(login), avatar_url, profile });
    }
    // Quem publicou perfil primeiro, depois por nível
    out.sort_by_key(|f| std::cmp::Reverse(f.profile.as_ref().map(|p| p.nivel + 1).unwrap_or(0)));
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn progress() -> Progress {
        Progress {
            xp: 450,
            level: 3,
            unlocked: vec![("modo_sofa".into(), "2026-10-24T00:00:00Z".into())],
            ..Default::default()
        }
    }

    #[test]
    fn perfil_publico_tem_so_o_necessario() {
        let conn = crate::db::open(":memory:").unwrap();
        crate::db::import_catalog_str(
            &conn,
            r#"[{"full_name":"dono/jogo","description":"","url":"","stars":1,"license":"","topics":"","pushed_at":"","latest_tag":"v1","platforms":"windows","assets":[],"trust_level":"x","trust_notes":["x"]},
                {"full_name":"dono/nao-instalado","description":"","url":"","stars":1,"license":"","topics":"","pushed_at":"","latest_tag":"v1","platforms":"windows","assets":[],"trust_level":"x","trust_notes":["x"]}]"#,
            "t",
        )
        .unwrap();
        crate::db::mark_installed(&conn, "dono/jogo", "v1", "C:\\Users\\fulano\\jogo.exe", "abc", true).unwrap();
        crate::db::record_playtime(&conn, "dono/jogo", 5400).unwrap();
        let p = build_profile("fulano", &progress(), &crate::db::list_all(&conn).unwrap());
        assert_eq!(p.nivel, 3);
        assert_eq!(p.titulo, "Explorador");
        assert_eq!(p.jogando, vec![PublicApp { full_name: "dono/jogo".into(), horas: 1.5 }]);
        let json = serde_json::to_string(&p).unwrap();
        assert!(!json.contains("C:\\\\Users"), "caminho local não pode ir para o perfil público");
        assert!(!json.contains("abc"), "hash local não precisa ir para o perfil");
        assert_eq!(parse_profile(&json).unwrap(), p);
    }

    #[test]
    fn perfil_de_terceiro_e_limpo() {
        let texto = format!(
            r#"{{"formato":"rrs-perfil","versao":1,"login":"{}","nivel":99999,"xp":5,
                 "conquistas":["modo_sofa","inventada","modo_sofa"],
                 "jogando":[{{"full_name":"dono/jogo","horas":2}},{{"full_name":"<script>","horas":-3}}]}}"#,
            "x".repeat(500)
        );
        let p = parse_profile(&texto).unwrap();
        assert_eq!(p.login.chars().count(), MAX_TEXT);
        assert_eq!(p.nivel, 999);
        assert_eq!(p.conquistas, vec!["modo_sofa".to_string()]);
        assert_eq!(p.jogando.len(), 1);
        assert!(parse_profile(r#"{"formato":"outro","versao":1,"login":"a","nivel":1,"xp":1}"#).is_err());
    }

    #[test]
    fn acha_o_perfil_na_lista_de_gists() {
        let gists = json!([
            {"files": {"notas.md": {"raw_url": "https://gist.githubusercontent.com/a/1/raw/notas.md"}}},
            {"files": {"rrs-perfil.json": {"raw_url": "https://gist.githubusercontent.com/a/2/raw/rrs-perfil.json"}}}
        ]);
        assert_eq!(
            find_profile_raw_url(&gists).as_deref(),
            Some("https://gist.githubusercontent.com/a/2/raw/rrs-perfil.json")
        );
        let falso = json!([{"files": {"rrs-perfil.json": {"raw_url": "https://exemplo.com/perfil.json"}}}]);
        assert!(find_profile_raw_url(&falso).is_none(), "só endereços do próprio GitHub");
    }

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
