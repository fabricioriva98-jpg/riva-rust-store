//! Interface gráfica do Terminal Integrado e Espelhamento de Editores de Codificação (VS Code & Google Antigravity).
//! Executa comandos Cargo/Rust em segundo plano, monitora arquivos de projeto e conecta com IDEs com 1 clique.

use super::{draw_pill, BgEvent, StoreGui};
use crate::brand;
use crate::ide;
use crate::terminal_studio;
use eframe::egui::{self, Color32, CornerRadius, RichText, Stroke};
use std::sync::Arc;

impl StoreGui {
    /// Inicia a execução de um comando no terminal integrado em segundo plano sem travar a interface.
    pub(super) fn spawn_terminal_cmd(&mut self, cmd_line: &str) {
        let trimmed = cmd_line.trim().to_string();
        if trimmed.is_empty() || self.terminal_is_running {
            return;
        }

        self.terminal_is_running = true;
        self.terminal_cancel.store(false, std::sync::atomic::Ordering::Relaxed);

        // Adiciona o comando no histórico visível do terminal
        self.terminal_logs.push((
            terminal_studio::TerminalLineKind::Command,
            format!("$ {trimmed}"),
        ));

        let dir = self.terminal_working_dir.clone();
        let events = Arc::clone(&self.bg_events);
        let cancel = Arc::clone(&self.terminal_cancel);

        std::thread::spawn(move || {
            let out = terminal_studio::run_terminal_command(&dir, &trimmed, &cancel);
            events.lock().unwrap().push(BgEvent::TerminalCommandFinished(out));
        });

        self.terminal_cmd_input.clear();
    }

    /// Avalia uma expressão no avaliador (compila e roda em segundo plano, com tempo máximo).
    fn spawn_terminal_eval(&mut self, expr: String) {
        if self.terminal_is_running {
            return;
        }
        self.terminal_is_running = true;
        self.terminal_cancel.store(false, std::sync::atomic::Ordering::Relaxed);
        self.terminal_logs.push((terminal_studio::TerminalLineKind::Command, format!(">> eval: {}", expr.trim())));
        let events = Arc::clone(&self.bg_events);
        let cancel = Arc::clone(&self.terminal_cancel);
        std::thread::spawn(move || {
            let out = terminal_studio::eval_rust_expression(&expr, &cancel);
            events.lock().unwrap().push(BgEvent::TerminalCommandFinished(out));
        });
    }

    /// Detecta a toolchain Rust e os editores em segundo plano (são vários programas e arquivos;
    /// na linha da interface travariam a tela).
    pub(super) fn spawn_dev_env_detection(&mut self) {
        if self.dev_env_detecting {
            return;
        }
        self.dev_env_detecting = true;
        let events = Arc::clone(&self.bg_events);
        std::thread::spawn(move || {
            let env = terminal_studio::detect_rust_env();
            let ides = ide::detect_ides();
            events.lock().unwrap().push(BgEvent::DevEnvDetected { env, ides });
        });
    }

