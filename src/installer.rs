use anyhow::{bail, Context, Result};
use eframe::egui::{self, Color32, CornerRadius, RichText, Stroke};
use std::path::{Path, PathBuf};
use std::process::Command;

/// Marcador gravado na pasta pela instalação; a desinstalação só age em pastas que o tenham
/// (ou o executável principal, em instalações anteriores ao marcador).
pub const INSTALL_MARKER: &str = "rrs-install.json";
const MAIN_EXE: &str = "RRS - Riva Rust Store.exe";

/// Únicos arquivos que a desinstalação apaga. A pasta `downloads/` (apps e jogos baixados) é mantida.
const INSTALLED_FILES: &[&str] = &[
    MAIN_EXE,
    "rrs.exe",
    "catalog.json",
    "gitstore.db",
    "gitstore.db-journal",
    "gitstore.db-wal",
    "gitstore.db-shm",
    INSTALL_MARKER,
];

/// Retorna a pasta padrão de instalação no Windows:
/// `%LOCALAPPDATA%\Programs\RRS-RivaRustStore`
pub fn default_install_dir() -> PathBuf {
    if let Ok(local_app_data) = std::env::var("LOCALAPPDATA") {
        PathBuf::from(local_app_data)
            .join("Programs")
            .join("RRS-RivaRustStore")
    } else if let Ok(user_profile) = std::env::var("USERPROFILE") {
        PathBuf::from(user_profile)
            .join("AppData")
            .join("Local")
            .join("Programs")
            .join("RRS-RivaRustStore")
    } else {
        PathBuf::from("C:\\RRS-RivaRustStore")
    }
}

/// Detecta se o executável atual foi aberto com nome de instalador (ex.: "Instalador - RRS...exe" ou "Setup...exe").
pub fn is_running_as_installer_exe() -> bool {
    let Ok(exe) = std::env::current_exe() else {
        return false;
    };
    let Some(stem) = exe.file_stem().and_then(|s| s.to_str()) else {
        return false;
    };
    let lower = stem.to_lowercase();
    lower.contains("instalador") || lower.contains("installer") || lower.contains("setup")
}

#[derive(Clone, Debug)]
pub struct InstallOptions {
    pub install_dir: PathBuf,
    pub desktop_shortcut: bool,
    pub start_menu_shortcut: bool,
    pub add_to_user_path: bool,
    pub launch_after_install: bool,
}

impl Default for InstallOptions {
    fn default() -> Self {
        Self {
            install_dir: default_install_dir(),
            desktop_shortcut: true,
            start_menu_shortcut: true,
            add_to_user_path: true,
            launch_after_install: true,
        }
    }
}

/// Executa a instalação completa do aplicativo no Windows:
/// 1. Cria a pasta de destino
/// 2. Copia o executável atual como `RRS - Riva Rust Store.exe` e `rrs.exe`
/// 3. Salva o `catalog.json` embutido e inicializa o banco `gitstore.db`
/// 4. Cria atalhos `.lnk` na Área de Trabalho e Menu Iniciar
/// 5. Adiciona ao PATH do usuário (opcional)
pub fn perform_install(opts: &InstallOptions) -> Result<PathBuf> {
    std::fs::create_dir_all(&opts.install_dir).with_context(|| {
        format!(
            "Não foi possível criar a pasta de instalação: {}",
            opts.install_dir.display()
        )
    })?;

    let current_exe = std::env::current_exe().context("Falha ao localizar executável atual")?;
    let target_gui_exe = opts.install_dir.join(MAIN_EXE);
    let target_cli_exe = opts.install_dir.join("rrs.exe");

    // Se não estivermos rodando de dentro do próprio arquivo de destino, copia o binário
    if !same_file_path(&current_exe, &target_gui_exe) {
        std::fs::copy(&current_exe, &target_gui_exe).with_context(|| {
            format!(
                "Falha ao copiar executável para {}",
                target_gui_exe.display()
            )
        })?;
    }
    if !same_file_path(&current_exe, &target_cli_exe) {
        let _ = std::fs::copy(&current_exe, &target_cli_exe);
    }
    write_install_marker(&opts.install_dir)?;

    // Grava o catalog.json atualizado e prepara o banco SQLite na pasta instalada
    let catalog_path = opts.install_dir.join("catalog.json");
    std::fs::write(&catalog_path, crate::db::EMBEDDED_CATALOG)
        .context("Falha ao gravar catalog.json na pasta de instalação")?;

    let db_path = opts.install_dir.join("gitstore.db");
    if let Ok(conn) = crate::db::open(&db_path.to_string_lossy()) {
        let _ = crate::db::import_catalog(&conn, &catalog_path);
    }

    // Cria atalhos no Windows (.lnk) e adiciona ao PATH via PowerShell
    create_windows_shortcuts_and_path(
        &target_gui_exe,
        &opts.install_dir,
        opts.desktop_shortcut,
        opts.start_menu_shortcut,
        opts.add_to_user_path,
    )?;

    Ok(target_gui_exe)
}

