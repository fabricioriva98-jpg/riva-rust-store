//! Execução de processos com tempo máximo, botão Parar e limite de saída.
//! Usado pelo terminal integrado, pelo avaliador e pelas lições: código do usuário pode ter
//! `loop {}` e comando pode nunca terminar (`cargo run` de um servidor), e nada disso pode
//! deixar um processo preso para sempre nem encher a memória com saída.

use std::io::Read;
use std::process::{Child, Command, ExitStatus, Stdio};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

/// Tempo máximo e tamanho máximo guardado de cada saída (stdout e stderr, separadamente).
#[derive(Debug, Clone, Copy)]
pub struct Limits {
    pub timeout: Duration,
    pub max_output: usize,
}

/// Terminal integrado: builds grandes passam de 10 min; o botão Parar cobre o resto.
pub const TERMINAL: Limits = Limits { timeout: Duration::from_secs(30 * 60), max_output: 1024 * 1024 };
/// Compilação de um arquivo com o `rustc` (lições e avaliador).
pub const COMPILE: Limits = Limits { timeout: Duration::from_secs(60), max_output: 256 * 1024 };
/// Execução de um programa de lição ou do avaliador.
pub const RUN: Limits = Limits { timeout: Duration::from_secs(10), max_output: 256 * 1024 };
/// Consultas rápidas (`--version`, `cargo new`).
pub const QUICK: Limits = Limits { timeout: Duration::from_secs(20), max_output: 64 * 1024 };

/// Resultado de [`run_limited`].
#[derive(Debug)]
pub struct LimitedOutput {
    pub stdout: String,
    pub stderr: String,
    pub status: Option<ExitStatus>,
    pub timed_out: bool,
    pub cancelled: bool,
    pub truncated: bool,
    pub elapsed: Duration,
}

impl LimitedOutput {
    pub fn success(&self) -> bool {
        !self.timed_out && !self.cancelled && self.status.is_some_and(|s| s.success())
    }

    /// Aviso para mostrar ao usuário quando o processo não terminou sozinho ou a saída foi cortada.
    pub fn notice(&self, limits: Limits) -> Option<String> {
        let mut parts = Vec::new();
        if self.timed_out {
            parts.push(format!("parado: passou do tempo máximo de {}", human(limits.timeout)));
        }
        if self.cancelled {
            parts.push("parado pelo botão Parar".to_string());
        }
        if self.truncated {
            parts.push(format!("saída cortada em {} KB", limits.max_output / 1024));
        }
        (!parts.is_empty()).then(|| parts.join("; "))
    }
}

fn human(d: Duration) -> String {
    let s = d.as_secs();
    if s >= 60 {
        format!("{} min", s / 60)
    } else {
        format!("{s} s")
    }
}

/// Roda `cmd` sem entrada (stdin vazio), guardando até `limits.max_output` bytes de cada saída.
/// Para o processo e os filhos dele quando passa de `limits.timeout` ou quando `cancel` vira `true`.
pub fn run_limited(mut cmd: Command, limits: Limits, cancel: Option<&AtomicBool>) -> std::io::Result<LimitedOutput> {
    let start = Instant::now();
    cmd.stdin(Stdio::null()).stdout(Stdio::piped()).stderr(Stdio::piped());
    let mut child = cmd.spawn()?;

    let (done_tx, done_rx) = mpsc::channel::<()>();
    let out_buf = Arc::new(Mutex::new((Vec::new(), false)));
    let err_buf = Arc::new(Mutex::new((Vec::new(), false)));
    for (pipe, buf) in [
        (child.stdout.take().map(|p| Box::new(p) as Box<dyn Read + Send>), Arc::clone(&out_buf)),
        (child.stderr.take().map(|p| Box::new(p) as Box<dyn Read + Send>), Arc::clone(&err_buf)),
    ] {
        let tx = done_tx.clone();
        std::thread::spawn(move || {
            if let Some(p) = pipe {
                read_capped_into(p, limits.max_output, &buf);
            }
            let _ = tx.send(());
        });
    }
    drop(done_tx);

    let mut timed_out = false;
    let mut cancelled = false;
    let status = loop {
        if let Some(st) = child.try_wait()? {
            break Some(st);
        }
        if cancel.is_some_and(|c| c.load(Ordering::Relaxed)) {
            cancelled = true;
        } else if start.elapsed() >= limits.timeout {
            timed_out = true;
        }
        if cancelled || timed_out {
            kill_tree(&mut child);
            break child.wait().ok();
        }
        std::thread::sleep(Duration::from_millis(30));
    };

    // Espera as leituras terminarem; se um neto ainda segurar a saída aberta, não trava aqui
    for _ in 0..2 {
        if done_rx.recv_timeout(Duration::from_secs(2)).is_err() {
            break;
        }
    }
    let take = |b: &Arc<Mutex<(Vec<u8>, bool)>>| {
        let g = b.lock().unwrap_or_else(|p| p.into_inner());
        (String::from_utf8_lossy(&g.0).to_string(), g.1)
    };
    let (stdout, t1) = take(&out_buf);
    let (stderr, t2) = take(&err_buf);
    Ok(LimitedOutput { stdout, stderr, status, timed_out, cancelled, truncated: t1 || t2, elapsed: start.elapsed() })
}

