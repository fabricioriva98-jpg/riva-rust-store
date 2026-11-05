//! Terminal Integrado e Espelhado de Desenvolvimento da RRS.
//! Permite executar comandos Cargo, rustc, rustup, git e utilitários de sistema diretamente no aplicativo
//! de forma assíncrona, com histórico de comandos e botões rápidos de produtividade.
//!
//! Tudo roda por [`crate::subprocess::run_limited`]: tempo máximo, botão Parar e saída limitada.
//! Regra (o teste `terminal_so_recebe_o_que_o_usuario_digita` em `gui.rs` cobra): o terminal só
//! executa o que o usuário digita ou os atalhos fixos; nunca texto do catálogo, do GitHub ou da IA.

use crate::subprocess::{self, Limits};
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::time::Instant;

/// Tipo de saída de linha do terminal.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TerminalLineKind {
    Command,
    Stdout,
    Stderr,
    SystemInfo,
}

/// Resultado da execução de um comando no terminal.
#[derive(Debug, Clone)]
pub struct TerminalOutput {
    #[allow(dead_code)]
    pub command: String,
    pub stdout: String,
    pub stderr: String,
    pub success: bool,
    pub elapsed_ms: u64,
    /// Parado por tempo máximo ou pelo botão Parar, ou saída cortada
    pub notice: Option<String>,
}

impl TerminalOutput {
    fn failed(command: String, stderr: String, elapsed_ms: u64) -> Self {
        Self { command, stdout: String::new(), stderr, success: false, elapsed_ms, notice: None }
    }
}

/// Pasta padrão dos projetos do terminal e das lições exportadas: `Documentos\Projetos Rust`.
/// Antes era a pasta de dados da RRS (onde fica o banco, com as chaves de API) ou, no modo
/// portátil, a própria pasta do programa.
pub fn default_projects_dir() -> PathBuf {
    let documents = std::env::var_os("USERPROFILE")
        .map(|h| PathBuf::from(h).join("Documents"))
        .filter(|p| p.is_dir())
        .unwrap_or_else(std::env::temp_dir);
    documents.join("Projetos Rust")
}

/// Executa um comando no terminal dentro do diretório de trabalho especificado.
pub fn run_terminal_command(working_dir: &Path, raw_cmd: &str, cancel: &AtomicBool) -> TerminalOutput {
    let trimmed = raw_cmd.trim();
    if trimmed.is_empty() {
        return TerminalOutput {
            command: String::new(),
            stdout: String::new(),
            stderr: String::new(),
            success: true,
            elapsed_ms: 0,
            notice: None,
        };
    }
    if let Err(e) = std::fs::create_dir_all(working_dir) {
        return TerminalOutput::failed(trimmed.into(), format!("Pasta de trabalho indisponível: {e}"), 0);
    }

    // Separa os argumentos preservando aspas
    let parts = split_args(trimmed);
    let prog = &parts[0];

    // Ferramentas de desenvolvimento (cargo, rustc, rustup, rustfmt, git, gh) são chamadas diretamente
    let is_direct_tool = ["cargo", "rustc", "rustup", "rustfmt", "git", "gh"].contains(&prog.to_lowercase().as_str());
    let mut cmd = if is_direct_tool {
        let mut c = Command::new(prog);
        c.args(&parts[1..]);
        c
    } else {
        // Demais comandos usam PowerShell nativo (nunca cmd.exe)
        let mut c = Command::new("powershell");
        c.args(["-NoProfile", "-NonInteractive", "-Command", trimmed]);
        c
    };
    cmd.current_dir(working_dir);
    finish(trimmed.to_string(), subprocess::run_limited(cmd, subprocess::TERMINAL, Some(cancel)), subprocess::TERMINAL)
}

