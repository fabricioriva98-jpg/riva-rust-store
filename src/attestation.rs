//! Conferência criptográfica do atestado de build do GitHub pelo GitHub CLI (`gh attestation verify`).
//! O `gh` confere a assinatura Sigstore, o certificado e a identidade do repositório; a RRS só chama o
//! comando e mostra o resultado. Sem o `gh` instalado e logado, a tela explica como instalar.

use anyhow::{bail, Context, Result};
use std::path::Path;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum GhVerify {
    /// Assinatura e origem conferidas pelo `gh`.
    Verified,
    /// O `gh` rodou e recusou (arquivo diferente do atestado, outro repositório...).
    Failed(String),
    /// Não deu para conferir (gh ausente, sem login, sem rede).
    Unavailable(String),
}

pub fn gh_installed() -> bool {
    std::process::Command::new("gh")
        .arg("--version")
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
}

/// Argumentos do `gh attestation verify` para um arquivo baixado de `dono/repo` (só GitHub).
pub fn verify_args(file: &Path, full_name: &str) -> Result<Vec<String>> {
    if crate::forge::is_other_forge(full_name) || full_name.split('/').count() != 2 {
        bail!("atestados de build só existem para repositórios do GitHub");
    }
    Ok(vec![
        "attestation".into(),
        "verify".into(),
        file.to_string_lossy().to_string(),
        "--repo".into(),
        full_name.to_string(),
    ])
}

/// Interpreta o resultado do `gh` (código de saída e mensagens).
pub fn interpret(exit_code: Option<i32>, stdout: &str, stderr: &str) -> GhVerify {
    let text = format!("{stdout}\n{stderr}");
    let lower = text.to_lowercase();
    match exit_code {
        Some(0) => GhVerify::Verified,
        _ if lower.contains("gh auth login") || lower.contains("authentication") || lower.contains("not logged") => {
            GhVerify::Unavailable("o GitHub CLI precisa de login: rode `gh auth login` uma vez".into())
        }
        Some(_) => {
            let first = text
                .lines()
                .map(str::trim)
                .find(|l| !l.is_empty())
                .unwrap_or("o gh recusou o atestado")
                .to_string();
            GhVerify::Failed(first)
        }
        None => GhVerify::Unavailable("o gh foi interrompido".into()),
    }
}

pub fn verify_with_gh(file: &Path, full_name: &str) -> Result<GhVerify> {
    if !file.is_file() {
        bail!("arquivo não encontrado: {}", file.display());
    }
    let args = verify_args(file, full_name)?;
    if !gh_installed() {
        return Ok(GhVerify::Unavailable(
            "GitHub CLI não instalado (winget install GitHub.cli e depois `gh auth login`)".into(),
        ));
    }
    let out = std::process::Command::new("gh")
        .args(&args)
        .output()
        .context("não foi possível executar o gh")?;
    Ok(interpret(
        out.status.code(),
        &String::from_utf8_lossy(&out.stdout),
        &String::from_utf8_lossy(&out.stderr),
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn monta_os_argumentos_so_para_github() {
        let args = verify_args(Path::new("C:\\RRS\\downloads\\cli_cli\\gh.msi"), "cli/cli").unwrap();
        assert_eq!(args[0..2], ["attestation", "verify"]);
        assert_eq!(args[3..], ["--repo", "cli/cli"]);
        assert!(verify_args(Path::new("x"), "codeberg.org/a/b").is_err());
        assert!(verify_args(Path::new("x"), "sem-barra").is_err());
    }

    #[test]
    fn interpreta_o_resultado_do_gh() {
        assert_eq!(interpret(Some(0), "Verification succeeded!", ""), GhVerify::Verified);
        assert!(matches!(
            interpret(Some(1), "", "To get started with GitHub CLI, please run:  gh auth login"),
            GhVerify::Unavailable(_)
        ));
        match interpret(Some(1), "", "\nError: verifying with issuer \"sigstore.dev\": no matching attestation\n") {
            GhVerify::Failed(msg) => assert!(msg.contains("no matching attestation"), "{msg}"),
            other => panic!("esperava falha, veio {other:?}"),
        }
        assert!(matches!(interpret(None, "", ""), GhVerify::Unavailable(_)));
    }
}