fn write_install_marker(install_dir: &Path) -> Result<()> {
    let marker = serde_json::json!({
        "app": "RRS - Riva Rust Store",
        "version": env!("CARGO_PKG_VERSION"),
        "installed_at": chrono::Utc::now().to_rfc3339(),
    });
    std::fs::write(
        install_dir.join(INSTALL_MARKER),
        serde_json::to_string_pretty(&marker)?,
    )
    .context("Falha ao gravar o marcador da instalação")
}

/// `true` se a pasta é uma instalação da RRS (tem o marcador ou o executável principal).
pub fn is_rrs_install_dir(dir: &Path) -> bool {
    dir.join(INSTALL_MARKER).is_file() || dir.join(MAIN_EXE).is_file()
}

#[derive(Debug, Default)]
pub struct UninstallReport {
    pub removed: Vec<PathBuf>,
    pub failed: Vec<(PathBuf, String)>,
    /// A pasta ficou porque ainda tem outros arquivos (ex.: `downloads/`).
    pub dir_kept: bool,
}

/// Apaga só os arquivos conhecidos da instalação e, se a pasta ficar vazia, a própria pasta.
/// Recusa qualquer pasta que não seja uma instalação da RRS.
pub fn remove_installed_files(install_dir: &Path) -> Result<UninstallReport> {
    if !is_rrs_install_dir(install_dir) {
        bail!(
            "{} não é uma instalação da RRS (falta {INSTALL_MARKER} ou {MAIN_EXE}). Nada foi removido.",
            install_dir.display()
        );
    }
    let mut report = UninstallReport::default();
    for name in INSTALLED_FILES {
        let path = install_dir.join(name);
        if !path.is_file() {
            continue;
        }
        match std::fs::remove_file(&path) {
            Ok(()) => report.removed.push(path),
            Err(e) => report.failed.push((path, e.to_string())),
        }
    }
    // remove_dir só apaga pasta vazia: qualquer arquivo do usuário mantém a pasta.
    report.dir_kept = std::fs::remove_dir(install_dir).is_err();
    Ok(report)
}

pub fn perform_uninstall(install_dir: &Path) -> Result<UninstallReport> {
    if !is_rrs_install_dir(install_dir) {
        bail!(
            "{} não é uma instalação da RRS. Nada foi removido.",
            install_dir.display()
        );
    }
    remove_windows_shortcuts()?;
    remove_installed_files(install_dir)
}

fn same_file_path(a: &Path, b: &Path) -> bool {
    match (std::fs::canonicalize(a), std::fs::canonicalize(b)) {
        (Ok(ca), Ok(cb)) => ca == cb,
        _ => a == b,
    }
}

