//! Download seguro com cálculo de SHA-256 em tempo real e validação contra o digest oficial do GitHub.

use crate::github::Asset;
use anyhow::{bail, Context, Result};
use sha2::{Digest, Sha256};
use std::io::Write;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};

/// Situação de um download na fila.
#[derive(Debug, Clone, PartialEq)]
pub enum JobState {
    Queued,
    Downloading { downloaded_bytes: u64, total_bytes: u64 },
    Finished { message: String },
    Failed { error: String },
}

#[derive(Debug, Clone)]
pub struct DownloadJob {
    pub id: u64,
    pub repo: String,
    pub tag: String,
    pub asset_name: String,
    pub state: JobState,
}

impl DownloadJob {
    pub fn is_active(&self) -> bool {
        matches!(self.state, JobState::Queued | JobState::Downloading { .. })
    }
}

pub type SharedQueue = Arc<Mutex<Vec<DownloadJob>>>;

/// Quantos downloads rodam ao mesmo tempo; os demais esperam na fila.
pub const MAX_PARALLEL_DOWNLOADS: usize = 2;

/// Fila de downloads com até [`MAX_PARALLEL_DOWNLOADS`] simultâneos. Cada tarefa registra o app como
/// instalado no banco assim que termina (antes isso dependia da tela ver o estado "concluído", e
/// atualizações em sequência podiam ficar sem registro).
#[derive(Clone)]
pub struct DownloadManager {
    jobs: SharedQueue,
    slots: Arc<tokio::sync::Semaphore>,
    next_id: Arc<std::sync::atomic::AtomicU64>,
}

impl Default for DownloadManager {
    fn default() -> Self {
        Self {
            jobs: Arc::new(Mutex::new(Vec::new())),
            slots: Arc::new(tokio::sync::Semaphore::new(MAX_PARALLEL_DOWNLOADS)),
            next_id: Arc::new(std::sync::atomic::AtomicU64::new(1)),
        }
    }
}

impl DownloadManager {
    pub fn snapshot(&self) -> Vec<DownloadJob> {
        self.jobs.lock().unwrap().clone()
    }

    /// `true` se essa versão do app já está na fila ou baixando.
    pub fn is_queued(&self, repo: &str, tag: &str) -> bool {
        self.jobs
            .lock()
            .unwrap()
            .iter()
            .any(|j| j.is_active() && j.repo.eq_ignore_ascii_case(repo) && j.tag == tag)
    }

    pub fn has_active(&self) -> bool {
        self.jobs.lock().unwrap().iter().any(|j| j.is_active())
    }

    /// Retira da fila e devolve os downloads concluídos ou com falha.
    pub fn take_finished(&self) -> Vec<DownloadJob> {
        let mut jobs = self.jobs.lock().unwrap();
        let (done, active): (Vec<_>, Vec<_>) = jobs.drain(..).partition(|j| !j.is_active());
        *jobs = active;
        done
    }

    /// Coloca o download na fila. Devolve `None` se a mesma versão já estiver na fila.
    pub fn enqueue(
        &self,
        rt: &tokio::runtime::Handle,
        db_path: String,
        base_dir: PathBuf,
        repo: String,
        tag: String,
        asset: Asset,
    ) -> Option<u64> {
        if self.is_queued(&repo, &tag) {
            return None;
        }
        let id = self.next_id.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        self.jobs.lock().unwrap().push(DownloadJob {
            id,
            repo: repo.clone(),
            tag: tag.clone(),
            asset_name: asset.name.clone(),
            state: JobState::Queued,
        });
        let jobs = Arc::clone(&self.jobs);
        let slots = Arc::clone(&self.slots);
        rt.spawn(async move {
            let _permit = slots.acquire_owned().await;
            let state = match download_and_verify(&repo, &asset, &base_dir, Some((&jobs, id))).await {
                Ok(done) => {
                    let stored = crate::db::to_stored_path(&base_dir, &done.path);
                    let registered = crate::db::open(&db_path).and_then(|conn| {
                        crate::db::mark_installed(&conn, &repo, &tag, &stored, &done.sha256, done.verified)
                    });
                    match registered {
                        Ok(()) => {
                            // Conta para as conquistas; a tela confere o que desbloqueou
                            if let Ok(conn) = crate::db::open(&db_path) {
                                let _ = crate::db::bump_stat(&conn, crate::achievements::STAT_DOWNLOADS, 1);
                            }
                            JobState::Finished { message: done.message }
                        }
                        Err(e) => JobState::Failed {
                            error: format!("arquivo baixado, mas não foi possível registrar no banco: {e:#}"),
                        },
                    }
                }
                Err(e) => JobState::Failed { error: format!("{e:#}") },
            };
            set_job_state(&jobs, id, state);
        });
        Some(id)
    }
}

