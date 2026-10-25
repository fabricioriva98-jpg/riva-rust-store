//! Terminal Integrado e Espelhado de Desenvolvimento da RRS.
//! Permite executar comandos Cargo, rustc, rustup, git e utilitários de sistema diretamente no aplicativo
//! de forma assíncrona, com histórico de comandos e botões rápidos de produtividade.

use std::path::Path;
use std::process::{Command, Stdio};
use std::time::Instant;

/// Tipo de saída de linha do terminal.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TerminalLineKind {
    Command,
    Stdout,
    Stderr,
    SystemInfo,
}

/// Uma entrada de log no console do terminal integrado.
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct TerminalLine {
    pub kind: TerminalLineKind,
    pub text: String,
    pub timestamp: String,
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
}

/// Executa um comando no terminal dentro do diretório de trabalho especificado.
pub fn run_terminal_command(working_dir: &Path, raw_cmd: &str) -> TerminalOutput {
    let start = Instant::now();
    let trimmed = raw_cmd.trim();

    if trimmed.is_empty() {
        return TerminalOutput {
            command: String::new(),
            stdout: String::new(),
            stderr: String::new(),
            success: true,
            elapsed_ms: 0,
        };
    }

    // Separa os argumentos preservando aspas
    let parts = split_args(trimmed);
    let prog = &parts[0];
    let args = &parts[1..];

    // Se for ferramenta direta de desenvolvimento (cargo, rustc, rustup, git), chama diretamente
    let is_direct_tool = ["cargo", "rustc", "rustup", "git", "gh"].contains(&prog.to_lowercase().as_str());

    let output_res = if is_direct_tool {
        let mut cmd = Command::new(prog);
        cmd.args(args);
        cmd.current_dir(working_dir);
        cmd.stdout(Stdio::piped());
        cmd.stderr(Stdio::piped());
        cmd.output()
    } else {
        // Demais comandos usam PowerShell nativo (nunca cmd.exe)
        let mut cmd = Command::new("powershell");
        cmd.args(["-NoProfile", "-NonInteractive", "-Command", trimmed]);
        cmd.current_dir(working_dir);
        cmd.stdout(Stdio::piped());
        cmd.stderr(Stdio::piped());
        cmd.output()
    };

    let elapsed_ms = start.elapsed().as_millis() as u64;

    match output_res {
        Ok(out) => {
            let stdout = String::from_utf8_lossy(&out.stdout).to_string();
            let stderr = String::from_utf8_lossy(&out.stderr).to_string();
            TerminalOutput {
                command: trimmed.to_string(),
                stdout,
                stderr,
                success: out.status.success(),
                elapsed_ms,
            }
        }
        Err(e) => TerminalOutput {
            command: trimmed.to_string(),
            stdout: String::new(),
            stderr: format!("Falha ao iniciar o processo '{}': {}", prog, e),
            success: false,
            elapsed_ms,
        },
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
    let rust_analyzer_installed = run_silent("rust-analyzer", &["--version"]).is_some();

    let active_toolchain = run_silent("rustup", &["show", "active-toolchain"]).and_then(|out| {
        out.split_whitespace().next().map(|s| s.to_string())
    });

    RustEnvInfo {
        rustc_version,
        cargo_version,
        rustup_version,
        clippy_installed,
        rust_analyzer_installed,
        active_toolchain,
    }
}

fn run_silent(cmd: &str, args: &[&str]) -> Option<String> {
    Command::new(cmd)
        .args(args)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output()
        .ok()
        .filter(|o| o.status.success())
        .map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string())
        .filter(|s| !s.is_empty())
}

/// Cria um novo projeto Cargo (--bin ou --lib) de forma declarativa.
pub fn create_cargo_project(parent_dir: &Path, name: &str, is_bin: bool) -> Result<String, String> {
    let flag = if is_bin { "--bin" } else { "--lib" };
    let output = Command::new("cargo")
        .arg("new")
        .arg(name)
        .arg(flag)
        .current_dir(parent_dir)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output()
        .map_err(|e| format!("Falha ao invocar 'cargo new': {}", e))?;

    if output.status.success() {
        let msg = String::from_utf8_lossy(&output.stderr).to_string();
        Ok(if msg.trim().is_empty() {
            format!("Projeto '{}' criado com sucesso em {:?}", name, parent_dir.join(name))
        } else {
            msg.trim().to_string()
        })
    } else {
        let err = String::from_utf8_lossy(&output.stderr).to_string();
        Err(err.trim().to_string())
    }
}

