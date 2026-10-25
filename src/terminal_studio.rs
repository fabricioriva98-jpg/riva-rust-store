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
}