    /// Renderiza a tela completa de Terminal Integrado e Espelho de IDEs.
    pub(super) fn render_terminal_studio(&mut self, ui: &mut egui::Ui) {
        // Se a pasta de trabalho ainda não foi definida, usa o diretório do projeto ou base
        if self.terminal_working_dir.as_os_str().is_empty() {
            self.terminal_working_dir = terminal_studio::default_projects_dir();
            self.terminal_dir_input = self.terminal_working_dir.to_string_lossy().to_string();
        }

        // ─── CABEÇALHO DO ESTÚDIO ──────────────────────────────────────────
        ui.horizontal(|ui| {
            brand::logo_badge(ui, 18.0);
            ui.add_space(6.0);
            ui.vertical(|ui| {
                ui.horizontal(|ui| {
                    ui.label(
                        RichText::new(self.tr("💻 Terminal Integrado & Espelho de IDEs (VS Code / Antigravity)"))
                            .size(20.0)
                            .strong()
                            .color(Color32::WHITE),
                    );
                    draw_pill(ui, &self.tr("Ambiente de Desenvolvimento"), Color32::from_rgb(30, 120, 200));
                });
                ui.label(
                    RichText::new(self.tr(
                        "Execute comandos Cargo/Rust em tempo real e conecte seus projetos instantaneamente aos seus programas de codificação.",
                    ))
                    .size(12.5)
                    .color(Color32::from_rgb(145, 160, 185)),
                );
            });
        });

        ui.add_space(10.0);

        // ─── BARRA DE DETECÇÃO DO AMBIENTE RUST (TOOLCHAIN & FERRAMENTAS) ──
        if self.rust_env_info.is_none() {
            self.spawn_dev_env_detection();
            ui.label(RichText::new(self.tr("⏳ Detectando a toolchain Rust e os editores...")).color(Color32::GRAY));
        }

        let mut refresh_env = false;
        if let Some(env_info) = self.rust_env_info.clone() {
            egui::Frame::NONE
                .fill(Color32::from_rgb(18, 24, 34))
                .stroke(Stroke::new(1.0_f32, Color32::from_rgb(36, 48, 70)))
                .corner_radius(CornerRadius::same(8))
                .inner_margin(egui::Margin::symmetric(12, 9))
                .show(ui, |ui| {
                    ui.horizontal_wrapped(|ui| {
                        ui.label(RichText::new(self.tr("🦀 Toolchain Rust do Sistema:")).strong().color(Color32::WHITE));
                        ui.add_space(6.0);

                        if let Some(ref v) = env_info.rustc_version {
                            draw_pill(ui, v, Color32::from_rgb(30, 110, 60));
                        } else {
                            draw_pill(ui, &self.tr("rustc não detectado"), Color32::from_rgb(140, 40, 40));
                        }

                        if let Some(ref v) = env_info.cargo_version {
                            draw_pill(ui, v, Color32::from_rgb(35, 75, 140));
                        }

                        if let Some(ref v) = env_info.rustup_version {
                            draw_pill(ui, v, Color32::from_rgb(60, 60, 90));
                        }

                        if env_info.clippy_installed {
                            draw_pill(ui, "clippy: ✔", Color32::from_rgb(40, 100, 70));
                        }

                        if env_info.rust_analyzer_installed {
                            draw_pill(ui, "rust-analyzer: ✔", Color32::from_rgb(40, 100, 70));
                        }

                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            if ui.button(self.tr("🔄 Atualizar")).clicked() {
                                refresh_env = true;
                            }
                        });
                    });
                });

