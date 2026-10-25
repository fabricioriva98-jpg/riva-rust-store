//! Catálogo online assinado (ed25519), no espírito do TUF: a assinatura cobre o SHA-256 do catálogo,
//! um número de versão crescente (bloqueia volta para catálogo antigo) e uma data de validade
//! (bloqueia catálogo "congelado" servido para sempre).
//!
//! Arquivos publicados: `catalog.json` e `catalog.json.sig` (JSON com os campos de [`CatalogSignature`]).
//! Chaves confiáveis: [`TRUSTED_KEYS`] (embutidas no .exe) e, opcionalmente, a chave salva em
//! `settings.catalog_pubkey`. Sem nenhuma chave, a atualização online fica recusada.

use crate::db;
use anyhow::{bail, Context, Result};
use chrono::{DateTime, Duration, Utc};
use ed25519_dalek::{Signature, Signer, SigningKey, VerifyingKey};
use rusqlite::Connection;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::path::Path;

pub const SIG_FORMAT: &str = "rrs-catalogo-assinado";
/// Chaves públicas (hex) do catálogo oficial. Vazio até a chave oficial ser gerada (ver CONTINUIDADE-IA.md).
pub const TRUSTED_KEYS: &[&str] = &[];
pub const SETTING_EXTRA_KEY: &str = "catalog_pubkey";
pub const SETTING_URL: &str = "catalog_url";
pub const SETTING_VERSION: &str = "catalog_version";

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct CatalogSignature {
    pub formato: String,
    pub versao: u64,
    /// RFC 3339; depois dessa data o catálogo é recusado.
    pub expira_em: String,
    /// SHA-256 (hex) dos bytes exatos do catalog.json.
    pub sha256: String,
    pub chave_publica: String,
    pub assinatura: String,
}

fn signed_message(versao: u64, expira_em: &str, sha256: &str) -> String {
    format!("rrs-catalogo-v1\n{versao}\n{expira_em}\n{sha256}")
}

pub fn to_hex(bytes: &[u8]) -> String {
    bytes.iter().map(|b| format!("{b:02x}")).collect()
}

pub fn from_hex(s: &str) -> Result<Vec<u8>> {
    let s = s.trim();
    if !s.len().is_multiple_of(2) || !s.bytes().all(|b| b.is_ascii_hexdigit()) {
        bail!("texto hexadecimal inválido");
    }
    (0..s.len())
        .step_by(2)
        .map(|i| u8::from_str_radix(&s[i..i + 2], 16).context("hex inválido"))
        .collect()
}

fn fixed<const N: usize>(bytes: Vec<u8>, what: &str) -> Result<[u8; N]> {
    bytes
        .try_into()
        .map_err(|_| anyhow::anyhow!("{what} com tamanho errado"))
}

/// Gera um par de chaves: (privada em hex, pública em hex). A privada nunca deve ir para o repositório.
pub fn generate_keypair() -> (String, String) {
    let sk = SigningKey::generate(&mut rand_core::OsRng);
    (to_hex(&sk.to_bytes()), to_hex(&sk.verifying_key().to_bytes()))
}

pub fn sign_catalog(catalog: &[u8], secret_hex: &str, versao: u64, validade_dias: i64) -> Result<CatalogSignature> {
    let sk = SigningKey::from_bytes(&fixed::<32>(from_hex(secret_hex)?, "chave privada")?);
    let sha256 = format!("{:x}", Sha256::digest(catalog));
    let expira_em = (Utc::now() + Duration::days(validade_dias)).to_rfc3339();
    let sig = sk.sign(signed_message(versao, &expira_em, &sha256).as_bytes());
    Ok(CatalogSignature {
        formato: SIG_FORMAT.into(),
        versao,
        expira_em,
        sha256,
        chave_publica: to_hex(&sk.verifying_key().to_bytes()),
        assinatura: to_hex(&sig.to_bytes()),
    })
}

