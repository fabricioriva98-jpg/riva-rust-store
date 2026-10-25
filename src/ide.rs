//! Módulo de Integração e Espelhamento com Editores de Codificação (IDEs).
//! Reconhece instalações de Visual Studio Code, Google Antigravity, Cursor e RustRover,
//! permitindo abrir workspaces com 1 clique, instalar extensões oficiais de Rust e espelhar arquivos de projetos.

use anyhow::{bail, Context, Result};
use std::path::{Path, PathBuf};
use std::process::Command;

/// Informações sobre um ambiente de desenvolvimento integrado (IDE) instalado no PC.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IdeInfo {
    pub id: &'static str,
    pub name: &'static str,
    pub badge: &'static str,
    pub exe_path: Option<PathBuf>,
    pub is_installed: bool,
    pub recommended_extensions: &'static [&'static str],
}

/// Detecta os principais editores de código e IDEs instalados no sistema.
pub fn detect_ides() -> Vec<IdeInfo> {
    let mut ides = Vec::new();

    // 1. Visual Studio Code
    let vscode_path = find_vscode();
    let vscode_installed = vscode_path.is_some();
    ides.push(IdeInfo {
        id: "vscode",
        name: "Visual Studio Code",
        badge: "🟦 VS Code",
        exe_path: vscode_path,
        is_installed: vscode_installed,
        recommended_extensions: &[
            "rust-lang.rust-analyzer",
            "vadimcn.vscode-lldb",
            "tamasfe.even-better-toml",
        ],
    });

    // 2. Google Antigravity
    let antigravity_path = find_antigravity();
    let antigravity_installed = antigravity_path.is_some();
    ides.push(IdeInfo {
        id: "antigravity",
        name: "Google Antigravity",
        badge: "⚛ Antigravity",
        exe_path: antigravity_path,
        is_installed: antigravity_installed,
        recommended_extensions: &[
            "rust-analyzer",
            "google-antigravity-sdk",
        ],
    });

    // 3. Cursor (AI Code Editor)
    let cursor_path = find_cursor();
    let cursor_installed = cursor_path.is_some();
    ides.push(IdeInfo {
        id: "cursor",
        name: "Cursor AI",
        badge: "✨ Cursor",
        exe_path: cursor_path,
        is_installed: cursor_installed,
        recommended_extensions: &["rust-lang.rust-analyzer"],
    });

    // 4. RustRover (JetBrains)
    let rustrover_path = find_rustrover();
    let rustrover_installed = rustrover_path.is_some();
    ides.push(IdeInfo {
        id: "rustrover",
        name: "JetBrains RustRover",
        badge: "🦀 RustRover",
        exe_path: rustrover_path,
        is_installed: rustrover_installed,
        recommended_extensions: &[],
    });

    ides
}

/// Localiza o executável do Visual Studio Code (`Code.exe`, nunca o `code.cmd`).
fn find_vscode() -> Option<PathBuf> {
    let standard = ["LOCALAPPDATA", "ProgramFiles", "ProgramFiles(x86)"].into_iter().filter_map(|var| {
        let base = PathBuf::from(std::env::var_os(var)?);
        let sub = if var == "LOCALAPPDATA" { r"Programs\Microsoft VS Code\Code.exe" } else { r"Microsoft VS Code\Code.exe" };
        Some(base.join(sub))
    });
    for p in standard {
        if p.is_file() {
            return Some(p);
        }
    }
    // Instalação em outro lugar: o instalador põe `<pasta>\bin` no PATH
    find_in_path("code").and_then(|cli| code_exe_from_cli(&cli))
}

/// `...\Microsoft VS Code\bin\code.cmd` → `...\Microsoft VS Code\Code.exe`.
fn code_exe_from_cli(cli: &Path) -> Option<PathBuf> {
    let exe = cli.parent()?.parent()?.join("Code.exe");
    exe.is_file().then_some(exe)
}

/// `...\Microsoft VS Code\Code.exe` → `...\Microsoft VS Code\bin\code.cmd` (a linha de comando oficial).
fn code_cli_from_exe(exe: &Path) -> Option<PathBuf> {
    let cli = exe.parent()?.join("bin").join("code.cmd");
    cli.is_file().then_some(cli)
}