fn finish(command: String, res: std::io::Result<subprocess::LimitedOutput>, limits: Limits) -> TerminalOutput {
    match res {
        Ok(out) => TerminalOutput {
            command,
            success: out.success(),
            elapsed_ms: out.elapsed.as_millis() as u64,
            notice: out.notice(limits),
            stdout: out.stdout,
            stderr: out.stderr,
        },
        Err(e) => TerminalOutput::failed(command.clone(), format!("Falha ao iniciar o processo '{command}': {e}"), 0),
    }
}

/// Divide linha de comando em tokens preservando aspas duplas.
fn split_args(cmd: &str) -> Vec<String> {
    let mut args = Vec::new();
    let mut current = String::new();
    let mut in_quotes = false;

    for c in cmd.chars() {
        if c == '"' {
            in_quotes = !in_quotes;
        } else if c.is_whitespace() && !in_quotes {
            if !current.is_empty() {
                args.push(current.clone());
                current.clear();
            }
        } else {
            current.push(c);
        }
    }
    if !current.is_empty() {
        args.push(current);
    }
    if args.is_empty() {
        args.push(cmd.to_string());
    }
    args
}

/// Informações sobre o ambiente Rust instalado no computador.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RustEnvInfo {
    pub rustc_version: Option<String>,
    pub cargo_version: Option<String>,
    pub rustup_version: Option<String>,
    pub clippy_installed: bool,
    pub rustfmt_installed: bool,
    pub rust_analyzer_installed: bool,
    pub active_toolchain: Option<String>,
}

/// Detecta as versões e ferramentas da toolchain Rust instaladas no sistema.
pub fn detect_rust_env() -> RustEnvInfo {
    let rustc_version = run_silent("rustc", &["--version"]);
    let cargo_version = run_silent("cargo", &["--version"]);
    let rustup_version = run_silent("rustup", &["--version"]).and_then(|v| {
        v.lines().next().map(|l| l.to_string())
    });

    let clippy_installed = run_silent("cargo", &["clippy", "--version"]).is_some();
    let rustfmt_installed = run_silent("rustfmt", &["--version"]).is_some();
    let rust_analyzer_installed = run_silent("rust-analyzer", &["--version"]).is_some();

    let active_toolchain = run_silent("rustup", &["show", "active-toolchain"]).and_then(|out| {
        out.split_whitespace().next().map(|s| s.to_string())
    });

    RustEnvInfo {
        rustc_version,
        cargo_version,
        rustup_version,
        clippy_installed,
        rustfmt_installed,
        rust_analyzer_installed,
        active_toolchain,
    }
}

fn run_silent(cmd: &str, args: &[&str]) -> Option<String> {
    let mut c = Command::new(cmd);
    c.args(args);
    subprocess::run_limited(c, subprocess::QUICK, None)
        .ok()
        .filter(|o| o.success())
        .map(|o| o.stdout.trim().to_string())
        .filter(|s| !s.is_empty())
}

/// Cria um novo projeto Cargo (--bin ou --lib) de forma declarativa.
pub fn create_cargo_project(parent_dir: &Path, name: &str, is_bin: bool) -> Result<String, String> {
    std::fs::create_dir_all(parent_dir).map_err(|e| format!("Pasta de trabalho indisponível: {e}"))?;
    let flag = if is_bin { "--bin" } else { "--lib" };
    let mut c = Command::new("cargo");
    c.arg("new").arg(name).arg(flag).current_dir(parent_dir);
    let output = subprocess::run_limited(c, subprocess::QUICK, None)
        .map_err(|e| format!("Falha ao invocar 'cargo new': {}", e))?;

    if output.success() {
        let msg = output.stderr.trim();
        Ok(if msg.is_empty() {
            format!("Projeto '{}' criado com sucesso em {:?}", name, parent_dir.join(name))
        } else {
            msg.to_string()
        })
    } else {
        Err(output.notice(subprocess::QUICK).unwrap_or_else(|| output.stderr.trim().to_string()))
    }
}

