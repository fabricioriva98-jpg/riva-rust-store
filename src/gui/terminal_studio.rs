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
        self.busy_task = Some(format!("Executando: {trimmed}"));

        // Adiciona o comando no histórico visível do terminal
        self.terminal_logs.push((
            terminal_studio::TerminalLineKind::Command,
            format!("$ {trimmed}"),
        ));

        let dir = self.terminal_working_dir.clone();
        let events = Arc::clone(&self.bg_events);

        std::thread::spawn(move || {
            let out = terminal_studio::run_terminal_command(&dir, &trimmed);
            events.lock().unwrap().push(BgEvent::TerminalCommandFinished(out));
        });

        self.terminal_cmd_input.clear();
    }

    /// Renderiza a tela completa de Terminal Integrado e Espelho de IDEs.
    pub(super) fn render_terminal_studio(&mut self, ui: &mut egui::Ui) {
        // Se a pasta de trabalho ainda não foi definida, usa o diretório do projeto ou base
        if self.terminal_working_dir.as_os_str().is_empty() {
            self.terminal_working_dir = self.base_dir();
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

        // ─── PAINEL DE INTEGRAÇÃO COM IDES (VS CODE & ANTIGRAVITY) ──────────
        let ides = ide::detect_ides();
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
                        .add(egui::Button::new(self.tr("⚙ Instalar rust-analyzer no VS Code")))
                        .on_hover_text(self.tr("Instala a extensão oficial de autocompletar e conferência de tipos no VS Code."))
                        .clicked()
                    {
                        match ide::install_vscode_extension("rust-lang.rust-analyzer") {
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
                        .add(egui::Button::new(self.tr("📁 Abrir Pasta no Explorer")))
                        .on_hover_text(self.tr("Abre a pasta atual no Windows Explorer."))
                        .clicked()
                    {
                        let _ = crate::downloader::reveal_in_folder(&dir.to_string_lossy());
                    }
                });
            });

        ui.add_space(10.0);

        // ─── DIRETÓRIO DE TRABALHO E ATALHOS RÁPIDOS ───────────────────────
        egui::Frame::NONE
            .fill(Color32::from_rgb(16, 20, 28))
            .stroke(Stroke::new(1.0_f32, Color32::from_rgb(32, 40, 56)))
            .corner_radius(CornerRadius::same(6))
            .inner_margin(egui::Margin::symmetric(10, 8))
            .show(ui, |ui| {
                ui.horizontal(|ui| {
                    ui.label(RichText::new(self.tr("📂 Pasta de Trabalho:")).strong().size(12.0).color(Color32::from_rgb(160, 175, 200)));
                    ui.label(
                        RichText::new(self.terminal_working_dir.to_string_lossy().to_string())
                            .monospace()
                            .size(12.0)
                            .color(Color32::WHITE),
                    );

                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        if ui.button(self.tr("🧹 Limpar Saída")).clicked() {
                            self.terminal_logs.clear();
                        }
                        if ui.button(self.tr("🔄 Resetar Pasta")).clicked() {
                            self.terminal_working_dir = self.base_dir();
                        }
                    });
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
        let available_h = (ui.available_height() - 55.0).max(220.0);
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
                    .desired_width(ui.available_width() - 110.0),
            );

            let enter_pressed = resp.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter));
            let btn_clicked = ui
                .add_enabled(
                    !self.terminal_is_running && !self.terminal_cmd_input.trim().is_empty(),
                    egui::Button::new(RichText::new(self.tr("▶ Executar")).strong()),
                )
                .clicked();

            if enter_pressed || btn_clicked {
                let cmd = self.terminal_cmd_input.clone();
                self.spawn_terminal_cmd(&cmd);
            }
        });
    }
}