/// Localiza o executável do Google Antigravity.
fn find_antigravity() -> Option<PathBuf> {
    if let Ok(local_app_data) = std::env::var("LOCALAPPDATA") {
        let p = PathBuf::from(&local_app_data).join(r"Programs\antigravity\Antigravity.exe");
        if p.is_file() {
            return Some(p);
        }
        // (O `antigravity-updater\pending\Antigravity-x64.exe` é o instalador da próxima versão: abrir uma
        // pasta com ele rodaria a instalação, por isso não entra aqui.)
    }

    find_in_path("antigravity").or_else(|| find_in_path("agy"))
}

/// Localiza o executável do Cursor.
fn find_cursor() -> Option<PathBuf> {
    if let Ok(local_app_data) = std::env::var("LOCALAPPDATA") {
        let p = PathBuf::from(local_app_data).join(r"Programs\cursor\Cursor.exe");
        if p.is_file() {
            return Some(p);
        }
    }
    find_in_path("cursor")
}

/// Localiza o JetBrains RustRover.
fn find_rustrover() -> Option<PathBuf> {
    if let Ok(local_app_data) = std::env::var("LOCALAPPDATA") {
        let p = PathBuf::from(local_app_data).join(r"Programs\RustRover\bin\rustrover64.exe");
        if p.is_file() {
            return Some(p);
        }
    }
    find_in_path("rustrover64").or_else(|| find_in_path("rustrover"))
}

/// Busca um executável no PATH do sistema, sem abrir processo (antes era um PowerShell por busca,
/// cerca de 300 ms cada).
fn find_in_path(cmd: &str) -> Option<PathBuf> {
    let path = std::env::var_os("PATH")?;
    let exts: Vec<String> = std::env::var("PATHEXT")
        .unwrap_or_else(|_| ".COM;.EXE;.BAT;.CMD".into())
        .split(';')
        .filter(|e| !e.is_empty())
        .map(|e| e.to_ascii_lowercase())
        .collect();
    find_in_dirs(cmd, std::env::split_paths(&path), &exts)
}

/// Procura `cmd` + cada extensão nas pastas, na ordem (como o Windows faz com PATH e PATHEXT).
fn find_in_dirs(cmd: &str, dirs: impl IntoIterator<Item = PathBuf>, exts: &[String]) -> Option<PathBuf> {
    for dir in dirs {
        for ext in exts {
            let candidate = dir.join(format!("{cmd}{ext}"));
            if candidate.is_file() {
                return Some(candidate);
            }
        }
    }
    None
}

/// Abre uma pasta de projeto na IDE especificada.
pub fn open_workspace_in_ide(ide_id: &str, folder: &Path) -> Result<String> {
    let ides = detect_ides();
    let Some(ide) = ides.iter().find(|i| i.id == ide_id) else {
        bail!("Editor não suportado: {ide_id}");
    };

    let Some(exe) = &ide.exe_path else {
        bail!("{} não foi encontrado no seu computador.", ide.name);
    };

    let mut cmd = Command::new(exe);
    cmd.arg(folder);

    // No Windows, spawna desacoplado para não prender o processo
    cmd.spawn()
        .with_context(|| format!("Falha ao iniciar {}: {}", ide.name, exe.display()))?;

    Ok(format!("{} aberto com sucesso em: {}", ide.name, folder.display()))
}

/// Abre um arquivo de código específico na IDE.
#[allow(dead_code)]
pub fn open_file_in_ide(ide_id: &str, file: &Path, line: Option<usize>) -> Result<String> {
    let ides = detect_ides();
    let Some(ide) = ides.iter().find(|i| i.id == ide_id) else {
        bail!("Editor não suportado: {ide_id}");
    };

    let Some(exe) = &ide.exe_path else {
        bail!("{} não foi encontrado no seu computador.", ide.name);
    };

    let mut cmd = Command::new(exe);
    if let Some(l) = line {
        if ide_id == "vscode" || ide_id == "cursor" {
            cmd.arg("--goto");
            cmd.arg(format!("{}:{}", file.display(), l));
        } else {
            cmd.arg(file);
        }
    } else {
        cmd.arg(file);
    }

    cmd.spawn()
        .with_context(|| format!("Falha ao abrir arquivo no {}: {}", ide.name, file.display()))?;

    Ok(format!("Arquivo aberto no {}: {}", ide.name, file.display()))
}