/// Abre o terminal nativo do Windows (Windows Terminal 'wt' ou PowerShell) na pasta indicada.
pub fn open_native_terminal(working_dir: &Path) -> Result<(), String> {
    let dir = if working_dir.is_dir() { working_dir } else { Path::new(".") };
    let wt = Command::new("wt")
        .arg("-d")
        .arg(dir)
        .spawn();
    if wt.is_ok() {
        return Ok(());
    }
    Command::new("powershell")
        .current_dir(dir)
        .arg("-NoExit")
        .spawn()
        .map_err(|e| format!("Não foi possível abrir o terminal: {e}"))?;
    Ok(())
}

/// Abre a pasta no Windows Explorer.
pub fn open_in_explorer(working_dir: &Path) -> Result<(), String> {
    let dir = if working_dir.is_dir() { working_dir } else { Path::new(".") };
    Command::new("explorer")
        .arg(dir)
        .spawn()
        .map_err(|e| format!("Não foi possível abrir o Windows Explorer: {e}"))?;
    Ok(())
}

/// Avalia uma expressão Rust ou código instantâneo gerando uma compilação temporária em rustc.
pub fn eval_rust_expression(expr: &str, cancel: &AtomicBool) -> TerminalOutput {
    let start = Instant::now();
    let trimmed = expr.trim();
    let command = format!("eval: {trimmed}");
    if trimmed.is_empty() {
        return TerminalOutput::failed("rust_eval".into(), "Expressão vazia".into(), 0);
    }

    let code = if trimmed.contains("fn main()") {
        trimmed.to_string()
    } else {
        format!(
            "fn main() {{\n    let _resultado = {{\n        {}\n    }};\n    println!(\"{{:?}}\", _resultado);\n}}\n",
            trimmed
        )
    };

    // Contador por processo: duas avaliações ao mesmo tempo nunca usam o mesmo arquivo
    static COUNTER: AtomicU64 = AtomicU64::new(0);
    let n = COUNTER.fetch_add(1, Ordering::Relaxed);
    let temp_dir = std::env::temp_dir();
    let file_id = format!("rrs_eval_{}_{n}", std::process::id());
    let src_path = temp_dir.join(format!("{}.rs", file_id));
    let exe_path = temp_dir.join(format!("{}.exe", file_id));

    if let Err(e) = std::fs::write(&src_path, code) {
        return TerminalOutput::failed(
            command,
            format!("Erro ao criar arquivo de scratchpad temporário: {}", e),
            start.elapsed().as_millis() as u64,
        );
    }

    let mut compile = Command::new("rustc");
    compile.arg("--edition=2021").arg("-O").arg(&src_path).arg("-o").arg(&exe_path);
    let compile_res = subprocess::run_limited(compile, subprocess::COMPILE, Some(cancel));
    let _ = std::fs::remove_file(&src_path);

    let out = match compile_res {
        Ok(out) if out.success() => {
            let run = subprocess::run_limited(Command::new(&exe_path), subprocess::RUN, Some(cancel));
            let mut t = finish(command, run, subprocess::RUN);
            t.elapsed_ms = start.elapsed().as_millis() as u64;
            t
        }
        Ok(out) => TerminalOutput {
            command,
            stdout: String::new(),
            notice: out.notice(subprocess::COMPILE),
            stderr: out.stderr,
            success: false,
            elapsed_ms: start.elapsed().as_millis() as u64,
        },
        Err(e) => TerminalOutput::failed(
            command,
            format!("Compilador rustc não disponível: {}", e),
            start.elapsed().as_millis() as u64,
        ),
    };
    let _ = std::fs::remove_file(&exe_path);
    let _ = std::fs::remove_file(exe_path.with_extension("pdb"));
    out
}