fn create_windows_shortcuts_and_path(
    target_exe: &Path,
    working_dir: &Path,
    desktop: bool,
    start_menu: bool,
    add_path: bool,
) -> Result<()> {
    #[cfg(target_os = "windows")]
    {
        let exe_str = target_exe.to_string_lossy().replace('\'', "''");
        let dir_str = working_dir.to_string_lossy().replace('\'', "''");

        let mut script = String::from("$ws = New-Object -ComObject WScript.Shell;\n");

        if desktop {
            script.push_str(&format!(
                "$desk = [Environment]::GetFolderPath('Desktop');\n\
                 $lnk = $ws.CreateShortcut((Join-Path $desk 'RRS - Riva Rust Store.lnk'));\n\
                 $lnk.TargetPath = '{exe_str}';\n\
                 $lnk.WorkingDirectory = '{dir_str}';\n\
                 $lnk.Description = 'RRS - Riva Rust Store: Loja Verificada de Apps e Jogos do GitHub + Tradutor C/C++ para Rust';\n\
                 $lnk.IconLocation = '{exe_str},0';\n\
                 $lnk.Save();\n"
            ));
        }

        if start_menu {
            script.push_str(&format!(
                "$progs = [Environment]::GetFolderPath('Programs');\n\
                 $lnk2 = $ws.CreateShortcut((Join-Path $progs 'RRS - Riva Rust Store.lnk'));\n\
                 $lnk2.TargetPath = '{exe_str}';\n\
                 $lnk2.WorkingDirectory = '{dir_str}';\n\
                 $lnk2.Description = 'RRS - Riva Rust Store: Loja Verificada de Apps e Jogos do GitHub + Tradutor C/C++ para Rust';\n\
                 $lnk2.IconLocation = '{exe_str},0';\n\
                 $lnk2.Save();\n"
            ));
        }

        if add_path {
            script.push_str(&format!(
                "$userPath = [Environment]::GetEnvironmentVariable('Path', 'User');\n\
                 if ($null -eq $userPath) {{ $userPath = '' }}\n\
                 $parts = $userPath -split ';' | Where-Object {{ $_ -ne '' }};\n\
                 if ($parts -notcontains '{dir_str}') {{\n\
                     $newPath = ($parts + '{dir_str}') -join ';';\n\
                     [Environment]::SetEnvironmentVariable('Path', $newPath, 'User');\n\
                 }}\n"
            ));
        }

        let _ = Command::new("powershell")
            .args(["-NoProfile", "-NonInteractive", "-WindowStyle", "Hidden", "-Command", &script])
            .status();
    }
    Ok(())
}

/// Garante a pasta no PATH do usuário (sem duplicar). Usado pelo "Adicionar ao terminal".
pub fn add_dir_to_user_path(dir: &Path) -> Result<()> {
    #[cfg(target_os = "windows")]
    {
        let dir_str = dir.to_string_lossy().replace('\'', "''");
        let script = format!(
            "$userPath = [Environment]::GetEnvironmentVariable('Path', 'User');\n\
             if ($null -eq $userPath) {{ $userPath = '' }}\n\
             $parts = $userPath -split ';' | Where-Object {{ $_ -ne '' }};\n\
             if ($parts -notcontains '{dir_str}') {{\n\
                 $newPath = ($parts + '{dir_str}') -join ';';\n\
                 [Environment]::SetEnvironmentVariable('Path', $newPath, 'User');\n\
             }}\n"
        );
        let status = Command::new("powershell")
            .args(["-NoProfile", "-NonInteractive", "-WindowStyle", "Hidden", "-Command", &script])
            .status()
            .context("não foi possível executar o PowerShell")?;
        if !status.success() {
            bail!("não foi possível atualizar o PATH do usuário");
        }
    }
    let _ = dir;
    Ok(())
}

fn remove_windows_shortcuts() -> Result<()> {
    #[cfg(target_os = "windows")]
    {
        let script = "\
            $desk = [Environment]::GetFolderPath('Desktop');\n\
            $progs = [Environment]::GetFolderPath('Programs');\n\
            Remove-Item -Path (Join-Path $desk 'RRS - Riva Rust Store.lnk') -Force -ErrorAction SilentlyContinue;\n\
            Remove-Item -Path (Join-Path $progs 'RRS - Riva Rust Store.lnk') -Force -ErrorAction SilentlyContinue;\n";
        let _ = Command::new("powershell")
            .args(["-NoProfile", "-NonInteractive", "-WindowStyle", "Hidden", "-Command", script])
            .status();
    }
    Ok(())
}

/// Abre a janela gráfica do Instalador Oficial da RRS - Riva Rust Store.
pub fn run_installer_gui() -> Result<()> {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_title("Instalador Oficial — RRS - Riva Rust Store")
            .with_icon(crate::brand::app_icon())
            .with_inner_size([640.0, 470.0])
            .with_min_inner_size([580.0, 440.0])
            .with_resizable(false),
        ..Default::default()
    };
    eframe::run_native(
        "Instalador — RRS - Riva Rust Store",
        options,
        Box::new(|cc| Ok(Box::new(InstallerWizard::new(cc)))),
    )
    .map_err(|e| anyhow::anyhow!("Erro ao abrir instalador gráfico: {e}"))
}