/// Id de extensão do VS Code (`editor.nome`): só letras, números e hífens. O `code.cmd` passa
/// pelo interpretador de lotes do Windows, então nada fora disso pode chegar a ele.
pub fn is_valid_extension_id(id: &str) -> bool {
    let mut parts = id.split('.');
    let ok = |p: Option<&str>| p.is_some_and(|p| !p.is_empty() && p.chars().all(|c| c.is_ascii_alphanumeric() || c == '-'));
    ok(parts.next()) && ok(parts.next()) && parts.next().is_none()
}

/// Instala uma extensão no VS Code pela linha de comando oficial (`bin\code.cmd`) da instalação detectada.
/// Antes chamava `code`, que o Windows não acha sem a extensão `.cmd` (o botão falhava sempre).
pub fn install_vscode_extension(ext_id: &str) -> Result<String> {
    if !is_valid_extension_id(ext_id) {
        bail!("Id de extensão inválido: {ext_id}");
    }
    let cli = find_vscode()
        .and_then(|exe| code_cli_from_exe(&exe))
        .or_else(|| find_in_path("code"))
        .context("VS Code não encontrado (nem o comando `code` no PATH).")?;
    let mut cmd = Command::new(&cli);
    cmd.args(["--install-extension", ext_id]);
    let limits = crate::subprocess::Limits { timeout: std::time::Duration::from_secs(5 * 60), max_output: 64 * 1024 };
    let out = crate::subprocess::run_limited(cmd, limits, None)
        .with_context(|| format!("Não foi possível chamar {}", cli.display()))?;
    if out.success() {
        Ok(format!("Extensão '{ext_id}' instalada no VS Code."))
    } else {
        let why = out.notice(limits).unwrap_or_else(|| format!("{} {}", out.stdout.trim(), out.stderr.trim()));
        bail!("Falha ao instalar a extensão no VS Code: {}", why.trim())
    }
}

/// Estrutura de arquivos do workspace espelhado.
#[derive(Debug, Clone, PartialEq, Eq)]
#[allow(dead_code)]
pub struct ProjectFileInfo {
    pub rel_path: String,
    pub is_dir: bool,
    pub size_bytes: u64,
}

/// Lista os arquivos de código Rust e de configuração de um workspace.
#[allow(dead_code)]
pub fn list_project_files(dir: &Path) -> Vec<ProjectFileInfo> {
    let mut files = Vec::new();
    collect_files_recursive(dir, dir, &mut files, 0);
    files
}

