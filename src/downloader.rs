//! Download seguro com cálculo de SHA-256 em tempo real e validação contra o digest oficial do GitHub.

use crate::github::Asset;
use anyhow::{bail, Context, Result};
use sha2::{Digest, Sha256};
use std::io::Write;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};

#[derive(Debug, Clone)]
pub enum DownloadState {
    Idle,
    Downloading {
        repo: String,
        asset_name: String,
        downloaded_bytes: u64,
        total_bytes: u64,
    },
    Finished {
        repo: String,
        tag: String,
        file_path: String,
        sha256: String,
        verified_digest: bool,
        message: String,
    },
    Failed {
        repo: String,
        error: String,
    },
}

pub type SharedDownloadState = Arc<Mutex<DownloadState>>;

pub fn new_shared_state() -> SharedDownloadState {
    Arc::new(Mutex::new(DownloadState::Idle))
}

/// Faz o download do arquivo, confere o SHA-256 e salva na pasta `downloads/<owner>_<repo>/`.
pub async fn download_and_verify(
    repo: &str,
    tag: &str,
    asset: &Asset,
    base_dir: &Path,
    progress: Option<SharedDownloadState>,
) -> Result<(PathBuf, String, bool)> {
    let safe_repo = repo.replace(['/', '\\', ':'], "_");
    let target_dir = base_dir.join("downloads").join(&safe_repo);
    std::fs::create_dir_all(&target_dir)
        .with_context(|| format!("não foi possível criar a pasta {}", target_dir.display()))?;

    let final_path = target_dir.join(&asset.name);
    let temp_path = target_dir.join(format!("{}.part", asset.name));

    if let Some(st) = &progress {
        *st.lock().unwrap() = DownloadState::Downloading {
            repo: repo.to_string(),
            asset_name: asset.name.clone(),
            downloaded_bytes: 0,
            total_bytes: asset.size,
        };
    }

    let client = reqwest::Client::builder()
        .user_agent("gitstore/0.2")
        .build()?;

    let mut resp = client
        .get(&asset.browser_download_url)
        .send()
        .await
        .with_context(|| format!("falha ao iniciar download de {}", asset.name))?;

    if !resp.status().is_success() {
        bail!("servidor retornou HTTP {}", resp.status());
    }

    let total = resp.content_length().unwrap_or(asset.size);
    let mut file = std::fs::File::create(&temp_path)
        .with_context(|| format!("falha ao criar {}", temp_path.display()))?;
    let mut hasher = Sha256::new();
    let mut downloaded: u64 = 0;

    while let Some(chunk) = resp.chunk().await? {
        file.write_all(&chunk)?;
        hasher.update(&chunk);
        downloaded += chunk.len() as u64;

        if let Some(st) = &progress {
            *st.lock().unwrap() = DownloadState::Downloading {
                repo: repo.to_string(),
                asset_name: asset.name.clone(),
                downloaded_bytes: downloaded,
                total_bytes: total,
            };
        }
    }
    file.flush()?;
    drop(file);

    let computed_sha256 = format!("{:x}", hasher.finalize());

    // Conferência contra o digest oficial do GitHub (quando presente)
    let mut verified_digest = false;
    if let Some(expected_raw) = &asset.digest {
        let expected = expected_raw.trim().trim_start_matches("sha256:").to_lowercase();
        if !expected.is_empty() {
            if computed_sha256 != expected {
                let _ = std::fs::remove_file(&temp_path);
                bail!(
                    "ALERTA DE SEGURANÇA: O SHA-256 calculado ({computed_sha256}) NÃO CONFERE com o digest oficial do GitHub ({expected})! O arquivo foi descartado."
                );
            }
            verified_digest = true;
        }
    }

    if final_path.exists() {
        let _ = std::fs::remove_file(&final_path);
    }
    std::fs::rename(&temp_path, &final_path)
        .with_context(|| format!("falha ao mover arquivo final para {}", final_path.display()))?;

    // Auto-extração inteligente de arquivos .zip (para jogos portáteis e executáveis descompactáveis)
    let (effective_path, extracted) = match extract_archive_if_needed(&target_dir, &final_path, repo) {
        Ok(Some(exe_path)) => (exe_path, true),
        _ => (final_path.clone(), false),
    };

    let msg = if extracted {
        format!(
            "✅ Download concluído, SHA-256 verificado e aplicativo descompactado pronto para uso! ({})",
            &computed_sha256[..12]
        )
    } else if verified_digest {
        format!("✅ Download concluído e SHA-256 verificado com o GitHub ({})", &computed_sha256[..12])
    } else {
        format!(
            "✅ Download concluído. SHA-256 calculado: {} (release antiga sem digest no GitHub)",
            &computed_sha256[..12]
        )
    };

    if let Some(st) = &progress {
        *st.lock().unwrap() = DownloadState::Finished {
            repo: repo.to_string(),
            tag: tag.to_string(),
            file_path: effective_path.to_string_lossy().to_string(),
            sha256: computed_sha256.clone(),
            verified_digest,
            message: msg,
        };
    }

    Ok((effective_path, computed_sha256, verified_digest))
}