/// Avalia uma expressão Rust ou código instantâneo gerando uma compilação temporária em rustc.
pub fn eval_rust_expression(expr: &str) -> TerminalOutput {
    let start = Instant::now();
    let trimmed = expr.trim();
    if trimmed.is_empty() {
        return TerminalOutput {
            command: "rust_eval".to_string(),
            stdout: String::new(),
            stderr: "Expressão vazia".to_string(),
            success: false,
            elapsed_ms: 0,
        };
    }

    let code = if trimmed.contains("fn main()") {
        trimmed.to_string()
    } else {
        format!(
            "fn main() {{\n    let _resultado = {{\n        {}\n    }};\n    println!(\"{{:?}}\", _resultado);\n}}\n",
            trimmed
        )
    };

    let temp_dir = std::env::temp_dir();
    let file_id = format!("rrs_eval_{}_{}", std::process::id(), Instant::now().elapsed().as_nanos());
    let src_path = temp_dir.join(format!("{}.rs", file_id));
    let exe_path = temp_dir.join(format!("{}.exe", file_id));

    if let Err(e) = std::fs::write(&src_path, code) {
        return TerminalOutput {
            command: format!("eval: {}", trimmed),
            stdout: String::new(),
            stderr: format!("Erro ao criar arquivo de scratchpad temporário: {}", e),
            success: false,
            elapsed_ms: start.elapsed().as_millis() as u64,
        };
    }

    // Compila com rustc
    let compile_res = Command::new("rustc")
        .arg("--edition=2021")
        .arg("-O")
        .arg(&src_path)
        .arg("-o")
        .arg(&exe_path)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output();

    let _ = std::fs::remove_file(&src_path);

    match compile_res {
        Ok(out) if out.status.success() => {
            // Executa o binário compilado
            let run_res = Command::new(&exe_path)
                .stdout(Stdio::piped())
                .stderr(Stdio::piped())
                .output();
            let _ = std::fs::remove_file(&exe_path);
            let elapsed_ms = start.elapsed().as_millis() as u64;

            match run_res {
                Ok(run_out) => TerminalOutput {
                    command: format!("eval: {}", trimmed),
                    stdout: String::from_utf8_lossy(&run_out.stdout).to_string(),
                    stderr: String::from_utf8_lossy(&run_out.stderr).to_string(),
                    success: run_out.status.success(),
                    elapsed_ms,
                },
                Err(e) => TerminalOutput {
                    command: format!("eval: {}", trimmed),
                    stdout: String::new(),
                    stderr: format!("Erro na execução do binário de teste: {}", e),
                    success: false,
                    elapsed_ms,
                },
            }
        }
        Ok(out) => {
            let _ = std::fs::remove_file(&exe_path);
            let elapsed_ms = start.elapsed().as_millis() as u64;
            TerminalOutput {
                command: format!("eval: {}", trimmed),
                stdout: String::new(),
                stderr: String::from_utf8_lossy(&out.stderr).to_string(),
                success: false,
                elapsed_ms,
            }
        }
        Err(e) => {
            let elapsed_ms = start.elapsed().as_millis() as u64;
            TerminalOutput {
                command: format!("eval: {}", trimmed),
                stdout: String::new(),
                stderr: format!("Compilador rustc não disponível: {}", e),
                success: false,
                elapsed_ms,
            }
        }
    }
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
        let res = run_terminal_command(&temp, "cargo --version");
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
        let res = eval_rust_expression("10 + 32");
        if res.stderr.contains("Compilador rustc não disponível") {
            return;
        }
        assert!(res.success);
        assert!(res.stdout.contains("42"));
    }
}