#[allow(dead_code)]
fn collect_files_recursive(root: &Path, current: &Path, list: &mut Vec<ProjectFileInfo>, depth: usize) {
    if depth > 4 {
        return;
    }
    let Ok(entries) = std::fs::read_dir(current) else { return };
    for e in entries.flatten() {
        let p = e.path();
        let name = e.file_name().to_string_lossy().to_string();

        // Ignora pastas target, .git e caches
        if name == "target" || name == ".git" || name.starts_with('.') {
            continue;
        }

        let rel = p.strip_prefix(root).unwrap_or(&p).to_string_lossy().replace('\\', "/");
        if p.is_dir() {
            list.push(ProjectFileInfo {
                rel_path: rel,
                is_dir: true,
                size_bytes: 0,
            });
            collect_files_recursive(root, &p, list, depth + 1);
        } else if p.is_file() {
            let meta = e.metadata().ok();
            let size = meta.map(|m| m.len()).unwrap_or(0);
            list.push(ProjectFileInfo {
                rel_path: rel,
                is_dir: false,
                size_bytes: size,
            });
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn detecta_ides_retorna_lista_com_vscode_e_antigravity() {
        let ides = detect_ides();
        assert!(ides.iter().any(|i| i.id == "vscode"), "deve incluir VS Code");
        assert!(ides.iter().any(|i| i.id == "antigravity"), "deve incluir Antigravity");
        assert!(ides.iter().any(|i| i.id == "cursor"), "deve incluir Cursor");
    }

    #[test]
    fn busca_no_path_segue_ordem_das_pastas_e_extensoes() {
        let base = std::env::temp_dir().join(format!("rrs_test_path_{}", std::process::id()));
        let (a, b) = (base.join("a"), base.join("b"));
        std::fs::create_dir_all(&a).unwrap();
        std::fs::create_dir_all(&b).unwrap();
        std::fs::write(a.join("editor.cmd"), "").unwrap();
        std::fs::write(b.join("editor.exe"), "").unwrap();
        // Arquivo sem extensão (como o script `code` do VS Code) não é executável no Windows
        std::fs::write(a.join("editor"), "").unwrap();
        let exts = vec![".exe".to_string(), ".cmd".to_string()];

        // Primeira pasta com qualquer extensão válida vence, como no Windows
        assert_eq!(find_in_dirs("editor", [a.clone(), b.clone()], &exts), Some(a.join("editor.cmd")));
        assert_eq!(find_in_dirs("editor", [b.clone(), a.clone()], &exts), Some(b.join("editor.exe")));
        assert_eq!(find_in_dirs("nao-existe", [a, b], &exts), None);
        let _ = std::fs::remove_dir_all(&base);
    }

    #[test]
    fn caminhos_do_vs_code_entre_exe_e_linha_de_comando() {
        let base = std::env::temp_dir().join(format!("rrs_test_vscode_{}", std::process::id()));
        std::fs::create_dir_all(base.join("bin")).unwrap();
        std::fs::write(base.join("Code.exe"), "").unwrap();
        std::fs::write(base.join("bin").join("code.cmd"), "").unwrap();
        assert_eq!(code_exe_from_cli(&base.join("bin").join("code.cmd")), Some(base.join("Code.exe")));
        assert_eq!(code_cli_from_exe(&base.join("Code.exe")), Some(base.join("bin").join("code.cmd")));
        let _ = std::fs::remove_dir_all(&base);
    }

    #[test]
    fn id_de_extensao_e_validado() {
        assert!(is_valid_extension_id("rust-lang.rust-analyzer"));
        assert!(is_valid_extension_id("tamasfe.even-better-toml"));
        for ruim in ["rust-analyzer", "a.b.c", "a.b & calc", "a.b\"", ".x", "x."] {
            assert!(!is_valid_extension_id(ruim), "{ruim}");
        }
    }

    #[test]
    fn codigo_distribuido_nao_tem_caminhos_deste_pc() {
        // O .exe vai para outras pessoas: caminhos com nome de usuário e pastas pessoais não podem estar nele
        let fontes = [include_str!("ide.rs"), include_str!("terminal_studio.rs"), include_str!("optimizer.rs")];
        let pessoais = [["E:", "\\Fabricio"].concat(), [":\\Users\\", "fabri"].concat()];
        for src in fontes {
            let code = &src[..src.find("#[cfg(test)]").unwrap_or(src.len())];
            for p in &pessoais {
                assert!(!code.contains(p.as_str()), "caminho pessoal no código: {p}");
            }
        }
    }

    #[test]
    fn deteccao_de_editores_nao_abre_processos_e_e_rapida() {
        // Antes eram até 6 PowerShell por chamada (cerca de 1 s neste PC); a tela chamava a cada quadro
        let src = include_str!("ide.rs");
        let detect = &src[src.find("pub fn detect_ides").unwrap()..src.find("/// Abre uma pasta de projeto").unwrap()];
        assert!(!detect.contains("Command::new"), "a detecção deve só conferir arquivos");
        let start = std::time::Instant::now();
        let _ = detect_ides();
        assert!(start.elapsed() < std::time::Duration::from_millis(500), "{:?}", start.elapsed());
    }

    #[test]
    fn list_project_files_ignora_target_e_git() {
        let temp = std::env::temp_dir().join(format!("rrs_test_ide_{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&temp);

        let src = temp.join("src");
        let target = temp.join("target");
        let _ = std::fs::create_dir_all(&src);
        let _ = std::fs::create_dir_all(&target);

        let _ = std::fs::write(temp.join("Cargo.toml"), "[package]\nname=\"teste\"");
        let _ = std::fs::write(src.join("main.rs"), "fn main() {}");
        let _ = std::fs::write(target.join("ignored.txt"), "ignorar");

        let files = list_project_files(&temp);
        assert!(files.iter().any(|f| f.rel_path == "Cargo.toml"));
        assert!(files.iter().any(|f| f.rel_path == "src/main.rs" || f.rel_path == "src"));
        assert!(!files.iter().any(|f| f.rel_path.starts_with("target")));

        let _ = std::fs::remove_dir_all(&temp);
    }
}