struct InstallerWizard {
    install_dir_input: String,
    desktop_shortcut: bool,
    start_menu_shortcut: bool,
    add_to_user_path: bool,
    launch_after_install: bool,
    status_msg: Option<(String, bool)>,
    installed_exe_path: Option<PathBuf>,
    /// Pasta aguardando o segundo clique de confirmação da desinstalação.
    confirm_uninstall: Option<PathBuf>,
}

impl InstallerWizard {
    fn new(cc: &eframe::CreationContext<'_>) -> Self {
        let mut visuals = egui::Visuals::dark();
        visuals.panel_fill = Color32::from_rgb(13, 16, 23);
        visuals.window_fill = Color32::from_rgb(18, 22, 32);
        cc.egui_ctx.set_visuals(visuals);
        crate::gui::configure_fonts(&cc.egui_ctx);

        let def = InstallOptions::default();
        Self {
            install_dir_input: def.install_dir.to_string_lossy().to_string(),
            desktop_shortcut: def.desktop_shortcut,
            start_menu_shortcut: def.start_menu_shortcut,
            add_to_user_path: def.add_to_user_path,
            launch_after_install: def.launch_after_install,
            status_msg: None,
            installed_exe_path: None,
            confirm_uninstall: None,
        }
    }
}

impl eframe::App for InstallerWizard {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default()
            .frame(
                egui::Frame::NONE
                    .fill(Color32::from_rgb(13, 16, 23))
                    .inner_margin(egui::Margin::symmetric(24, 20)),
            )
            .show(ctx, |ui| {
                // Banner de Cabeçalho do Instalador
                egui::Frame::NONE
                    .fill(Color32::from_rgb(22, 28, 44))
                    .stroke(Stroke::new(1.0_f32, Color32::from_rgb(55, 85, 145)))
                    .corner_radius(CornerRadius::same(12))
                    .inner_margin(egui::Margin::symmetric(18, 16))
                    .show(ui, |ui| {
                        ui.horizontal(|ui| {
                            crate::brand::logo_badge(ui, 20.0);
                            ui.add_space(10.0);
                            ui.vertical(|ui| {
                                ui.label(
                                    RichText::new("Instalador Oficial — RRS - Riva Rust Store")
                                        .size(19.0)
                                        .strong()
                                        .color(Color32::WHITE),
                                );
                                ui.label(
                                    RichText::new(
                                        "Loja Verificada de Apps e Jogos do GitHub (48 títulos) + Tradutor C/C++ ➔ Rust",
                                    )
                                    .size(12.5)
                                    .color(Color32::from_rgb(165, 185, 220)),
                                );
                            });
                        });
                    });

                ui.add_space(16.0);

                // Pasta de destino
                ui.label(
                    RichText::new("📂 Pasta de Instalação no Computador:")
                        .size(13.0)
                        .strong()
                        .color(Color32::from_rgb(210, 220, 235)),
                );
                ui.add_space(4.0);
                ui.add(
                    egui::TextEdit::singleline(&mut self.install_dir_input)
                        .desired_width(f32::INFINITY)
                        .margin(egui::Margin::symmetric(10, 8)),
                );

                ui.add_space(14.0);

                // Opções do instalador
                egui::Frame::NONE
                    .fill(Color32::from_rgb(19, 23, 33))
                    .stroke(Stroke::new(1.0_f32, Color32::from_rgb(36, 44, 62)))
                    .corner_radius(CornerRadius::same(10))
                    .inner_margin(egui::Margin::symmetric(16, 12))
                    .show(ui, |ui| {
                        ui.set_width(ui.available_width());
                        ui.checkbox(
                            &mut self.desktop_shortcut,
                            RichText::new("Criar ícone na Área de Trabalho (Desktop)")
                                .size(13.5)
                                .color(Color32::WHITE),
                        );
                        ui.add_space(4.0);
                        ui.checkbox(
                            &mut self.start_menu_shortcut,
                            RichText::new("Criar atalho no Menu Iniciar do Windows")
                                .size(13.5)
                                .color(Color32::WHITE),
                        );
                        ui.add_space(4.0);
                        ui.checkbox(
                            &mut self.add_to_user_path,
                            RichText::new(
                                "Adicionar comando 'rrs' ao Terminal (PowerShell / CMD) de qualquer pasta",
                            )
                            .size(13.5)
                            .color(Color32::WHITE),
                        );
                        ui.add_space(4.0);
                        ui.checkbox(
                            &mut self.launch_after_install,
                            RichText::new("Iniciar a RRS - Riva Rust Store automaticamente após instalar")
                                .size(13.5)
                                .strong()
                                .color(Color32::from_rgb(120, 230, 160)),
                        );
                    });

                ui.add_space(16.0);

                // Botões de ação
                ui.horizontal(|ui| {
                    let install_btn = egui::Button::new(
                        RichText::new("🚀 Instalar e Começar a Usar")
                            .size(15.0)
                            .strong()
                            .color(Color32::WHITE),
                    )
                    .fill(Color32::from_rgb(32, 145, 82))
                    .min_size(egui::vec2(270.0, 42.0))
                    .corner_radius(CornerRadius::same(8));

                    if ui.add(install_btn).clicked() {
                        let opts = InstallOptions {
                            install_dir: PathBuf::from(self.install_dir_input.trim()),
                            desktop_shortcut: self.desktop_shortcut,
                            start_menu_shortcut: self.start_menu_shortcut,
                            add_to_user_path: self.add_to_user_path,
                            launch_after_install: self.launch_after_install,
                        };
                        match perform_install(&opts) {
                            Ok(exe_path) => {
                                self.installed_exe_path = Some(exe_path.clone());
                                self.status_msg = Some((
                                    format!(
                                        "✔ Instalado com sucesso em: {}\nAtalhos criados na Área de Trabalho e no Menu Iniciar!",
                                        opts.install_dir.display()
                                    ),
                                    true,
                                ));
                                if self.launch_after_install {
                                    let _ = Command::new(&exe_path)
                                        .current_dir(&opts.install_dir)
                                        .spawn();
                                    ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                                }
                            }
                            Err(e) => {
                                self.status_msg = Some((format!("✘ Erro na instalação: {e}"), false));
                            }
                        }
                    }

                    ui.add_space(8.0);

                    let portable_btn = egui::Button::new(
                        RichText::new("▶ Abrir Sem Instalar (Portátil)")
                            .size(13.5)
                            .color(Color32::from_rgb(215, 225, 240)),
                    )
                    .fill(Color32::from_rgb(38, 46, 64))
                    .min_size(egui::vec2(210.0, 42.0))
                    .corner_radius(CornerRadius::same(8));

                    if ui.add(portable_btn).clicked() {
                        if let Ok(cur) = std::env::current_exe() {
                            let _ = Command::new(cur).arg("gui").spawn();
                            ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                        }
                    }

                    let target_dir = PathBuf::from(self.install_dir_input.trim());
                    if self.confirm_uninstall.as_ref() != Some(&target_dir) {
                        self.confirm_uninstall = None;
                    }
                    if is_rrs_install_dir(&target_dir) {
                        ui.add_space(6.0);
                        let confirming = self.confirm_uninstall.is_some();
                        let uninst_btn = egui::Button::new(
                            RichText::new(if confirming { "⚠ Confirmar" } else { "🗑 Desinstalar" })
                                .size(12.5)
                                .color(Color32::from_rgb(255, 175, 175)),
                        )
                        .fill(Color32::from_rgb(95, 32, 38))
                        .min_size(egui::vec2(95.0, 42.0))
                        .corner_radius(CornerRadius::same(8));

                        if ui.add(uninst_btn).clicked() {
                            if !confirming {
                                self.confirm_uninstall = Some(target_dir.clone());
                                self.status_msg = Some((
                                    format!(
                                        "Clique em '⚠ Confirmar' para remover a RRS de {}.\nSó os arquivos da RRS serão apagados; a pasta downloads (apps e jogos baixados) fica.",
                                        target_dir.display()
                                    ),
                                    false,
                                ));
                            } else {
                                self.confirm_uninstall = None;
                                self.status_msg = Some(match perform_uninstall(&target_dir) {
                                    Ok(rep) if rep.failed.is_empty() => (
                                        if rep.dir_kept {
                                            format!(
                                                "✔ RRS e atalhos removidos ({} arquivos). A pasta ficou porque ainda tem outros arquivos (ex.: downloads).",
                                                rep.removed.len()
                                            )
                                        } else {
                                            "✔ RRS, atalhos e pasta de instalação removidos.".into()
                                        },
                                        true,
                                    ),
                                    Ok(rep) => (
                                        format!(
                                            "✘ {} arquivo(s) não puderam ser removidos (feche a RRS e tente de novo): {}",
                                            rep.failed.len(),
                                            rep.failed
                                                .iter()
                                                .map(|(p, _)| p.display().to_string())
                                                .collect::<Vec<_>>()
                                                .join(", ")
                                        ),
                                        false,
                                    ),
                                    Err(e) => (format!("✘ Falha ao desinstalar: {e}"), false),
                                });
                            }
                        }
                    }
                });

                if let Some((msg, ok)) = &self.status_msg {
                    ui.add_space(14.0);
                    let bg = if *ok {
                        Color32::from_rgb(18, 52, 34)
                    } else {
                        Color32::from_rgb(62, 22, 26)
                    };
                    let fg = if *ok {
                        Color32::from_rgb(125, 235, 165)
                    } else {
                        Color32::from_rgb(255, 145, 145)
                    };
                    egui::Frame::NONE
                        .fill(bg)
                        .corner_radius(CornerRadius::same(8))
                        .inner_margin(egui::Margin::symmetric(12, 10))
                        .show(ui, |ui| {
                            ui.set_width(ui.available_width());
                            ui.label(RichText::new(msg).size(12.5).strong().color(fg));
                        });
                }
            });
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn temp_dir(tag: &str) -> PathBuf {
        let nanos = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let dir = std::env::temp_dir().join(format!("rrs-teste-{tag}-{}-{nanos}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        dir
    }

    fn touch(path: &Path) {
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent).unwrap();
        }
        std::fs::write(path, b"x").unwrap();
    }