/// Lê tudo de `r` (para o processo nunca travar com a saída cheia), mas guarda só `max` bytes.
fn read_capped_into(mut r: impl Read, max: usize, buf: &Mutex<(Vec<u8>, bool)>) {
    let mut chunk = [0u8; 8192];
    loop {
        match r.read(&mut chunk) {
            Ok(0) | Err(_) => return,
            Ok(n) => {
                let mut g = buf.lock().unwrap_or_else(|p| p.into_inner());
                let room = max.saturating_sub(g.0.len());
                g.0.extend_from_slice(&chunk[..n.min(room)]);
                if n > room {
                    g.1 = true;
                }
            }
        }
    }
}

/// Encerra o processo e os filhos dele (`cargo run` abre o programa compilado; o PowerShell abre o comando).
fn kill_tree(child: &mut Child) {
    #[cfg(target_os = "windows")]
    {
        let _ = Command::new("taskkill")
            .args(["/PID", &child.id().to_string(), "/T", "/F"])
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status();
    }
    let _ = child.kill();
}

#[cfg(test)]
mod tests {
    use super::*;

    fn powershell(script: &str) -> Command {
        let mut c = Command::new("powershell");
        c.args(["-NoProfile", "-NonInteractive", "-Command", script]);
        c
    }

    #[test]
    fn leitura_guarda_so_o_limite_e_avisa_que_cortou() {
        let buf = Mutex::new((Vec::new(), false));
        read_capped_into(std::io::Cursor::new(vec![b'x'; 100_000]), 1000, &buf);
        let g = buf.lock().unwrap();
        assert_eq!(g.0.len(), 1000);
        assert!(g.1);

        let buf = Mutex::new((Vec::new(), false));
        read_capped_into(std::io::Cursor::new(b"curto".to_vec()), 1000, &buf);
        assert_eq!(buf.lock().unwrap().0, b"curto");
        assert!(!buf.lock().unwrap().1);
    }

    #[test]
    fn processo_que_nao_termina_e_parado_no_tempo_maximo() {
        let limits = Limits { timeout: Duration::from_secs(2), max_output: 1024 };
        let out = run_limited(powershell("Start-Sleep -Seconds 60"), limits, None).unwrap();
        assert!(out.timed_out);
        assert!(!out.success());
        assert!(out.elapsed < Duration::from_secs(15), "{:?}", out.elapsed);
        assert!(out.notice(limits).unwrap().contains("tempo máximo"));
    }

    #[test]
    fn botao_parar_encerra_o_processo() {
        let cancel = Arc::new(AtomicBool::new(false));
        let c2 = Arc::clone(&cancel);
        std::thread::spawn(move || {
            std::thread::sleep(Duration::from_millis(1500));
            c2.store(true, Ordering::Relaxed);
        });
        let out = run_limited(powershell("Start-Sleep -Seconds 60"), TERMINAL, Some(&cancel)).unwrap();
        assert!(out.cancelled);
        assert!(out.elapsed < Duration::from_secs(15), "{:?}", out.elapsed);
    }

    #[test]
    fn comando_normal_devolve_saida_e_sucesso() {
        let out = run_limited(powershell("Write-Output 'ola'; exit 0"), QUICK, None).unwrap();
        assert!(out.success(), "{out:?}");
        assert_eq!(out.stdout.trim(), "ola");
        assert!(out.notice(QUICK).is_none());
    }
}