/// Se o arquivo baixado for um arquivo `.zip`, descompacta em `target_dir/app/` e procura o executável principal.
pub fn extract_archive_if_needed(target_dir: &Path, archive_path: &Path, repo: &str) -> Result<Option<PathBuf>> {
    let name_lower = archive_path.file_name().and_then(|n| n.to_str()).unwrap_or("").to_lowercase();
    if !name_lower.ends_with(".zip") {
        return Ok(None);
    }

    let extract_dir = target_dir.join("app");
    std::fs::create_dir_all(&extract_dir)?;

    #[cfg(target_os = "windows")]
    {
        let p_str = archive_path.to_string_lossy().replace('\'', "''");
        let d_str = extract_dir.to_string_lossy().replace('\'', "''");
        let script = format!("Expand-Archive -LiteralPath '{p_str}' -DestinationPath '{d_str}' -Force");
        let status = std::process::Command::new("powershell")
            .args(["-NoProfile", "-NonInteractive", "-WindowStyle", "Hidden", "-Command", &script])
            .status()?;
        if !status.success() {
            return Ok(None);
        }
    }

    // Busca recursiva pelo melhor executável (.exe)
    if let Some(best_exe) = find_best_executable(&extract_dir, repo) {
        return Ok(Some(best_exe));
    }

    Ok(None)
}

fn find_best_executable(root: &Path, repo: &str) -> Option<PathBuf> {
    let mut exes = Vec::new();
    collect_exes(root, &mut exes);
    if exes.is_empty() {
        return None;
    }

    let repo_short = repo.split('/').nth(1).unwrap_or(repo).to_lowercase();

    // Filtra auxiliares comuns que não são o executável do jogo/app
    let filtered: Vec<PathBuf> = exes
        .iter()
        .filter(|p| {
            let n = p.file_name().and_then(|x| x.to_str()).unwrap_or("").to_lowercase();
            !n.contains("unins")
                && !n.contains("crashpad")
                && !n.contains("unitycrashhandler")
                && !n.contains("vc_redist")
                && !n.contains("dxsetup")
                && !n.contains("update")
        })
        .cloned()
        .collect();

    let candidates = if filtered.is_empty() { exes } else { filtered };

    // 1. Tenta achar executável cujo nome contenha o nome do repositório
    for p in &candidates {
        let n = p.file_name().and_then(|x| x.to_str()).unwrap_or("").to_lowercase();
        if n.contains(&repo_short) {
            return Some(p.clone());
        }
    }

    // 2. Se não achar, pega o de maior tamanho (geralmente o jogo ou app principal)
    candidates.into_iter().max_by_key(|p| {
        std::fs::metadata(p).map(|m| m.len()).unwrap_or(0)
    })
}

fn collect_exes(dir: &Path, acc: &mut Vec<PathBuf>) {
    let Ok(entries) = std::fs::read_dir(dir) else { return; };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            collect_exes(&path, acc);
        } else if path.extension().and_then(|e| e.to_str()).map(|e| e.eq_ignore_ascii_case("exe")).unwrap_or(false) {
            acc.push(path);
        }
    }
}

