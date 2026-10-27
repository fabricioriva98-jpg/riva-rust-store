//! Login com o GitHub e perfil do usuário.
//!
//! Dois caminhos:
//! - **Código no navegador** (OAuth Device Flow): precisa de um "OAuth App" registrado no GitHub com
//!   "Enable Device Flow"; o Client ID (não é segredo) fica em `settings.github_oauth_client_id`.
//! - **Token pessoal**: o usuário cola um token criado em github.com/settings/tokens.
//!
//! O token vai para o Gerenciador de Credenciais do Windows (crate `keyring`), não para o SQLite.
//! Escopos pedidos: `read:user` (perfil) e `gist` (publicar perfil e compartilhar biblioteca).

use anyhow::{anyhow, bail, Context, Result};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

pub const KEYRING_SERVICE: &str = "RRS - Riva Rust Store";
pub const KEYRING_USER: &str = "github";
pub const SETTING_CLIENT_ID: &str = "github_oauth_client_id";
pub const SETTING_PROFILE_CACHE: &str = "github_perfil";
pub const SCOPES: &str = "read:user gist";

#[derive(Debug, Clone, PartialEq)]
pub struct DeviceCode {
    pub device_code: String,
    pub user_code: String,
    pub verification_uri: String,
    pub expires_in: u64,
    pub interval: u64,
}