            ui.add_space(8.0);
        }

        if refresh_env {
            self.spawn_dev_env_detection();
        }

        // ─── PAINEL DE INTEGRAÇÃO COM IDES (VS CODE & ANTIGRAVITY) ──────────
        let ides = self.detected_ides.clone();
        egui::Frame::NONE
            .fill(Color32::from_rgb(20, 24, 34))
            .stroke(Stroke::new(1.0_f32, Color32::from_rgb(38, 46, 62)))
            .corner_radius(CornerRadius::same(8))
            .inner_margin(egui::Margin::same(12))
            .show(ui, |ui| {
                ui.horizontal_wrapped(|ui| {
                    ui.label(RichText::new(self.tr("🚀 Programas de Codificação Detectados:")).strong().color(Color32::WHITE));
                    ui.add_space(8.0);

                    for ide_item in &ides {
                        if ide_item.is_installed {
                            draw_pill(ui, &format!("{} (Pronto)", ide_item.badge), Color32::from_rgb(30, 110, 60));
                        } else {
                            draw_pill(ui, &format!("{} (Não detectado)", ide_item.badge), Color32::from_rgb(60, 66, 80));
                        }
                    }
                });

                ui.add_space(8.0);
                ui.horizontal_wrapped(|ui| {
                    let dir = self.terminal_working_dir.clone();

                    if ui
                        .add(egui::Button::new(RichText::new(self.tr("🟦 Abrir no VS Code")).strong()))
                        .on_hover_text(self.tr("Abre a pasta atual do projeto no Visual Studio Code."))
                        .clicked()
                    {
                        match ide::open_workspace_in_ide("vscode", &dir) {
                            Ok(msg) => {
                                self.status_msg = format!("✔ {msg}");
                                self.status_is_error = false;
                            }
                            Err(e) => {
                                self.status_msg = format!("❌ {e}");
                                self.status_is_error = true;
                            }
                        }
                    }

                    if ui
                        .add(egui::Button::new(RichText::new(self.tr("⚛ Abrir no Antigravity")).strong()))
                        .on_hover_text(self.tr("Abre o workspace atual no Google Antigravity."))
                        .clicked()
                    {
                        match ide::open_workspace_in_ide("antigravity", &dir) {
                            Ok(msg) => {
                                self.status_msg = format!("✔ {msg}");
                                self.status_is_error = false;
                            }
                            Err(e) => {
                                self.status_msg = format!("❌ {e}");
                                self.status_is_error = true;
                            }
                        }
                    }

                    ui.add_space(6.0);
                    ui.separator();
                    ui.add_space(6.0);

                    if ui
                        .add_enabled(self.busy_task.is_none(), egui::Button::new(self.tr("⚙ Instalar rust-analyzer no VS Code")))
                        .on_hover_text(self.tr("Instala a extensão oficial de autocompletar e conferência de tipos no VS Code."))
                        .clicked()
                    {
                        // Leva de segundos a minutos (baixa a extensão): em segundo plano
                        self.busy_task = Some(self.tr("Instalando rust-analyzer no VS Code..."));
                        let events = Arc::clone(&self.bg_events);
                        std::thread::spawn(move || {
                            let res = ide::install_vscode_extension("rust-lang.rust-analyzer").map_err(|e| format!("{e:#}"));
                            events.lock().unwrap().push(BgEvent::IdeActionFinished(res));
                        });
                    }

                    if ui
                        .add(egui::Button::new(self.tr("📁 Abrir Pasta no Explorer")))
                        .on_hover_text(self.tr("Abre a pasta atual no Windows Explorer."))
                        .clicked()
                    {
                        let _ = crate::downloader::reveal_in_folder(&dir.to_string_lossy());
                    }
                });
            });

        ui.add_space(8.0);

        // ─── FERRAMENTAS RÁPIDAS: NOVO PROJETO CARGO & AVALIADOR SCRATCHPAD ─
        egui::Frame::NONE
            .fill(Color32::from_rgb(18, 22, 30))
            .stroke(Stroke::new(1.0_f32, Color32::from_rgb(34, 42, 58)))
            .corner_radius(CornerRadius::same(6))
            .inner_margin(egui::Margin::symmetric(10, 8))
            .show(ui, |ui| {
                ui.horizontal_wrapped(|ui| {
                    // GERADOR CARGO
                    ui.label(RichText::new(self.tr("📦 Novo Projeto Cargo:")).strong().size(12.0).color(Color32::WHITE));
                    let cargo_hint = self.tr("nome_do_projeto");
                    ui.add(
                        egui::TextEdit::singleline(&mut self.cargo_new_name)
                            .hint_text(cargo_hint)
                            .desired_width(120.0),
                    );

                    ui.radio_value(&mut self.cargo_new_is_bin, true, "--bin (App)");
                    ui.radio_value(&mut self.cargo_new_is_bin, false, "--lib (Lib)");

                    if ui
                        .add_enabled(
                            !self.cargo_new_name.trim().is_empty() && !self.terminal_is_running,
                            egui::Button::new(RichText::new(self.tr("➕ Criar")).strong()),
                        )
                        .clicked()
                    {
                        let name = self.cargo_new_name.trim().to_string();
                        match terminal_studio::create_cargo_project(&self.terminal_working_dir, &name, self.cargo_new_is_bin) {
                            Ok(msg) => {
                                self.terminal_logs.push((
                                    terminal_studio::TerminalLineKind::SystemInfo,
                                    format!("✔ {}", msg),
                                ));
                                let project = self.terminal_working_dir.join(&name);
                                if project.is_dir() {
                                    self.terminal_dir_input = project.to_string_lossy().to_string();
                                    self.terminal_working_dir = project;
                                }
                                self.status_msg = msg;
                                self.status_is_error = false;
                                self.cargo_new_name.clear();
                            }
                            Err(e) => {
                                self.terminal_logs.push((
                                    terminal_studio::TerminalLineKind::Stderr,
                                    format!("❌ {}", e),
                                ));
                                self.status_msg = e;
                                self.status_is_error = true;
                            }
                        }
                    }

                    ui.add_space(8.0);
                    ui.separator();
                    ui.add_space(8.0);

                    // AVALIADOR REPL EXPRESSÕES
                    ui.label(RichText::new(self.tr("⚡ Scratchpad REPL:")).strong().size(12.0).color(Color32::WHITE));
                    let repl_hint = self.tr("Expressão Rust (ex: 2 + 2 * 10)");
                    let repl_resp = ui.add(
                        egui::TextEdit::singleline(&mut self.terminal_repl_input)
                            .hint_text(repl_hint)
                            .desired_width(170.0),
                    );

                    let eval_enter = repl_resp.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter));
                    let eval_clicked = ui
                        .add_enabled(
                            !self.terminal_repl_input.trim().is_empty() && !self.terminal_is_running,
                            egui::Button::new(RichText::new(self.tr("▶ Avaliar")).strong()),
                        )
                        .clicked();

                    if (eval_enter || eval_clicked) && !self.terminal_repl_input.trim().is_empty() && !self.terminal_is_running {
                        let expr = std::mem::take(&mut self.terminal_repl_input);
                        self.spawn_terminal_eval(expr);
                    }
                });
            });

        ui.add_space(8.0);

        // ─── DIRETÓRIO DE TRABALHO E ATALHOS RÁPIDOS ───────────────────────
        egui::Frame::NONE
            .fill(Color32::from_rgb(16, 20, 28))
            .stroke(Stroke::new(1.0_f32, Color32::from_rgb(32, 40, 56)))
            .corner_radius(CornerRadius::same(6))
            .inner_margin(egui::Margin::symmetric(10, 8))
            .show(ui, |ui| {
                ui.horizontal_wrapped(|ui| {
                    ui.label(RichText::new(self.tr("📂 Pasta de Trabalho:")).strong().size(12.0).color(Color32::from_rgb(160, 175, 200)));
                    ui.add(
                        egui::TextEdit::singleline(&mut self.terminal_dir_input)
                            .font(egui::TextStyle::Monospace)
                            .desired_width(380.0),
                    );
                    if ui
                        .add_enabled(!self.terminal_is_running, egui::Button::new(self.tr("Usar pasta")))
                        .clicked()
                    {
                        let p = std::path::PathBuf::from(self.terminal_dir_input.trim());
                        if p.is_dir() {
                            self.terminal_working_dir = p;
                        } else {
                            self.status_msg = format!("❌ {} {}", self.tr("Pasta não encontrada:"), self.terminal_dir_input.trim());
                            self.status_is_error = true;
                        }
                    }
                    if ui
                        .add_enabled(!self.terminal_is_running, egui::Button::new(self.tr("🔄 Resetar Pasta")))
                        .on_hover_text(self.tr("Volta para a pasta Projetos Rust, dentro de Documentos."))
                        .clicked()
                    {
                        self.terminal_working_dir = terminal_studio::default_projects_dir();
                        self.terminal_dir_input = self.terminal_working_dir.to_string_lossy().to_string();
                    }
                    if ui.button(self.tr("🧹 Limpar Saída")).clicked() {
                        self.terminal_logs.clear();
                    }
                });
            });

        ui.add_space(8.0);

        // ─── BOTÕES RÁPIDOS DE CARGO ───────────────────────────────────────
        ui.horizontal_wrapped(|ui| {
            ui.label(RichText::new(self.tr("Atalhos Cargo:")).strong().size(12.0).color(Color32::WHITE));

            let shortcuts = [
                ("⚡ cargo check", "cargo check"),
                ("🔨 cargo build", "cargo build"),
                ("▶ cargo run", "cargo run"),
                ("🧪 cargo test", "cargo test"),
                ("🦀 rustc --version", "rustc --version"),
                ("🔍 rustup show", "rustup show"),
                ("📦 cargo clean", "cargo clean"),
            ];

            for (label, cmd) in shortcuts {
                if ui
                    .add_enabled(!self.terminal_is_running, egui::Button::new(RichText::new(label).size(12.0)))
                    .clicked()
                {
                    self.spawn_terminal_cmd(cmd);
                }
            }
        });

        ui.add_space(8.0);

        // ─── CONSOLE DO TERMINAL INTEGRADO (SAÍDA COLORIDA) ────────────────
        let available_h = (ui.available_height() - 55.0).max(200.0);
        egui::Frame::NONE
            .fill(Color32::from_rgb(10, 12, 16))
            .stroke(Stroke::new(1.0_f32, Color32::from_rgb(26, 32, 44)))
            .corner_radius(CornerRadius::same(6))
            .inner_margin(egui::Margin::same(10))
            .show(ui, |ui| {
                ui.set_height(available_h);
                egui::ScrollArea::vertical()
                    .id_salt("terminal_output_scroll")
                    .stick_to_bottom(true)
                    .show(ui, |ui| {
                        ui.set_width(ui.available_width());

                        if self.terminal_logs.is_empty() {
                            ui.label(
                                RichText::new(self.tr(
                                    "Terminal pronto. Digite um comando abaixo ou use os atalhos rápidos do Cargo.",
                                ))
                                .monospace()
                                .size(12.0)
                                .color(Color32::from_rgb(110, 125, 145)),
                            );
                        }

                        for (kind, text) in &self.terminal_logs {
                            let text_col = match kind {
                                terminal_studio::TerminalLineKind::Command => Color32::from_rgb(80, 220, 120),
                                terminal_studio::TerminalLineKind::Stdout => Color32::from_rgb(220, 225, 235),
                                terminal_studio::TerminalLineKind::Stderr => Color32::from_rgb(240, 90, 80),
                                terminal_studio::TerminalLineKind::SystemInfo => Color32::from_rgb(90, 175, 240),
                            };

                            ui.label(
                                RichText::new(text)
                                    .monospace()
                                    .size(12.0)
                                    .color(text_col),
                            );
                        }
                    });
            });

        ui.add_space(6.0);

        // ─── BARRA DE ENTRADA DO COMANDO ───────────────────────────────────
        ui.horizontal(|ui| {
            ui.label(RichText::new("$").monospace().strong().size(15.0).color(Color32::from_rgb(80, 220, 120)));

            let input_hint = self.tr("Digite um comando (ex: cargo build, cargo add serde, rustc --version)...");
            let resp = ui.add(
                egui::TextEdit::singleline(&mut self.terminal_cmd_input)
                    .hint_text(input_hint)
                    .desired_width(ui.available_width() - 200.0),
            );

            let enter_pressed = resp.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter));
            let btn_clicked = ui
                .add_enabled(
                    !self.terminal_is_running && !self.terminal_cmd_input.trim().is_empty(),
                    egui::Button::new(RichText::new(self.tr("▶ Executar")).strong()),
                )
                .clicked();

            if self.terminal_is_running
                && ui
                    .add(egui::Button::new(RichText::new(self.tr("⏹ Parar")).strong()).fill(Color32::from_rgb(150, 50, 40)))
                    .on_hover_text(self.tr("Encerra o comando e os programas que ele abriu."))
                    .clicked()
            {
                self.terminal_cancel.store(true, std::sync::atomic::Ordering::Relaxed);
            }

            if enter_pressed || btn_clicked {
                let cmd = self.terminal_cmd_input.clone();
                self.spawn_terminal_cmd(&cmd);
            }
        });
        ui.label(
            RichText::new(self.tr(
                "Os comandos rodam no PowerShell com as permissões da sua conta. Cole só comandos em que você confia. Tempo máximo: 30 min por comando.",
            ))
            .size(11.0)
            .color(Color32::from_rgb(140, 150, 170)),
        );
    }
}