/// Confere chave confiável, assinatura, SHA-256, versão (contra volta para catálogo antigo) e validade.
pub fn verify_catalog(
    catalog: &[u8],
    sig: &CatalogSignature,
    trusted: &[String],
    last_version: Option<u64>,
    now: DateTime<Utc>,
) -> Result<()> {
    if sig.formato != SIG_FORMAT {
        bail!("assinatura em formato desconhecido ('{}')", sig.formato);
    }
    let key = sig.chave_publica.trim().to_lowercase();
    if !trusted.iter().any(|t| t.trim().eq_ignore_ascii_case(&key)) {
        bail!("o catálogo foi assinado por uma chave que esta RRS não reconhece");
    }
    let vk = VerifyingKey::from_bytes(&fixed::<32>(from_hex(&key)?, "chave pública")?)
        .context("chave pública inválida")?;
    let signature = Signature::from_bytes(&fixed::<64>(from_hex(&sig.assinatura)?, "assinatura")?);
    vk.verify_strict(signed_message(sig.versao, &sig.expira_em, &sig.sha256).as_bytes(), &signature)
        .map_err(|_| anyhow::anyhow!("assinatura do catálogo inválida"))?;
    let actual = format!("{:x}", Sha256::digest(catalog));
    if !actual.eq_ignore_ascii_case(sig.sha256.trim()) {
        bail!("o SHA-256 do catálogo baixado não confere com o assinado");
    }
    if let Some(last) = last_version {
        if sig.versao < last {
            bail!(
                "catálogo mais antigo (versão {}) que o já instalado (versão {last}); recusado",
                sig.versao
            );
        }
    }
    let expires = DateTime::parse_from_rfc3339(&sig.expira_em)
        .context("data de validade inválida")?
        .with_timezone(&Utc);
    if expires <= now {
        bail!("catálogo vencido em {}; o servidor pode estar desatualizado ou congelado", sig.expira_em);
    }
    Ok(())
}

/// Chaves aceitas: as embutidas e a salva nas configurações.
pub fn trusted_keys(conn: &Connection) -> Vec<String> {
    let mut keys: Vec<String> = TRUSTED_KEYS.iter().map(|k| k.to_string()).collect();
    if let Some(extra) = db::get_setting(conn, SETTING_EXTRA_KEY) {
        keys.push(extra);
    }
    keys
}

/// Lê o catálogo e a assinatura de uma URL https:// (ou de um caminho local, para testes e pastas sincronizadas).
pub async fn fetch_signed(location: &str) -> Result<(Vec<u8>, CatalogSignature)> {
    let location = location.trim();
    let sig_location = format!("{location}.sig");
    let (catalog, sig_raw) = if location.starts_with("https://") || location.starts_with("http://") {
        let client = reqwest::Client::builder().user_agent("gitstore/0.3").build()?;
        let get = |url: String| {
            let client = client.clone();
            async move {
                let resp = client.get(&url).send().await.with_context(|| format!("falha ao baixar {url}"))?;
                if !resp.status().is_success() {
                    bail!("{url} respondeu HTTP {}", resp.status());
                }
                Ok::<_, anyhow::Error>(resp.bytes().await?.to_vec())
            }
        };
        (get(location.to_string()).await?, get(sig_location).await?)
    } else {
        (
            std::fs::read(location).with_context(|| format!("falha ao ler {location}"))?,
            std::fs::read(&sig_location).with_context(|| format!("falha ao ler {sig_location}"))?,
        )
    };
    let sig: CatalogSignature = serde_json::from_slice(&sig_raw).context("arquivo .sig inválido")?;
    Ok((catalog, sig))
}

/// Confere e, se tudo estiver certo, grava o catálogo novo, importa no banco e guarda a versão aceita.
pub fn apply_signed(conn: &Connection, catalog_path: &Path, catalog: &[u8], sig: &CatalogSignature) -> Result<usize> {
    let last = db::get_setting(conn, SETTING_VERSION).and_then(|v| v.parse::<u64>().ok());
    verify_catalog(catalog, sig, &trusted_keys(conn), last, Utc::now())?;
    let text = std::str::from_utf8(catalog).context("catálogo não está em UTF-8")?;
    let n = db::import_catalog_str(conn, text, "catálogo assinado")?;
    std::fs::write(catalog_path, catalog)
        .with_context(|| format!("falha ao gravar {}", catalog_path.display()))?;
    db::set_setting(conn, SETTING_VERSION, &sig.versao.to_string())?;
    Ok(n)
}

#[cfg(test)]
mod tests {
    use super::*;