    #[test]
    fn recusa_pasta_que_nao_e_instalacao() {
        let dir = temp_dir("alheia");
        touch(&dir.join("notas.md"));
        touch(&dir.join("catalog.json"));
        touch(&dir.join("docs").join("relatorio.txt"));

        assert!(!is_rrs_install_dir(&dir));
        assert!(remove_installed_files(&dir).is_err());
        assert!(perform_uninstall(&dir).is_err());
        assert!(dir.join("notas.md").exists());
        assert!(dir.join("catalog.json").exists());
        assert!(dir.join("docs").join("relatorio.txt").exists());

        std::fs::remove_dir_all(&dir).unwrap();
    }

    #[test]
    fn apaga_so_arquivos_da_rrs_e_mantem_downloads() {
        let dir = temp_dir("com-downloads");
        write_install_marker(&dir).unwrap();
        for name in [MAIN_EXE, "rrs.exe", "catalog.json", "gitstore.db"] {
            touch(&dir.join(name));
        }
        touch(&dir.join("meu-arquivo.txt"));
        touch(&dir.join("downloads").join("dono_jogo").join("jogo.exe"));

        let rep = remove_installed_files(&dir).unwrap();

        assert_eq!(rep.removed.len(), 5);
        assert!(rep.failed.is_empty());
        assert!(rep.dir_kept);
        for name in [MAIN_EXE, "rrs.exe", "catalog.json", "gitstore.db", INSTALL_MARKER] {
            assert!(!dir.join(name).exists(), "{name} deveria ter sido removido");
        }
        assert!(dir.join("meu-arquivo.txt").exists());
        assert!(dir.join("downloads").join("dono_jogo").join("jogo.exe").exists());

        std::fs::remove_dir_all(&dir).unwrap();
    }

    #[test]
    fn remove_a_pasta_quando_fica_vazia() {
        let dir = temp_dir("vazia");
        write_install_marker(&dir).unwrap();
        touch(&dir.join(MAIN_EXE));
        touch(&dir.join("gitstore.db"));

        let rep = remove_installed_files(&dir).unwrap();

        assert!(!rep.dir_kept);
        assert!(!dir.exists());
    }

    #[test]
    fn reconhece_instalacao_antiga_sem_marcador() {
        let dir = temp_dir("antiga");
        touch(&dir.join(MAIN_EXE));
        assert!(is_rrs_install_dir(&dir));
        assert!(remove_installed_files(&dir).is_ok());
        assert!(!dir.exists());
    }

    #[test]
    fn marcador_grava_json_valido() {
        let dir = temp_dir("marcador");
        write_install_marker(&dir).unwrap();
        assert!(is_rrs_install_dir(&dir));
        let raw = std::fs::read_to_string(dir.join(INSTALL_MARKER)).unwrap();
        let v: serde_json::Value = serde_json::from_str(&raw).unwrap();
        assert_eq!(v["version"], env!("CARGO_PKG_VERSION"));
        std::fs::remove_dir_all(&dir).unwrap();
    }
}