#[derive(Debug, Clone, PartialEq)]
pub enum PollResult {
    Token(String),
    Pending,
    SlowDown,
    Expired,
    Denied,
    Error(String),
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
pub struct GithubUser {
    pub login: String,
    #[serde(default)]
    pub name: Option<String>,
    #[serde(default)]
    pub avatar_url: String,
    #[serde(default)]
    pub html_url: String,
    #[serde(default)]
    pub public_repos: u64,
    #[serde(default)]
    pub followers: u64,
    #[serde(default)]
    pub following: u64,
    #[serde(default)]
    pub created_at: String,
}

pub fn parse_device_code(v: &Value) -> Result<DeviceCode> {
    if let Some(err) = v["error"].as_str() {
        bail!(
            "o GitHub recusou o pedido de código ({err}): confira o Client ID do OAuth App e se \"Enable Device Flow\" está marcado"
        );
    }
    Ok(DeviceCode {
        device_code: v["device_code"].as_str().context("resposta sem device_code")?.to_string(),
        user_code: v["user_code"].as_str().context("resposta sem user_code")?.to_string(),
        verification_uri: v["verification_uri"]
            .as_str()
            .unwrap_or("https://github.com/login/device")
            .to_string(),
        expires_in: v["expires_in"].as_u64().unwrap_or(900),
        interval: v["interval"].as_u64().unwrap_or(5).max(1),
    })
}

pub fn parse_poll(v: &Value) -> PollResult {
    if let Some(token) = v["access_token"].as_str().filter(|t| !t.is_empty()) {
        return PollResult::Token(token.to_string());
    }
    match v["error"].as_str() {
        Some("authorization_pending") => PollResult::Pending,
        Some("slow_down") => PollResult::SlowDown,
        Some("expired_token") => PollResult::Expired,
        Some("access_denied") => PollResult::Denied,
        Some(other) => PollResult::Error(other.to_string()),
        None => PollResult::Error("resposta inesperada do GitHub".into()),
    }
}

fn client() -> Result<reqwest::Client> {
    Ok(reqwest::Client::builder()
        .user_agent("gitstore/0.3")
        .connect_timeout(std::time::Duration::from_secs(15))
        .timeout(std::time::Duration::from_secs(30))
        .build()?)
}

pub async fn request_device_code(client_id: &str) -> Result<DeviceCode> {
    if client_id.trim().is_empty() {
        bail!("falta o Client ID do OAuth App (veja \"Como registrar\" nas Configurações) ou entre com um token pessoal");
    }
    let resp = client()?
        .post("https://github.com/login/device/code")
        .header("Accept", "application/json")
        .form(&[("client_id", client_id.trim()), ("scope", SCOPES)])
        .send()
        .await
        .context("sem conexão com o GitHub")?;
    parse_device_code(&resp.json().await?)
}

pub async fn poll_token(client_id: &str, device_code: &str) -> Result<PollResult> {
    let resp = client()?
        .post("https://github.com/login/oauth/access_token")
        .header("Accept", "application/json")
        .form(&[
            ("client_id", client_id.trim()),
            ("device_code", device_code),
            ("grant_type", "urn:ietf:params:oauth:grant-type:device_code"),
        ])
        .send()
        .await
        .context("sem conexão com o GitHub")?;
    Ok(parse_poll(&resp.json().await?))
}

/// Espera o usuário autorizar no navegador, respeitando o intervalo pedido pelo GitHub.
pub async fn wait_for_token(client_id: &str, code: &DeviceCode, cancel: Arc<AtomicBool>) -> Result<String> {
    let mut interval = code.interval;
    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(code.expires_in);
    loop {
        tokio::time::sleep(std::time::Duration::from_secs(interval)).await;
        if cancel.load(Ordering::Relaxed) {
            bail!("login cancelado");
        }
        if std::time::Instant::now() > deadline {
            bail!("o código expirou; peça um novo");
        }
        match poll_token(client_id, &code.device_code).await? {
            PollResult::Token(t) => return Ok(t),
            PollResult::Pending => {}
            PollResult::SlowDown => interval += 5,
            PollResult::Expired => bail!("o código expirou; peça um novo"),
            PollResult::Denied => bail!("o acesso foi negado no GitHub"),
            PollResult::Error(e) => bail!("erro no login do GitHub: {e}"),
        }
    }
}

pub async fn fetch_user(token: &str) -> Result<GithubUser> {
    let resp = client()?
        .get("https://api.github.com/user")
        .header("Authorization", format!("Bearer {}", token.trim()))
        .header("Accept", "application/vnd.github+json")
        .send()
        .await
        .context("sem conexão com o GitHub")?;
    if resp.status() == reqwest::StatusCode::UNAUTHORIZED {
        bail!("token inválido ou revogado");
    }
    if !resp.status().is_success() {
        bail!("o GitHub respondeu HTTP {}", resp.status());
    }
    Ok(resp.json().await?)
}

fn entry() -> Result<keyring::Entry> {
    keyring::Entry::new(KEYRING_SERVICE, KEYRING_USER).map_err(|e| anyhow!("cofre de credenciais indisponível: {e}"))
}

pub fn save_token(token: &str) -> Result<()> {
    entry()?
        .set_password(token.trim())
        .map_err(|e| anyhow!("não foi possível guardar o token no Gerenciador de Credenciais: {e}"))
}

pub fn load_token() -> Option<String> {
    entry().ok()?.get_password().ok().filter(|t| !t.trim().is_empty())
}

pub fn delete_token() -> Result<()> {
    match entry()?.delete_credential() {
        Ok(()) | Err(keyring::Error::NoEntry) => Ok(()),
        Err(e) => Err(anyhow!("não foi possível remover o token: {e}")),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn le_o_codigo_do_device_flow() {
        let v = json!({"device_code": "abc", "user_code": "WDJB-MJHT", "verification_uri": "https://github.com/login/device", "expires_in": 899, "interval": 5});
        let c = parse_device_code(&v).unwrap();
        assert_eq!(c.user_code, "WDJB-MJHT");
        assert_eq!(c.interval, 5);
        let err = parse_device_code(&json!({"error": "unauthorized_client"})).unwrap_err();
        assert!(err.to_string().contains("Enable Device Flow"), "{err}");
    }

    #[test]
    fn interpreta_a_espera_pelo_token() {
        assert_eq!(parse_poll(&json!({"access_token": "gho_x", "token_type": "bearer"})), PollResult::Token("gho_x".into()));
        assert_eq!(parse_poll(&json!({"error": "authorization_pending"})), PollResult::Pending);
        assert_eq!(parse_poll(&json!({"error": "slow_down", "interval": 10})), PollResult::SlowDown);
        assert_eq!(parse_poll(&json!({"error": "expired_token"})), PollResult::Expired);
        assert_eq!(parse_poll(&json!({"error": "access_denied"})), PollResult::Denied);
        assert!(matches!(parse_poll(&json!({})), PollResult::Error(_)));
    }

    #[test]
    fn perfil_do_github() {
        let u: GithubUser = serde_json::from_value(json!({
            "login": "fulano", "name": null, "avatar_url": "https://avatars.githubusercontent.com/u/1",
            "html_url": "https://github.com/fulano", "public_repos": 3, "followers": 10, "following": 4,
            "created_at": "2015-01-01T00:00:00Z", "plan": {"name": "free"}
        }))
        .unwrap();
        assert_eq!(u.login, "fulano");
        assert!(u.name.is_none());
        assert_eq!(u.following, 4);
    }

    #[test]
    fn sem_client_id_nao_pede_codigo() {
        let rt = tokio::runtime::Runtime::new().unwrap();
        let err = rt.block_on(request_device_code("  ")).unwrap_err();
        assert!(err.to_string().contains("Client ID"));
    }
}