/// Formata um bloco de código Rust usando o formatador oficial `rustfmt`.
/// Executa de forma segura com tempo limite [`subprocess::QUICK`].
pub fn format_rust_code(code: &str) -> Result<String, String> {
    let trimmed = code.trim();
    if trimmed.is_empty() {
        return Ok(code.to_string());
    }

    static FMT_COUNTER: AtomicU64 = AtomicU64::new(0);
    let n = FMT_COUNTER.fetch_add(1, Ordering::Relaxed);
    let temp_dir = std::env::temp_dir();
    let file_id = format!("rrs_fmt_{}_{n}", std::process::id());
    let src_path = temp_dir.join(format!("{}.rs", file_id));

    if let Err(e) = std::fs::write(&src_path, code) {
        return Err(format!("Erro ao criar arquivo temporário para formatação: {e}"));
    }

    let mut cmd = Command::new("rustfmt");
    cmd.arg("--edition=2021").arg(&src_path);
    let res = subprocess::run_limited(cmd, subprocess::QUICK, None);

    let out = match res {
        Ok(output) if output.success() => {
            match std::fs::read_to_string(&src_path) {
                Ok(formatted) => Ok(formatted),
                Err(e) => Err(format!("Erro ao ler código formatado: {e}")),
            }
        }
        Ok(output) => {
            let err = output.stderr.trim();
            if err.is_empty() {
                Err("Não foi possível formatar o código (verifique se há erros de sintaxe)".to_string())
            } else {
                Err(err.to_string())
            }
        }
        Err(e) => Err(format!("rustfmt não encontrado ou indisponível: {e}. Instale com 'rustup component add rustfmt'")),
    };

    let _ = std::fs::remove_file(&src_path);
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn split_args_preserva_aspas() {
        let tokens = split_args(r#"cargo add serde --features "derive json""#);
        assert_eq!(tokens, vec!["cargo", "add", "serde", "--features", "derive json"]);
    }

    #[test]
    fn execucao_de_comando_simples_funciona() {
        let temp = std::env::temp_dir();
        let res = run_terminal_command(&temp, "cargo --version", &AtomicBool::new(false));
        if res.stderr.contains("Falha ao iniciar o processo") {
            // Em ambiente sem cargo no PATH dos testes, ignora
            return;
        }
        assert!(res.success);
        assert!(res.stdout.contains("cargo") || res.stdout.contains("1."));
    }

    #[test]
    fn deteccao_de_ambiente_rust_funciona() {
        let env = detect_rust_env();
        assert!(env.rustc_version.is_some() || env.cargo_version.is_some());
    }

    #[test]
    fn avaliacao_de_expressao_rust_funciona() {
        let res = eval_rust_expression("10 + 32", &AtomicBool::new(false));
        if res.stderr.contains("Compilador rustc não disponível") {
            return;
        }
        assert!(res.success);
        assert!(res.stdout.contains("42"));
    }

    #[test]
    fn avaliacao_com_laco_infinito_para_sozinha() {
        let res = eval_rust_expression("fn main() { loop { std::hint::spin_loop(); } }", &AtomicBool::new(false));
        if res.stderr.contains("Compilador rustc não disponível") {
            return;
        }
        assert!(!res.success);
        assert!(res.notice.as_deref().is_some_and(|n| n.contains("tempo máximo")), "{res:?}");
        assert!(res.elapsed_ms < 40_000, "{}", res.elapsed_ms);
    }

    #[test]
    fn pasta_padrao_fica_fora_da_pasta_da_rrs() {
        let dir = default_projects_dir();
        assert!(dir.ends_with("Projetos Rust"), "{dir:?}");
        let exe_dir = std::env::current_exe().unwrap().parent().unwrap().to_path_buf();
        assert!(!dir.starts_with(&exe_dir));
    }

    #[test]
    fn formatacao_de_codigo_com_rustfmt_funciona() {
        let unformatted = "fn   main(){\nprintln!(\"oi\");\n}\n";
        match format_rust_code(unformatted) {
            Ok(formatted) => {
                assert!(formatted.contains("fn main() {\n    println!(\"oi\");\n}"));
            }
            Err(e) if e.contains("rustfmt não encontrado") => {
                // Em ambiente sem rustfmt, teste passa suavemente
            }
            Err(e) => panic!("Erro inesperado na formatação: {e}"),
        }
    }
}
