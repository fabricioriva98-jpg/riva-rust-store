//! Ferramentas de terminal em Rust já compiladas (como o cargo-binstall): o nome do crate leva ao
//! repositório no crates.io, e a RRS baixa o binário da release desse repositório, sem compilar.
//! Depois de baixar, "Adicionar ao terminal" copia o .exe para `bin\` da RRS e põe essa pasta no PATH.

use anyhow::{anyhow, bail, Context, Result};
use std::path::{Path, PathBuf};

/// Nomes de crate: letras, números, `-` e `_` (regra do crates.io), até 64 caracteres.
pub fn is_valid_crate_name(name: &str) -> bool {
    !name.is_empty()
        && name.len() <= 64
        && name.chars().next().is_some_and(|c| c.is_ascii_alphabetic())
        && name.chars().all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_')
}

/// Repositório (formato do catálogo) a partir da resposta de `/api/v1/crates/<nome>`.
pub fn repository_from_json(v: &serde_json::Value) -> Option<String> {
    let url = v["crate"]["repository"].as_str()?;
    crate::forge::RepoRef::parse(url).map(|r| r.full_name())
}

pub async fn crate_repository(name: &str) -> Result<String> {
    let name = name.trim();
    if !is_valid_crate_name(name) {
        bail!("nome de crate inválido: {name}");
    }
    // O crates.io pede um User-Agent que identifique o programa
    let client = reqwest::Client::builder()
        .user_agent("gitstore/0.3 (RRS - Riva Rust Store)")
        .build()?;
    let resp = client
        .get(format!("https://crates.io/api/v1/crates/{name}"))
        .send()
        .await
        .context("sem conexão com o crates.io")?;
    if resp.status() == reqwest::StatusCode::NOT_FOUND {
        bail!("crate '{name}' não existe no crates.io");
    }
    if !resp.status().is_success() {
        bail!("crates.io respondeu HTTP {}", resp.status());
    }
    let v: serde_json::Value = resp.json().await?;
    repository_from_json(&v).ok_or_else(|| anyhow!("o crate '{name}' não informa um repositório reconhecido (GitHub, Codeberg ou GitLab)"))
}

/// Destino do executável na pasta `bin\` da RRS.
pub fn bin_target(base_dir: &Path, exe: &Path) -> Result<PathBuf> {
    let name = exe.file_name().context("arquivo sem nome")?;
    if !exe
        .extension()
        .is_some_and(|x| x.eq_ignore_ascii_case("exe"))
    {
        bail!("só executáveis .exe podem ir para o terminal: {}", exe.display());
    }
    Ok(base_dir.join("bin").join(name))
}

/// Copia o executável para `bin\` e garante essa pasta no PATH do usuário.
pub fn add_to_terminal(base_dir: &Path, exe: &Path) -> Result<PathBuf> {
    if !exe.is_file() {
        bail!("executável não encontrado: {}", exe.display());
    }
    let target = bin_target(base_dir, exe)?;
    std::fs::create_dir_all(target.parent().unwrap_or(base_dir))?;
    std::fs::copy(exe, &target).with_context(|| format!("falha ao copiar para {}", target.display()))?;
    crate::installer::add_dir_to_user_path(target.parent().unwrap_or(base_dir))?;
    Ok(target)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn valida_nomes_de_crate() {
        assert!(is_valid_crate_name("ripgrep"));
        assert!(is_valid_crate_name("cargo-binstall"));
        assert!(!is_valid_crate_name("rg; calc"));
        assert!(!is_valid_crate_name("../x"));
        assert!(!is_valid_crate_name("1abc"));
        assert!(!is_valid_crate_name(""));
    }

    #[test]
    fn repositorio_vem_do_crates_io() {
        let v = serde_json::json!({"crate": {"name": "ripgrep", "repository": "https://github.com/BurntSushi/ripgrep"}});
        assert_eq!(repository_from_json(&v).as_deref(), Some("BurntSushi/ripgrep"));
        let cb = serde_json::json!({"crate": {"repository": "https://codeberg.org/dono/ferramenta.git"}});
        assert_eq!(repository_from_json(&cb).as_deref(), Some("codeberg.org/dono/ferramenta"));
        assert!(repository_from_json(&serde_json::json!({"crate": {"repository": null}})).is_none());
        assert!(repository_from_json(&serde_json::json!({"crate": {"repository": "https://exemplo.com/x/y"}})).is_none());
    }

    #[test]
    fn destino_na_pasta_bin() {
        let t = bin_target(Path::new(r"E:\RRS"), Path::new(r"E:\RRS\downloads\BurntSushi_ripgrep\app\rg.exe")).unwrap();
        assert_eq!(t, PathBuf::from(r"E:\RRS\bin\rg.exe"));
        assert!(bin_target(Path::new(r"E:\RRS"), Path::new(r"E:\RRS\x.zip")).is_err());
    }
}
