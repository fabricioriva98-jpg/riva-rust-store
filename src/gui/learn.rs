//! Interface gráfica do espaço "Aprender & Programar em Rust" (Estúdio do Iniciante).
//! Lições práticas, editor de código interativo, execução via rustc e diagnóstico pedagógico.

use super::{draw_pill, BgEvent, StoreGui};
use crate::brand;
use crate::learn::{self, LESSONS};
use eframe::egui::{self, Color32, CornerRadius, RichText, Stroke, Vec2};
use std::sync::Arc;

impl StoreGui {
    /// Inicia a compilação e execução do código Rust em segundo plano, sem travar a interface.
    pub(super) fn spawn_learn_run(&mut self) {
        if self.learn_is_running {
            return;
        }
        self.learn_is_running = true;
        self.learn_cancel.store(false, std::sync::atomic::Ordering::Relaxed);
        let cancel = Arc::clone(&self.learn_cancel);
        let code = self.learn_code_input.clone();
        let events = Arc::clone(&self.bg_events);

        std::thread::spawn(move || {
            let res = learn::compile_and_run(&code, &cancel);
            events.lock().unwrap().push(BgEvent::LearnRunFinished(res));
        });
    }

    /// Renderiza a tela completa de Aprendizado e Programação em Rust.
    pub(super) fn render_learn_rust(&mut self, ui: &mut egui::Ui) {
        let lesson_count = LESSONS.len();
        if self.learn_lesson_idx >= lesson_count {
            self.learn_lesson_idx = 0;
        }
        let current_lesson = &LESSONS[self.learn_lesson_idx];

        // ─── CABEÇALHO DO ESTÚDIO ──────────────────────────────────────────
        ui.horizontal(|ui| {
            brand::logo_badge(ui, 18.0);
            ui.add_space(6.0);
            ui.vertical(|ui| {
                ui.label(
                    RichText::new(self.tr("📖 Aprender & Programar em Rust — Estúdio do Iniciante"))
                        .size(20.0)
                        .strong()
                        .color(Color32::WHITE),
                );
                ui.label(
                    RichText::new(self.tr(
                        "Aprenda Rust do zero com lições interativas, compile código direto na tela e crie seus primeiros programas.",
                    ))
                    .size(12.5)
                    .color(Color32::from_rgb(145, 160, 185)),
                );
            });
        });

        ui.add_space(10.0);

        // ─── BARRA DE FERRAMENTAS E SELEÇÃO DE LIÇÃO ───────────────────────
        egui::Frame::NONE
            .fill(Color32::from_rgb(22, 26, 36))
            .stroke(Stroke::new(1.0_f32, Color32::from_rgb(38, 46, 62)))
            .corner_radius(CornerRadius::same(8))
            .inner_margin(egui::Margin::symmetric(14, 10))
            .show(ui, |ui| {
                ui.horizontal_wrapped(|ui| {
                    ui.label(RichText::new(self.tr("Nível:")).strong().color(Color32::WHITE));
                    if ui.selectable_label(self.learn_level_filter == 0, self.tr("Todos (20)")).clicked() {
                        self.learn_level_filter = 0;
                    }
                    if ui.selectable_label(self.learn_level_filter == 1, self.tr("🐣 Básico (1-10)")).clicked() {
                        self.learn_level_filter = 1;
                        if self.learn_lesson_idx >= 10 {
                            self.learn_lesson_idx = 0;
                            self.learn_code_input = LESSONS[0].initial_code.to_string();
                            self.learn_output = None;
                        }
                    }
                    if ui.selectable_label(self.learn_level_filter == 2, self.tr("🚀 Intermediário (11-20)")).clicked() {
                        self.learn_level_filter = 2;
                        if self.learn_lesson_idx < 10 {
                            self.learn_lesson_idx = 10;
                            self.learn_code_input = LESSONS[10].initial_code.to_string();
                            self.learn_output = None;
                        }
                    }

                    ui.add_space(10.0);
                    ui.separator();
                    ui.add_space(8.0);

                    ui.label(RichText::new(self.tr("📚 Escolha a Lição:")).strong().color(Color32::WHITE));

                    let prev_idx = self.learn_lesson_idx;
                    egui::ComboBox::from_id_salt("learn_lesson_combo")
                        .width(320.0)
                        .selected_text(current_lesson.title)
                        .show_ui(ui, |ui| {
                            for (idx, l) in LESSONS.iter().enumerate() {
                                if self.learn_level_filter == 1 && l.level != "Básico" {
                                    continue;
                                }
                                if self.learn_level_filter == 2 && l.level != "Intermediário" {
                                    continue;
                                }
                                if ui.selectable_value(&mut self.learn_lesson_idx, idx, l.title).clicked() {
                                    // Se mudou de lição, carrega o código inicial correspondente
                                    if self.learn_lesson_idx != prev_idx {
                                        self.learn_code_input = LESSONS[self.learn_lesson_idx].initial_code.to_string();
                                        self.learn_output = None;
                                    }
                                }
                            }
                        });

                    ui.add_space(8.0);

                    if ui
                        .add(egui::Button::new(RichText::new(self.tr("🔄 Restaurar Lição")).size(12.5)))
                        .on_hover_text(self.tr("Restaura o código original desta lição."))
                        .clicked()
                    {
                        self.learn_code_input = current_lesson.initial_code.to_string();
                        self.learn_output = None;
                        self.status_msg = self.tr("Código inicial da lição restaurado.");
                        self.status_is_error = false;
                    }

                    ui.add_space(4.0);

                    if ui
                        .add(egui::Button::new(RichText::new(self.tr("📦 Gerar Projeto Cargo")).size(12.5)))
                        .on_hover_text(self.tr("Exporta este código como um projeto Cargo completo com Cargo.toml e main.rs."))
                        .clicked()
                    {
                        let stem = current_lesson.id.replace('_', "-");
                        let proj_name = format!("rrs-aprenda-{stem}");
                        let base = crate::terminal_studio::default_projects_dir();
                        match learn::generate_beginner_cargo_project(&base, &proj_name, &self.learn_code_input) {
                            Ok(p) => {
                                self.status_msg = format!("✔ {} {}", self.tr("Projeto Cargo criado em:"), p.display());
                                self.status_is_error = false;
                                let _ = crate::downloader::reveal_in_folder(&p.to_string_lossy());
                            }
                            Err(e) => {
                                self.status_msg = format!("{} {e}", self.tr("Erro ao exportar projeto:"));
                                self.status_is_error = true;
                            }
                        }
                    }

                    ui.add_space(4.0);

                    let cheat_label = if self.learn_show_cheatsheet {
                        self.tr("💡 Ocultar Dicas")
                    } else {
                        self.tr("💡 Guia Rápido")
                    };
                    if ui.selectable_label(self.learn_show_cheatsheet, cheat_label).clicked() {
                        self.learn_show_cheatsheet = !self.learn_show_cheatsheet;
                    }
                });
            });

        ui.add_space(10.0);

        // ─── CORPO PRINCIPAL EM COLUNAS DUPLAS ──────────────────────────────
        ui.columns(2, |cols| {
            // COLUNA 1: EXPLICAÇÃO, CONCEITOS E DESAFIO
            cols[0].vertical(|ui| {
                egui::ScrollArea::vertical()
                    .id_salt("learn_explanation_scroll")
                    .show(ui, |ui| {
                        // Cartão da Lição
                        egui::Frame::NONE
                            .fill(Color32::from_rgb(18, 22, 30))
                            .stroke(Stroke::new(1.0_f32, Color32::from_rgb(34, 42, 58)))
                            .corner_radius(CornerRadius::same(8))
                            .inner_margin(egui::Margin::same(14))
                            .show(ui, |ui| {
                                ui.horizontal(|ui| {
                                    draw_pill(
                                        ui,
                                        current_lesson.level,
                                        if current_lesson.level == "Básico" {
                                            Color32::from_rgb(30, 110, 60)
                                        } else {
                                            Color32::from_rgb(180, 75, 25)
                                        },
                                    );
                                    draw_pill(ui, current_lesson.category, Color32::from_rgb(35, 85, 155));
                                    ui.add_space(4.0);
                                    ui.label(
                                        RichText::new(current_lesson.title)
                                            .size(16.5)
                                            .strong()
                                            .color(Color32::WHITE),
                                    );
                                });

                                ui.add_space(4.0);
                                ui.label(
                                    RichText::new(current_lesson.summary)
                                        .size(12.5)
                                        .color(Color32::from_rgb(170, 185, 210)),
                                );

                                ui.add_space(10.0);
                                ui.separator();
                                ui.add_space(8.0);

                                // Texto detalhado da explicação
                                ui.label(
                                    RichText::new(current_lesson.explanation)
                                        .size(13.0)
                                        .color(Color32::from_rgb(220, 228, 240)),
                                );

                                ui.add_space(14.0);

                                // Caixa de Desafio
                                egui::Frame::NONE
                                    .fill(Color32::from_rgb(26, 32, 46))
                                    .stroke(Stroke::new(1.0_f32, Color32::from_rgb(55, 95, 160)))
                                    .corner_radius(CornerRadius::same(6))
                                    .inner_margin(egui::Margin::same(10))
                                    .show(ui, |ui| {
                                        ui.horizontal(|ui| {
                                            ui.label(
                                                RichText::new(self.tr("🎯 Desafio Prático:"))
                                                    .strong()
                                                    .color(Color32::from_rgb(115, 185, 255)),
                                            );
                                        });
                                        ui.add_space(4.0);
                                        ui.label(
                                            RichText::new(current_lesson.challenge)
                                                .size(12.5)
                                                .color(Color32::from_rgb(205, 220, 245)),
                                        );
                                    });
                            });

                        // Guia Rápido (Cheat Sheet Opcional)
                        if self.learn_show_cheatsheet {
                            ui.add_space(10.0);
                            egui::Frame::NONE
                                .fill(Color32::from_rgb(20, 24, 34))
                                .stroke(Stroke::new(1.0_f32, Color32::from_rgb(45, 55, 75)))
                                .corner_radius(CornerRadius::same(8))
                                .inner_margin(egui::Margin::same(12))
                                .show(ui, |ui| {
                                    ui.label(
                                        RichText::new(self.tr("💡 Guia Rápido do Rustáceo"))
                                            .size(14.5)
                                            .strong()
                                            .color(Color32::from_rgb(245, 165, 80)),
                                    );
                                    ui.add_space(6.0);
                                    ui.label(
                                        RichText::new(
                                            "• `let x = 5;`  ➔ Variável imutável (fixa)\n\
                                             • `let mut x = 5;`  ➔ Variável mutável (pode mudar)\n\
                                             • `i32` / `f64`  ➔ Números inteiros / decimais\n\
                                             • `println!(\"{}\", x);`  ➔ Imprime texto formatado\n\
                                             • `cargo run`  ➔ Compila e executa o projeto\n\
                                             • `cargo check`  ➔ Validação ultrarrápida de tipos",
                                        )
                                        .size(12.0)
                                        .color(Color32::from_rgb(180, 195, 220)),
                                    );
                                });
                        }
                    });
            });

            // COLUNA 2: EDITOR DE CÓDIGO E CONSOLE DE EXECUÇÃO
            cols[1].vertical(|ui| {
                // Barra de Ação do Editor
                ui.horizontal(|ui| {
                    ui.label(RichText::new(self.tr("💻 Editor de Código Rust:")).strong().color(Color32::WHITE));

                    ui.add_space(8.0);
                    {
                        let btn_text = if self.learn_is_running {
                            self.tr("⏳ Compilando...")
                        } else {
                            self.tr("▶ Executar Código")
                        };

                        let run_btn = egui::Button::new(
                            RichText::new(btn_text)
                                .size(13.5)
                                .strong()
                                .color(Color32::WHITE),
                        )
                        .fill(Color32::from_rgb(28, 120, 65))
                        .min_size(Vec2::new(160.0, 30.0));

                        if ui.add_enabled(!self.learn_is_running, run_btn).clicked() {
                            self.spawn_learn_run();
                        }
                        if self.learn_is_running
                            && ui
                                .add(egui::Button::new(self.tr("⏹ Parar")).fill(Color32::from_rgb(150, 50, 40)))
                                .on_hover_text(self.tr("Encerra a compilação ou o programa (para sozinho depois de 10 s rodando)."))
                                .clicked()
                        {
                            self.learn_cancel.store(true, std::sync::atomic::Ordering::Relaxed);
                        }
                    }
                });

                ui.add_space(6.0);

                // Editor de Texto Multilinha Monospace
                egui::Frame::NONE
                    .fill(Color32::from_rgb(12, 14, 20))
                    .stroke(Stroke::new(1.0_f32, Color32::from_rgb(32, 40, 56)))
                    .corner_radius(CornerRadius::same(6))
                    .inner_margin(egui::Margin::same(8))
                    .show(ui, |ui| {
                        ui.add(
                            egui::TextEdit::multiline(&mut self.learn_code_input)
                                .font(egui::TextStyle::Monospace)
                                .desired_rows(15)
                                .desired_width(f32::INFINITY)
                                .code_editor(),
                        );
                    });

                ui.add_space(10.0);

                // Painel de Saída do Terminal / rustc
                ui.horizontal(|ui| {
                    ui.label(
                        RichText::new(self.tr("📟 Saída do Terminal & Compilador:"))
                            .strong()
                            .color(Color32::from_rgb(190, 205, 225)),
                    );

                    if let Some(res) = &self.learn_output {
                        if res.success {
                            draw_pill(ui, &self.tr("✔ Sucesso"), Color32::from_rgb(20, 105, 50));
                        } else {
                            draw_pill(ui, &self.tr("❌ Erro"), Color32::from_rgb(145, 30, 30));
                        }
                        ui.label(
                            RichText::new(format!("⚡ {} ms", res.elapsed_ms))
                                .size(11.5)
                                .color(Color32::GRAY),
                        );
                    }
                });

                ui.add_space(4.0);

                egui::Frame::NONE
                    .fill(Color32::from_rgb(8, 10, 15))
                    .stroke(Stroke::new(1.0_f32, Color32::from_rgb(26, 32, 44)))
                    .corner_radius(CornerRadius::same(6))
                    .inner_margin(egui::Margin::same(10))
                    .show(ui, |ui| {
                        egui::ScrollArea::vertical()
                            .id_salt("learn_console_output_scroll")
                            .max_height(160.0)
                            .show(ui, |ui| {
                                if let Some(res) = &self.learn_output {
                                    let text_color = if res.success {
                                        Color32::from_rgb(110, 225, 140)
                                    } else {
                                        Color32::from_rgb(255, 120, 120)
                                    };
                                    ui.label(
                                        RichText::new(&res.output)
                                            .font(egui::FontId::monospace(12.0))
                                            .color(text_color),
                                    );
                                } else {
                                    ui.label(
                                        RichText::new(self.tr("Clique em '▶ Executar Código' para compilar e testar este programa."))
                                            .font(egui::FontId::monospace(12.0))
                                            .color(Color32::from_rgb(95, 105, 125)),
                                    );
                                }
                            });
                    });

                // Dica Pedagógica para Iniciantes (quando houver erro)
                if let Some(res) = &self.learn_output {
                    if let Some(tip) = &res.beginner_tip {
                        ui.add_space(6.0);
                        egui::Frame::NONE
                            .fill(Color32::from_rgb(32, 30, 18))
                            .stroke(Stroke::new(1.0_f32, Color32::from_rgb(160, 130, 40)))
                            .corner_radius(CornerRadius::same(6))
                            .inner_margin(egui::Margin::same(8))
                            .show(ui, |ui| {
                                ui.label(
                                    RichText::new(tip)
                                        .size(12.0)
                                        .color(Color32::from_rgb(255, 225, 130)),
                                );
                            });
                    }
                }
            });
        });
    }
}
