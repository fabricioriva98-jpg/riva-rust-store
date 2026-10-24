use anyhow::{Context, Result};
use eframe::egui::{self, Color32, CornerRadius, RichText, Stroke};
use std::path::{Path, PathBuf};
use std::process::Command;

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
    let target_gui_exe = opts.install_dir.join("RRS - Riva Rust Store.exe");
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

pub fn perform_uninstall(install_dir: &Path) -> Result<()> {
    remove_windows_shortcuts()?;
    if install_dir.exists() {
        let _ = std::fs::remove_dir_all(install_dir);
    }
    Ok(())
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
}

impl InstallerWizard {
    fn new(cc: &eframe::CreationContext<'_>) -> Self {
        let mut visuals = egui::Visuals::dark();
        visuals.panel_fill = Color32::from_rgb(13, 16, 23);
        visuals.window_fill = Color32::from_rgb(18, 22, 32);
        cc.egui_ctx.set_visuals(visuals);

        let def = InstallOptions::default();
        Self {
            install_dir_input: def.install_dir.to_string_lossy().to_string(),
            desktop_shortcut: def.desktop_shortcut,
            start_menu_shortcut: def.start_menu_shortcut,
            add_to_user_path: def.add_to_user_path,
            launch_after_install: def.launch_after_install,
            status_msg: None,
            installed_exe_path: None,
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
                            egui::Frame::NONE
                                .fill(Color32::from_rgb(215, 75, 40))
                                .corner_radius(CornerRadius::same(10))
                                .inner_margin(egui::Margin::symmetric(12, 10))
                                .show(ui, |ui| {
                                    ui.label(
                                        RichText::new("🦀 RRS")
                                            .size(20.0)
                                            .strong()
                                            .color(Color32::WHITE),
                                    );
                                });
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
                    if target_dir.exists() {
                        ui.add_space(6.0);
                        let uninst_btn = egui::Button::new(
                            RichText::new("🗑 Desinstalar")
                                .size(12.5)
                                .color(Color32::from_rgb(255, 175, 175)),
                        )
                        .fill(Color32::from_rgb(95, 32, 38))
                        .min_size(egui::vec2(95.0, 42.0))
                        .corner_radius(CornerRadius::same(8));

                        if ui.add(uninst_btn).clicked() {
                            match perform_uninstall(&target_dir) {
                                Ok(_) => {
                                    self.status_msg = Some((
                                        "✔ Aplicativo e atalhos removidos com sucesso.".into(),
                                        true,
                                    ));
                                }
                                Err(e) => {
                                    self.status_msg =
                                        Some((format!("✘ Falha ao desinstalar: {e}"), false));
                                }
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