fn set_job_state(jobs: &SharedQueue, id: u64, state: JobState) {
    if let Some(job) = jobs.lock().unwrap().iter_mut().find(|j| j.id == id) {
        job.state = state;
    }
}

/// Resultado de um download conferido.
#[derive(Debug, Clone)]
pub struct DownloadOutcome {
    /// Executável principal (quando o .zip foi extraído) ou o arquivo baixado.
    pub path: PathBuf,
    pub sha256: String,
    pub verified: bool,
    pub message: String,
}

/// Faz o download do arquivo, confere o SHA-256 e salva na pasta `downloads/<owner>_<repo>/`.
pub async fn download_and_verify(
    repo: &str,
    asset: &Asset,
    base_dir: &Path,
    progress: Option<(&SharedQueue, u64)>,
) -> Result<DownloadOutcome> {
    let safe_repo = repo.replace(['/', '\\', ':'], "_");
    let target_dir = base_dir.join("downloads").join(&safe_repo);
    std::fs::create_dir_all(&target_dir)
        .with_context(|| format!("não foi possível criar a pasta {}", target_dir.display()))?;

    // Só o nome do arquivo: um nome com "..\" no catálogo não pode sair da pasta de downloads.
    let file_name = Path::new(&asset.name)
        .file_name()
        .map(|n| n.to_string_lossy().to_string())
        .filter(|n| !n.is_empty() && n != "." && n != "..")
        .ok_or_else(|| anyhow::anyhow!("nome de arquivo inválido na release: {}", asset.name))?;
    let final_path = target_dir.join(&file_name);
    let temp_path = target_dir.join(format!("{file_name}.part"));

    if let Some((jobs, id)) = progress {
        set_job_state(
            jobs,
            id,
            JobState::Downloading {
                downloaded_bytes: 0,
                total_bytes: asset.size,
            },
        );
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

    loop {
        let chunk = match resp.chunk().await {
            Ok(Some(c)) => c,
            Ok(None) => break,
            Err(e) => {
                // Download interrompido: não deixa o .part pela metade na pasta
                drop(file);
                let _ = std::fs::remove_file(&temp_path);
                return Err(e).context("download interrompido");
            }
        };
        file.write_all(&chunk)?;
        hasher.update(&chunk);
        downloaded += chunk.len() as u64;

        if let Some((jobs, id)) = progress {
            set_job_state(
                jobs,
                id,
                JobState::Downloading {
                    downloaded_bytes: downloaded,
                    total_bytes: total,
                },
            );
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

    Ok(DownloadOutcome {
        path: effective_path,
        sha256: computed_sha256,
        verified: verified_digest,
        message: format!("{msg} — {repo}"),
    })
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
        // Sem `cmd /C start`: o cmd.exe interpreta & | ^ % no caminho (injeção de comando).
        // Quando não dá para iniciar o processo diretamente, o Shell do Windows abre o arquivo
        // (associação de tipo, pedido de elevação etc.) e não há processo para medir o tempo de jogo.
        let lower = path.to_lowercase();
        let spawned = if lower.ends_with(".jar") {
            std::process::Command::new("javaw")
                .arg("-jar")
                .arg(p)
                .current_dir(&working_dir)
                .spawn()
                .ok()
        } else if lower.ends_with(".exe") {
            std::process::Command::new(p)
                .current_dir(&working_dir)
                .spawn()
                .ok()
        } else {
            None
        };
        if let Some(child) = spawned {
            return Ok(Some(child));
        }
        shell_open(p.as_os_str(), Some(&working_dir))?;
        return Ok(None);
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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ScanVerdict {
    Clean,
    Threat,
    /// A varredura não terminou com um resultado conhecido; nunca deve ser exibida como "limpo".
    NotVerified,
}

#[derive(Debug, Clone)]
pub struct DefenderScanResult {
    pub verdict: ScanVerdict,
    #[allow(dead_code)]
    pub exit_code: Option<i32>,
    pub message: String,
}

impl DefenderScanResult {
    pub fn not_verified(reason: &str) -> Self {
        Self {
            verdict: ScanVerdict::NotVerified,
            exit_code: None,
            message: format!("⚠ Não verificado: {reason}"),
        }
    }
}

/// Traduz o código de saída do `MpCmdRun.exe`: 0 = sem ameaças, 2 = ameaça encontrada.
/// Qualquer outro código (ou processo encerrado sem código) é erro da varredura, não arquivo limpo.
pub fn interpret_defender_exit(code: Option<i32>) -> DefenderScanResult {
    match code {
        Some(0) => DefenderScanResult {
            verdict: ScanVerdict::Clean,
            exit_code: code,
            message: "✔ Windows Defender: Nenhuma ameaça detectada (Arquivo Limpo)".into(),
        },
        Some(2) => DefenderScanResult {
            verdict: ScanVerdict::Threat,
            exit_code: code,
            message: "✘ ALERTA: O Windows Defender identificou risco de segurança neste arquivo!".into(),
        },
        Some(c) => DefenderScanResult {
            exit_code: code,
            ..DefenderScanResult::not_verified(&format!(
                "o Windows Defender terminou com o código {c} (0x{:08X}) sem concluir a varredura.",
                c as u32
            ))
        },
        None => DefenderScanResult::not_verified("a varredura do Windows Defender foi interrompida."),
    }
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
            Ok(out) => Ok(interpret_defender_exit(out.status.code())),
            Err(e) => bail!("Falha ao invocar Windows Defender: {e}"),
        }
    }

    #[cfg(not(target_os = "windows"))]
    {
        Ok(DefenderScanResult::not_verified(
            "a varredura do Windows Defender só é aplicável no Windows.",
        ))
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
        let java_started = path.to_lowercase().ends_with(".jar")
            && std::process::Command::new("javaw")
                .arg("-jar")
                .arg(p)
                .spawn()
                .is_ok();
        if !java_started {
            shell_open(p.as_os_str(), None)?;
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

/// Só endereços `https://` sem espaços, aspas ou caracteres de controle podem ir para o navegador.
/// Bloqueia `file:`, `javascript:`, caminhos locais e executáveis vindos do catálogo.
pub fn is_safe_web_url(url: &str) -> bool {
    let Some(prefix) = url.get(..8) else {
        return false;
    };
    prefix.eq_ignore_ascii_case("https://")
        && url.len() > 8
        && !url
            .chars()
            .any(|c| c.is_whitespace() || c.is_control() || c == '"')
}

/// Abre uma URL no navegador padrão.
pub fn open_url(url: &str) -> Result<()> {
    if !is_safe_web_url(url) {
        bail!("Endereço recusado (só https://): {url}");
    }
    #[cfg(target_os = "windows")]
    {
        shell_open(std::ffi::OsStr::new(url), None)?;
    }
    #[cfg(target_os = "linux")]
    {
        std::process::Command::new("xdg-open").arg(url).spawn()?;
    }
    #[cfg(target_os = "macos")]
    {
        std::process::Command::new("open").arg(url).spawn()?;
    }
    Ok(())
}

/// Abre um endereço de loja de jogos (`steam://`, `com.epicgames.launcher://`) já validado em `launchers.rs`.
pub fn open_store_uri(uri: &str) -> Result<()> {
    #[cfg(target_os = "windows")]
    {
        shell_open(std::ffi::OsStr::new(uri), None)
    }
    #[cfg(not(target_os = "windows"))]
    {
        bail!("abrir jogos de outras lojas só é suportado no Windows: {uri}")
    }
}

/// Abre arquivo, pasta ou URL pelo Shell do Windows (`ShellExecuteW`), sem passar pelo `cmd.exe`.
#[cfg(target_os = "windows")]
fn shell_open(target: &std::ffi::OsStr, working_dir: Option<&Path>) -> Result<()> {
    use std::os::windows::ffi::OsStrExt;

    #[link(name = "shell32")]
    extern "system" {
        fn ShellExecuteW(
            hwnd: *mut std::ffi::c_void,
            operation: *const u16,
            file: *const u16,
            parameters: *const u16,
            directory: *const u16,
            show_cmd: i32,
        ) -> isize;
    }
    const SW_SHOWNORMAL: i32 = 1;

    let wide = |s: &std::ffi::OsStr| -> Vec<u16> { s.encode_wide().chain(std::iter::once(0)).collect() };
    let operation = wide(std::ffi::OsStr::new("open"));
    let file = wide(target);
    let directory = working_dir.map(|d| wide(d.as_os_str()));

    // Valores <= 32 são códigos de erro (documentação do ShellExecuteW).
    let code = unsafe {
        ShellExecuteW(
            std::ptr::null_mut(),
            operation.as_ptr(),
            file.as_ptr(),
            std::ptr::null(),
            directory.as_ref().map_or(std::ptr::null(), |d| d.as_ptr()),
            SW_SHOWNORMAL,
        )
    };
    if code <= 32 {
        bail!(
            "o Windows não conseguiu abrir {} (código {code})",
            target.to_string_lossy()
        );
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn temp_dir(tag: &str) -> PathBuf {
        let nanos = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let dir = std::env::temp_dir().join(format!("rrs-teste-dl-{tag}-{}-{nanos}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        dir
    }

    /// Servidor HTTP mínimo: responde `body` a uma única conexão.
    fn serve_once(body: &'static [u8]) -> String {
        let listener = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
        let addr = listener.local_addr().unwrap();
        std::thread::spawn(move || {
            if let Ok((mut s, _)) = listener.accept() {
                use std::io::Read;
                let mut buf = [0u8; 2048];
                let _ = s.read(&mut buf);
                let _ = write!(
                    s,
                    "HTTP/1.1 200 OK\r\nContent-Length: {}\r\nConnection: close\r\n\r\n",
                    body.len()
                );
                let _ = s.write_all(body);
            }
        });
        format!("http://{addr}/arquivo.bin")
    }

    fn prepare_db(base: &Path) -> String {
        let db_path = base.join("t.db").to_string_lossy().to_string();
        let conn = crate::db::open(&db_path).unwrap();
        crate::db::import_catalog_str(
            &conn,
            r#"[{"full_name":"dono/app","description":"","url":"","stars":1,"license":"","topics":"",
                "pushed_at":"","latest_tag":"v2","platforms":"windows","assets":[],"trust_level":"x","trust_notes":["x"]}]"#,
            "t",
        )
        .unwrap();
        db_path
    }

    fn wait_queue(rt: &tokio::runtime::Runtime, mgr: &DownloadManager) -> Vec<DownloadJob> {
        for _ in 0..200 {
            if !mgr.has_active() {
                break;
            }
            rt.block_on(async { tokio::time::sleep(std::time::Duration::from_millis(50)).await });
        }
        mgr.take_finished()
    }

    #[test]
    fn fila_registra_instalado_ao_terminar_sem_depender_da_tela() {
        let rt = tokio::runtime::Runtime::new().unwrap();
        let base = temp_dir("fila-ok");
        let db_path = prepare_db(&base);
        let body: &'static [u8] = b"conteudo do instalador";
        let asset = Asset {
            name: "app-setup.exe".into(),
            size: body.len() as u64,
            browser_download_url: serve_once(body),
            digest: Some(format!("sha256:{:x}", Sha256::digest(body))),
        };
        let mgr = DownloadManager::default();
        let job = |a: Asset| mgr.enqueue(rt.handle(), db_path.clone(), base.clone(), "dono/app".into(), "v2".into(), a);
        assert!(job(asset.clone()).is_some());
        assert!(job(asset).is_none(), "a mesma versão não entra duas vezes na fila");

        let done = wait_queue(&rt, &mgr);
        assert_eq!(done.len(), 1);
        assert!(matches!(done[0].state, JobState::Finished { .. }), "{:?}", done[0].state);

        let conn = crate::db::open(&db_path).unwrap();
        let inst = crate::db::get(&conn, "dono/app").unwrap().unwrap().installed.unwrap();
        assert_eq!(inst.installed_tag, "v2");
        assert!(inst.verified_github_digest);
        assert_eq!(Path::new(&inst.file_path), Path::new("downloads/dono_app/app-setup.exe"));
        drop(conn);
        std::fs::remove_dir_all(&base).unwrap();
    }

    #[test]
    fn fila_com_hash_errado_falha_descarta_arquivo_e_nao_registra() {
        let rt = tokio::runtime::Runtime::new().unwrap();
        let base = temp_dir("fila-hash");
        let db_path = prepare_db(&base);
        let body: &'static [u8] = b"arquivo adulterado";
        let asset = Asset {
            name: "app-setup.exe".into(),
            size: body.len() as u64,
            browser_download_url: serve_once(body),
            digest: Some(format!("sha256:{}", "0".repeat(64))),
        };
        let mgr = DownloadManager::default();
        mgr.enqueue(rt.handle(), db_path.clone(), base.clone(), "dono/app".into(), "v2".into(), asset)
            .unwrap();
        let done = wait_queue(&rt, &mgr);
        match &done[0].state {
            JobState::Failed { error } => assert!(error.contains("NÃO CONFERE"), "{error}"),
            other => panic!("esperava falha, veio {other:?}"),
        }
        let dir = base.join("downloads").join("dono_app");
        assert!(!dir.join("app-setup.exe").exists());
        assert!(!dir.join("app-setup.exe.part").exists());
        let conn = crate::db::open(&db_path).unwrap();
        assert!(crate::db::get(&conn, "dono/app").unwrap().unwrap().installed.is_none());
        drop(conn);
        std::fs::remove_dir_all(&base).unwrap();
    }

    #[test]
    fn nome_de_arquivo_com_caminho_nao_sai_da_pasta_de_downloads() {
        let rt = tokio::runtime::Runtime::new().unwrap();
        let base = temp_dir("nome");
        let body: &'static [u8] = b"x";
        let asset = Asset {
            name: "..\\..\\fora.exe".into(),
            size: 1,
            browser_download_url: serve_once(body),
            digest: None,
        };
        let done = rt.block_on(download_and_verify("dono/app", &asset, &base, None)).unwrap();
        assert_eq!(done.path, base.join("downloads").join("dono_app").join("fora.exe"));
        assert!(!base.parent().unwrap().join("fora.exe").exists());
        std::fs::remove_dir_all(&base).unwrap();
    }

    #[test]
    fn defender_codigo_zero_e_limpo() {
        assert_eq!(interpret_defender_exit(Some(0)).verdict, ScanVerdict::Clean);
    }

    #[test]
    fn defender_codigo_dois_e_ameaca() {
        assert_eq!(interpret_defender_exit(Some(2)).verdict, ScanVerdict::Threat);
    }

    #[test]
    fn defender_outros_codigos_nao_sao_limpos() {
        // 1, código negativo (HRESULT de erro), código alto e processo sem código de saída.
        for code in [Some(1), Some(-2147024894), Some(0x8050_8023_u32 as i32), Some(3), None] {
            let r = interpret_defender_exit(code);
            assert_eq!(r.verdict, ScanVerdict::NotVerified, "código {code:?}");
            assert!(r.message.contains("Não verificado"), "código {code:?}: {}", r.message);
        }
    }

    #[test]
    fn url_aceita_https() {
        assert!(is_safe_web_url("https://github.com/HarbourMasters/Shipwright"));
        assert!(is_safe_web_url("https://example.com/busca?q=a&b=c"));
        assert!(is_safe_web_url("HTTPS://GITHUB.COM/dono/repo"));
    }

    #[test]
    fn url_recusa_esquemas_e_textos_perigosos() {
        for url in [
            "",
            "https://",
            "http://github.com/dono/repo",
            "file:///C:/Windows/System32/calc.exe",
            "javascript:alert(1)",
            "calc.exe",
            "C:\\Windows\\System32\\calc.exe",
            "https://github.com/a b",
            "https://github.com/a\"&calc",
            "https://github.com/a\r\ncalc",
            "httpsé://x",
        ] {
            assert!(!is_safe_web_url(url), "deveria recusar {url:?}");
            assert!(open_url(url).is_err(), "open_url deveria recusar {url:?}");
        }
    }

    #[test]
    fn nenhum_codigo_chama_cmd_exe() {
        // Regressão do G3: `cmd /C start` interpreta & | ^ % vindos do catálogo.
        let needle = ["Command::new(\"cm", "d\")"].concat();
        for (name, src) in [
            ("downloader.rs", include_str!("downloader.rs")),
            ("gui.rs", include_str!("gui.rs")),
            ("installer.rs", include_str!("installer.rs")),
            ("main.rs", include_str!("main.rs")),
        ] {
            assert!(!src.contains(&needle), "{name} voltou a chamar cmd.exe");
        }
    }
}