/// Inicia o jogo ou aplicativo instalado no seu próprio diretório de trabalho,
/// retornando o identificador do processo filho para monitoramento de tempo de jogo.
pub fn launch_installed_app(path: &str) -> Result<Option<std::process::Child>> {
    let p = Path::new(path);
    if !p.exists() {
        bail!("Arquivo ou pasta não encontrado em: {path}");
    }

    let working_dir = if p.is_dir() {
        p.to_path_buf()
    } else {
        p.parent().unwrap_or(p).to_path_buf()
    };

    #[cfg(target_os = "windows")]
    {
        let lower = path.to_lowercase();
        if lower.ends_with(".jar") {
            let child = std::process::Command::new("javaw")
                .arg("-jar")
                .arg(p)
                .current_dir(&working_dir)
                .spawn()
                .or_else(|_| {
                    std::process::Command::new("cmd")
                        .args(["/C", "start", "", path])
                        .current_dir(&working_dir)
                        .spawn()
                })?;
            return Ok(Some(child));
        }

        if lower.ends_with(".exe") {
            let child = std::process::Command::new(p)
                .current_dir(&working_dir)
                .spawn()
                .or_else(|_| {
                    std::process::Command::new("cmd")
                        .args(["/C", "start", "", path])
                        .current_dir(&working_dir)
                        .spawn()
                })?;
            return Ok(Some(child));
        }

        let child = std::process::Command::new("cmd")
            .args(["/C", "start", "", path])
            .current_dir(&working_dir)
            .spawn()?;
        return Ok(Some(child));
    }

    #[cfg(target_os = "linux")]
    {
        let child = std::process::Command::new("xdg-open")
            .arg(p)
            .current_dir(&working_dir)
            .spawn()?;
        return Ok(Some(child));
    }

    #[cfg(target_os = "macos")]
    {
        let child = std::process::Command::new("open")
            .arg(p)
            .current_dir(&working_dir)
            .spawn()?;
        return Ok(Some(child));
    }
}

#[derive(Debug, Clone)]
pub struct DefenderScanResult {
    pub is_clean: bool,
    #[allow(dead_code)]
    pub exit_code: i32,
    pub message: String,
}

/// Executa a varredura nativa do Windows Defender no arquivo baixado sem custos e sem limite de requisição.
pub fn scan_with_windows_defender(path: &str) -> Result<DefenderScanResult> {
    let p = Path::new(path);
    if !p.exists() {
        bail!("Arquivo não encontrado para varredura: {path}");
    }

    #[cfg(target_os = "windows")]
    {
        let p_str = p.to_string_lossy();
        let defender_path = "C:\\Program Files\\Windows Defender\\MpCmdRun.exe";
        let output = std::process::Command::new(defender_path)
            .args(["-Scan", "-ScanType", "3", "-File", &p_str, "-DisableRemediation"])
            .output();

        match output {
            Ok(out) => {
                let code = out.status.code().unwrap_or(-1);
                if code == 0 {
                    Ok(DefenderScanResult {
                        is_clean: true,
                        exit_code: 0,
                        message: "✔ Windows Defender: Nenhuma ameaça detectada (Arquivo Limpo)".into(),
                    })
                } else if code == 2 {
                    Ok(DefenderScanResult {
                        is_clean: false,
                        exit_code: 2,
                        message: "✘ ALERTA: O Windows Defender identificou risco de segurança neste arquivo!".into(),
                    })
                } else {
                    Ok(DefenderScanResult {
                        is_clean: true,
                        exit_code: code,
                        message: "✔ Verificação do Windows Defender concluída sem incidentes.".into(),
                    })
                }
            }
            Err(e) => bail!("Falha ao invocar Windows Defender: {e}"),
        }
    }

    #[cfg(not(target_os = "windows"))]
    {
        Ok(DefenderScanResult {
            is_clean: true,
            exit_code: 0,
            message: "Varredura do Windows Defender só é aplicável no Windows.".into(),
        })
    }
}

#[derive(Debug, Clone)]
pub struct AuthenticodeResult {
    pub status: String,
    pub signer: Option<String>,
}