    const CATALOG: &[u8] = br#"[{"full_name":"dono/app","description":"","url":"","stars":1,"license":"","topics":"","pushed_at":"","latest_tag":"v1","platforms":"windows","assets":[],"trust_level":"x","trust_notes":["x"]}]"#;

    fn public_key_of(secret_hex: &str) -> Result<String> {
        let sk = SigningKey::from_bytes(&fixed::<32>(from_hex(secret_hex)?, "chave privada")?);
        Ok(to_hex(&sk.verifying_key().to_bytes()))
    }

    fn signed(versao: u64, dias: i64) -> (String, CatalogSignature) {
        let (secret, public) = generate_keypair();
        (public, sign_catalog(CATALOG, &secret, versao, dias).unwrap())
    }

    #[test]
    fn assinatura_valida_e_aceita() {
        let (public, sig) = signed(5, 14);
        verify_catalog(CATALOG, &sig, &[public], Some(4), Utc::now()).unwrap();
    }

    #[test]
    fn catalogo_alterado_e_recusado() {
        let (public, sig) = signed(1, 14);
        let mut tampered = CATALOG.to_vec();
        tampered[10] ^= 1;
        let err = verify_catalog(&tampered, &sig, &[public], None, Utc::now()).unwrap_err();
        assert!(err.to_string().contains("SHA-256"), "{err}");
    }

    #[test]
    fn chave_desconhecida_e_recusada() {
        let (_, sig) = signed(1, 14);
        let (_, outra) = generate_keypair();
        assert!(verify_catalog(CATALOG, &sig, &[outra], None, Utc::now()).is_err());
        assert!(verify_catalog(CATALOG, &sig, &[], None, Utc::now()).is_err());
    }

    #[test]
    fn versao_antiga_e_recusada() {
        let (public, sig) = signed(3, 14);
        let err = verify_catalog(CATALOG, &sig, &[public], Some(7), Utc::now()).unwrap_err();
        assert!(err.to_string().contains("mais antigo"), "{err}");
    }

    #[test]
    fn catalogo_vencido_e_recusado() {
        let (public, sig) = signed(1, 1);
        let depois = Utc::now() + Duration::days(2);
        let err = verify_catalog(CATALOG, &sig, &[public], None, depois).unwrap_err();
        assert!(err.to_string().contains("vencido"), "{err}");
    }

    #[test]
    fn mudar_versao_sem_reassinar_invalida_a_assinatura() {
        let (public, mut sig) = signed(1, 14);
        sig.versao = 99;
        let err = verify_catalog(CATALOG, &sig, &[public], None, Utc::now()).unwrap_err();
        assert!(err.to_string().contains("assinatura"), "{err}");
    }

    #[test]
    fn hex_e_chave_publica_derivada() {
        assert_eq!(from_hex(&to_hex(&[0, 15, 255])).unwrap(), vec![0, 15, 255]);
        assert!(from_hex("zz").is_err());
        let (secret, public) = generate_keypair();
        assert_eq!(public_key_of(&secret).unwrap(), public);
    }

    #[test]
    fn aplicar_grava_importa_e_guarda_a_versao() {
        let dir = std::env::temp_dir().join(format!("rrs-teste-assin-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let catalog_path = dir.join("catalog.json");
        let conn = db::open(":memory:").unwrap();
        let (secret, public) = generate_keypair();
        let sig = sign_catalog(CATALOG, &secret, 2, 14).unwrap();

        // Sem chave confiável: recusa
        assert!(apply_signed(&conn, &catalog_path, CATALOG, &sig).is_err());
        assert!(!catalog_path.exists());

        db::set_setting(&conn, SETTING_EXTRA_KEY, &public).unwrap();
        assert_eq!(apply_signed(&conn, &catalog_path, CATALOG, &sig).unwrap(), 1);
        assert_eq!(std::fs::read(&catalog_path).unwrap(), CATALOG);
        assert_eq!(db::get_setting(&conn, SETTING_VERSION).as_deref(), Some("2"));
        assert!(db::get(&conn, "dono/app").unwrap().is_some());

        // Depois de aceitar a versão 2, a versão 1 é recusada
        let velho = sign_catalog(CATALOG, &secret, 1, 14).unwrap();
        assert!(apply_signed(&conn, &catalog_path, CATALOG, &velho).is_err());
        std::fs::remove_dir_all(&dir).unwrap();
    }
}
