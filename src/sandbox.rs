//! Abrir um instalador baixado no Windows Sandbox (máquina descartável do Windows Pro/Enterprise/Education).
//! A pasta do download entra só para leitura; ao fechar o Sandbox, tudo que rodou lá dentro some.
//! No Windows Home o Sandbox não existe e o botão fica escondido.

use anyhow::{bail, Context, Result};
use std::path::{Path, PathBuf};

const SANDBOX_DESKTOP: &str = r"C:\Users\WDAGUtilityAccount\Desktop\RRS";

pub fn sandbox_exe() -> PathBuf {
    std::env::var_os("WINDIR")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from(r"C:\Windows"))
        .join("System32")
        .join("WindowsSandbox.exe")
}

pub fn available() -> bool {
    sandbox_exe().is_file()
}

fn xml_escape(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&apos;")
}

/// Configuração `.wsb`: pasta do arquivo mapeada só para leitura, sem rede para a área de trabalho do
/// host, e o arquivo aberto ao iniciar.
pub fn build_wsb(file: &Path) -> Result<String> {
    let folder = file.parent().context("arquivo sem pasta")?;
    let name = file
        .file_name()
        .context("arquivo sem nome")?
        .to_string_lossy()
        .to_string();
    Ok(format!(
        "<Configuration>\n  <MappedFolders>\n    <MappedFolder>\n      <HostFolder>{}</HostFolder>\n      <SandboxFolder>{SANDBOX_DESKTOP}</SandboxFolder>\n      <ReadOnly>true</ReadOnly>\n    </MappedFolder>\n  </MappedFolders>\n  <ClipboardRedirection>Disable</ClipboardRedirection>\n  <PrinterRedirection>Disable</PrinterRedirection>\n  <LogonCommand>\n    <Command>explorer.exe \"{}\"</Command>\n  </LogonCommand>\n</Configuration>\n",
        xml_escape(&folder.to_string_lossy()),
        xml_escape(&format!("{SANDBOX_DESKTOP}\\{name}")),
    ))
}

/// Grava o `.wsb` ao lado dos downloads e abre o Sandbox.
pub fn open_in_sandbox(base_dir: &Path, file: &Path) -> Result<PathBuf> {
    if !available() {
        bail!("Windows Sandbox indisponível (precisa do Windows Pro, Enterprise ou Education com o recurso ativado)");
    }
    if !file.is_file() {
        bail!("arquivo não encontrado: {}", file.display());
    }
    let dir = base_dir.join("sandbox");
    std::fs::create_dir_all(&dir)?;
    let wsb = dir.join("rrs-teste.wsb");
    std::fs::write(&wsb, build_wsb(file)?)?;
    std::process::Command::new(sandbox_exe())
        .arg(&wsb)
        .spawn()
        .context("não foi possível abrir o Windows Sandbox")?;
    Ok(wsb)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn wsb_mapeia_a_pasta_so_para_leitura_e_abre_o_arquivo() {
        let xml = build_wsb(Path::new(r"C:\RRS\downloads\dono_app\Setup App & Cia <1>.exe")).unwrap();
        assert!(xml.contains(r"<HostFolder>C:\RRS\downloads\dono_app</HostFolder>"));
        assert!(xml.contains("<ReadOnly>true</ReadOnly>"));
        assert!(xml.contains("Setup App &amp; Cia &lt;1&gt;.exe"), "nome precisa ser escapado no XML");
        assert!(!xml.contains("Cia <1>"));
        assert!(xml.contains("<ClipboardRedirection>Disable</ClipboardRedirection>"));
    }

    #[test]
    fn sem_sandbox_nao_tenta_abrir() {
        if available() {
            return; // PC com Sandbox: não abre janela em teste
        }
        let err = open_in_sandbox(Path::new("."), Path::new("Cargo.toml")).unwrap_err();
        assert!(err.to_string().contains("indisponível"));
    }
}