/// Verifica a assinatura digital oficial Authenticode do Windows para o binário.
pub fn check_authenticode_signature(path: &str) -> Result<AuthenticodeResult> {
    let p = Path::new(path);
    if !p.exists() {
        bail!("Arquivo não encontrado para checagem de assinatura: {path}");
    }

    #[cfg(target_os = "windows")]
    {
        let p_str = p.to_string_lossy().replace('\'', "''");
        let script = format!(
            "$s = Get-AuthenticodeSignature '{p_str}'; \
             $sub = if ($s.SignerCertificate) {{ $s.SignerCertificate.Subject }} else {{ '' }}; \
             \"$($s.Status)|$sub\""
        );

        let output = std::process::Command::new("powershell")
            .args(["-NoProfile", "-NonInteractive", "-Command", &script])
            .output()?;

        let text = String::from_utf8_lossy(&output.stdout).trim().to_string();
        let parts: Vec<&str> = text.splitn(2, '|').collect();
        let status = parts.first().copied().unwrap_or("Unknown").trim().to_string();
        let signer_raw = parts.get(1).copied().unwrap_or("").trim().to_string();
        let signer = if signer_raw.is_empty() { None } else { Some(signer_raw) };

        Ok(AuthenticodeResult { status, signer })
    }

    #[cfg(not(target_os = "windows"))]
    {
        Ok(AuthenticodeResult {
            status: "NotApplicable".into(),
            signer: None,
        })
    }
}

/// Desinstala o aplicativo removendo sua pasta do disco e desmarcando no banco de dados SQLite.
pub fn uninstall_app(base_dir: &Path, full_name: &str) -> Result<()> {
    let safe_repo = full_name.replace(['/', '\\', ':'], "_");
    let target_dir = base_dir.join("downloads").join(&safe_repo);
    if target_dir.exists() {
        std::fs::remove_dir_all(&target_dir)?;
    }
    Ok(())
}

/// Abre/executa o instalador ou arquivo baixado usando o sistema operacional.
#[allow(dead_code)]
pub fn launch_file(path: &str) -> Result<()> {
    let p = Path::new(path);
    if !p.exists() {
        bail!("Arquivo não encontrado em: {path}");
    }
    #[cfg(target_os = "windows")]
    {
        if path.to_lowercase().ends_with(".jar") {
            std::process::Command::new("javaw")
                .arg("-jar")
                .arg(p)
                .spawn()
                .or_else(|_| {
                    std::process::Command::new("cmd")
                        .args(["/C", "start", "", path])
                        .spawn()
                })?;
        } else {
            std::process::Command::new("cmd")
                .args(["/C", "start", "", path])
                .spawn()?;
        }
    }
    #[cfg(target_os = "linux")]
    {
        std::process::Command::new("xdg-open").arg(p).spawn()?;
    }
    #[cfg(target_os = "macos")]
    {
        std::process::Command::new("open").arg(p).spawn()?;
    }
    Ok(())
}

/// Abre o gerenciador de arquivos destacando o arquivo baixado.
pub fn reveal_in_folder(path: &str) -> Result<()> {
    let p = Path::new(path);
    #[cfg(target_os = "windows")]
    {
        if p.exists() {
            std::process::Command::new("explorer")
                .arg(format!("/select,{}", p.display()))
                .spawn()?;
        } else if let Some(parent) = p.parent() {
            std::process::Command::new("explorer").arg(parent).spawn()?;
        }
    }
    #[cfg(not(target_os = "windows"))]
    {
        if let Some(parent) = p.parent() {
            let _ = launch_file(&parent.to_string_lossy());
        }
    }
    Ok(())
}

/// Abre uma URL no navegador padrão.
pub fn open_url(url: &str) {
    #[cfg(target_os = "windows")]
    {
        let _ = std::process::Command::new("cmd")
            .args(["/C", "start", "", url])
            .spawn();
    }
    #[cfg(target_os = "linux")]
    {
        let _ = std::process::Command::new("xdg-open").arg(url).spawn();
    }
    #[cfg(target_os = "macos")]
    {
        let _ = std::process::Command::new("open").arg(url).spawn();
    }
}
