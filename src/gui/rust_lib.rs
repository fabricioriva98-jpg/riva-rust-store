//! Interface gráfica do Catálogo e Referência Completa da Biblioteca Rust (std & Crates).
//! Exibe documentação em português, tipos essenciais, métodos principais e permite testar códigos no editor.

use super::{draw_pill, NavSection, StoreGui};
use crate::brand;
use crate::rust_lib::{self, RUST_LIBS};
use eframe::egui::{self, Color32, CornerRadius, RichText, Stroke};

impl StoreGui {
    /// Renderiza a tela de referência e exploração da Biblioteca Rust.
    pub(super) fn render_rust_lib_browser(&mut self, ui: &mut egui::Ui) {
        if self.rust_lib_selected_idx >= RUST_LIBS.len() {
            self.rust_lib_selected_idx = 0;
        }
        let current_lib = &RUST_LIBS[self.rust_lib_selected_idx];

        // ─── CABEÇALHO DA BIBLIOTECA ───────────────────────────────────────
        ui.horizontal(|ui| {
            brand::logo_badge(ui, 18.0);
            ui.add_space(6.0);
            ui.vertical(|ui| {
                ui.horizontal(|ui| {
                    ui.label(
                        RichText::new(self.tr("📚 Biblioteca Rust — Standard Library (`std`) & Crates"))
                            .size(20.0)
                            .strong()
                            .color(Color32::WHITE),
                    );
                    draw_pill(ui, &self.tr("Documentação & Exemplos"), Color32::from_rgb(220, 105, 30));
                });
                ui.label(
                    RichText::new(self.tr(
                        "Referência completa da Standard Library e dos principais pacotes para desenvolvimento em Rust com exemplos executáveis.",
                    ))
                    .size(12.5)
                    .color(Color32::from_rgb(145, 160, 185)),
                );
            });
        });

        ui.add_space(10.0);

        // ─── BARRA DE BUSCA E FILTROS ──────────────────────────────────────
        let search_hint = self.tr("Buscar por módulo, tipo ou crate (ex: HashMap, Arc, Mutex, serde, tokio)...");
        egui::Frame::NONE
            .fill(Color32::from_rgb(20, 24, 34))
            .stroke(Stroke::new(1.0_f32, Color32::from_rgb(38, 46, 62)))
            .corner_radius(CornerRadius::same(8))
            .inner_margin(egui::Margin::symmetric(14, 10))
            .show(ui, |ui| {
                ui.horizontal_wrapped(|ui| {
                    ui.add(
                        egui::TextEdit::singleline(&mut self.rust_lib_search)
                            .hint_text(search_hint)
                            .desired_width(320.0),
                    );

                    ui.add_space(8.0);
                    ui.separator();
                    ui.add_space(8.0);

                    ui.label(RichText::new(self.tr("Categoria:")).strong().color(Color32::WHITE));

                    let categories = [
                        ("Todas", "todas"),
                        ("Coleções", "coleções"),
                        ("Concorrência", "concorrência"),
                        ("I/O & Arquivos", "arquivos"),
                        ("Redes", "redes"),
                        ("Sistema & SO", "processos"),
                        ("Memória & Tipos", "memória"),
                        ("Ecossistema Crates", "crates"),
                    ];

                    for (label, key) in categories {
                        let is_active = self.rust_lib_category_filter == key;
                        if ui.selectable_label(is_active, self.tr(label)).clicked() {
                            self.rust_lib_category_filter = key.to_string();
                        }
                    }
                });
            });

        ui.add_space(10.0);

        // ─── CONTEÚDO EM COLUNAS DUPLAS ────────────────────────────────────
        let filtered = rust_lib::search_libs(&self.rust_lib_search, &self.rust_lib_category_filter);

        ui.columns(2, |cols| {
            // COLUNA 1: LISTA DE MÓDULOS / CRATES
            cols[0].vertical(|ui| {
                ui.label(
                    RichText::new(format!(
                        "{} {} {}",
                        self.tr("Encontrados"),
                        filtered.len(),
                        self.tr("módulos e crates")
                    ))
                    .size(12.0)
                    .color(Color32::from_rgb(140, 155, 175)),
                );

                ui.add_space(4.0);

                egui::ScrollArea::vertical()
                    .id_salt("rust_lib_list_scroll")
                    .show(ui, |ui| {
                        for item in &filtered {
                            let is_selected = RUST_LIBS[self.rust_lib_selected_idx].id == item.id;
                            let frame_col = if is_selected {
                                Color32::from_rgb(32, 44, 64)
                            } else {
                                Color32::from_rgb(18, 22, 30)
                            };

                            let border_col = if is_selected {
                                Color32::from_rgb(50, 120, 200)
                            } else {
                                Color32::from_rgb(30, 36, 48)
                            };

                            egui::Frame::NONE
                                .fill(frame_col)
                                .stroke(Stroke::new(1.0_f32, border_col))
                                .corner_radius(CornerRadius::same(6))
                                .inner_margin(egui::Margin::same(10))
                                .show(ui, |ui| {
                                    ui.set_width(ui.available_width());
                                    ui.horizontal(|ui| {
                                        if item.is_std {
                                            draw_pill(ui, "std", Color32::from_rgb(40, 110, 80));
                                        } else {
                                            draw_pill(ui, "crate", Color32::from_rgb(160, 80, 20));
                                        }

                                        let btn = ui.selectable_label(
                                            is_selected,
                                            RichText::new(item.name).strong().size(13.5).color(Color32::WHITE),
                                        );
                                        if btn.clicked() {
                                            if let Some(pos) = RUST_LIBS.iter().position(|l| l.id == item.id) {
                                                self.rust_lib_selected_idx = pos;
                                            }
                                        }
                                    });

                                    ui.add_space(2.0);
                                    ui.label(
                                        RichText::new(item.summary)
                                            .size(11.5)
                                            .color(Color32::from_rgb(160, 175, 195)),
                                    );
                                });

                            ui.add_space(4.0);
                        }
                    });
            });

            // COLUNA 2: DETALHES, EXPLICAÇÃO E CÓDIGO INTERATIVO
            cols[1].vertical(|ui| {
                egui::ScrollArea::vertical()
                    .id_salt("rust_lib_detail_scroll")
                    .show(ui, |ui| {
                        egui::Frame::NONE
                            .fill(Color32::from_rgb(18, 22, 32))
                            .stroke(Stroke::new(1.0_f32, Color32::from_rgb(36, 46, 64)))
                            .corner_radius(CornerRadius::same(8))
                            .inner_margin(egui::Margin::same(14))
                            .show(ui, |ui| {
                                ui.horizontal(|ui| {
                                    draw_pill(ui, current_lib.category.label(), Color32::from_rgb(35, 75, 140));
                                    ui.add_space(4.0);
                                    ui.label(
                                        RichText::new(current_lib.name)
                                            .strong()
                                            .size(17.0)
                                            .color(Color32::WHITE),
                                    );
                                });

                                ui.add_space(4.0);
                                ui.label(
                                    RichText::new(current_lib.summary)
                                        .size(12.5)
                                        .color(Color32::from_rgb(180, 195, 220)),
                                );

                                ui.add_space(8.0);
                                ui.separator();
                                ui.add_space(8.0);

                                ui.label(
                                    RichText::new(current_lib.description)
                                        .size(13.0)
                                        .color(Color32::from_rgb(220, 228, 240)),
                                );

                                // Tipos e métodos principais
                                ui.add_space(10.0);
                                ui.label(RichText::new(self.tr("Principais Tipos & Métodos:")).strong().size(12.5).color(Color32::WHITE));
                                ui.horizontal_wrapped(|ui| {
                                    for t in current_lib.key_types {
                                        draw_pill(ui, t, Color32::from_rgb(45, 60, 85));
                                    }
                                });
                                ui.add_space(4.0);
                                ui.horizontal_wrapped(|ui| {
                                    for m in current_lib.key_methods {
                                        draw_pill(ui, m, Color32::from_rgb(28, 48, 68));
                                    }
                                });

                                // Se for crate externo, exibe a dependência Cargo.toml
                                if let Some(dep) = current_lib.cargo_dep {
                                    ui.add_space(10.0);
                                    ui.horizontal(|ui| {
                                        ui.label(RichText::new(self.tr("Dependência Cargo.toml:")).strong().size(12.5).color(Color32::WHITE));
                                        if ui.button(self.tr("📋 Copiar")).clicked() {
                                            ui.ctx().copy_text(dep.to_string());
                                            self.status_msg = self.tr("Dependência copiada para a área de transferência.");
                                            self.status_is_error = false;
                                        }
                                    });
                                    ui.add_space(2.0);
                                    ui.label(
                                        RichText::new(dep)
                                            .monospace()
                                            .size(11.5)
                                            .color(Color32::from_rgb(120, 215, 140)),
                                    );
                                }

                                ui.add_space(14.0);
                                ui.separator();
                                ui.add_space(8.0);

                                // Bloco de Código de Exemplo
                                ui.horizontal(|ui| {
                                    ui.label(RichText::new(self.tr("💻 Exemplo de Código Interativo:")).strong().size(13.5).color(Color32::WHITE));

                                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                        if ui
                                            .add(egui::Button::new(RichText::new(self.tr("▶ Testar no Editor")).strong()))
                                            .on_hover_text(self.tr("Carrega este código no Estúdio de Aprendizado e compila na hora."))
                                            .clicked()
                                        {
                                            self.learn_code_input = current_lib.example_code.to_string();
                                            self.learn_output = None;
                                            self.section = NavSection::LearnRust;
                                            self.status_msg = format!("{} {}", self.tr("Código carregado no editor:"), current_lib.name);
                                            self.status_is_error = false;
                                        }

                                        if ui.button(self.tr("📋 Copiar")).clicked() {
                                            ui.ctx().copy_text(current_lib.example_code.to_string());
                                            self.status_msg = self.tr("Exemplo de código copiado!");
                                            self.status_is_error = false;
                                        }

                                        if ui.button(self.tr("🌐 Docs ↗")).clicked() {
                                            let _ = crate::downloader::open_url(current_lib.docs_url);
                                        }
                                    });
                                });

                                ui.add_space(6.0);
                                egui::Frame::NONE
                                    .fill(Color32::from_rgb(12, 14, 20))
                                    .stroke(Stroke::new(1.0_f32, Color32::from_rgb(28, 36, 50)))
                                    .corner_radius(CornerRadius::same(6))
                                    .inner_margin(egui::Margin::same(10))
                                    .show(ui, |ui| {
                                        ui.set_width(ui.available_width());
                                        ui.label(
                                            RichText::new(current_lib.example_code)
                                                .monospace()
                                                .size(12.0)
                                                .color(Color32::from_rgb(220, 230, 245)),
                                        );
                                    });
                            });
                    });
            });
        });
    }
}
