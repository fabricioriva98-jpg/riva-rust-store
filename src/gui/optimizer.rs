//! Interface gráfica do Otimizador do PC e Diagnóstico de Sistema da RRS.
//! Exibe especificações de hardware (CPU, RAM, GPU, Discos), inventário completo de programas instalados,
//! programas de inicialização e ações de 1 clique para máxima performance em jogos e compilações Rust.

use super::{draw_pill, BgEvent, StoreGui};
use crate::brand;
use crate::optimizer;
use eframe::egui::{self, Color32, CornerRadius, RichText, Stroke};
use std::sync::Arc;

impl StoreGui {
    /// Inicia a varredura do sistema em segundo plano sem travar a interface.
    pub(super) fn spawn_optimizer_scan(&mut self) {
        if self.optimizer_is_scanning {
            return;
        }
        self.optimizer_is_scanning = true;
        self.busy_task = Some(self.tr("Analisando hardware, programas instalados e configurações..."));
        let events = Arc::clone(&self.bg_events);

        std::thread::spawn(move || {
            let specs = optimizer::collect_system_specs();
            let programs = optimizer::collect_installed_programs();
            let startup = optimizer::collect_startup_apps();
            let actions = optimizer::diagnose_optimizations();

            events.lock().unwrap().push(BgEvent::OptimizerScanFinished {
                specs,
                programs,
                startup,
                actions,
            });
        });
    }

    /// Executa uma ação de otimização de 1 clique em segundo plano.
    pub(super) fn spawn_optimizer_action(&mut self, action_id: &'static str) {
        self.busy_task = Some(self.tr("Aplicando otimização no sistema..."));
        let events = Arc::clone(&self.bg_events);

        std::thread::spawn(move || {
            let res = optimizer::apply_optimization(action_id).map_err(|e| e.to_string());
            events.lock().unwrap().push(BgEvent::OptimizerActionFinished(res));
        });
    }

    /// Renderiza a tela completa do Otimizador do PC.
    pub(super) fn render_optimizer(&mut self, ui: &mut egui::Ui) {
        if self.optimizer_specs.is_none() && !self.optimizer_is_scanning {
            self.spawn_optimizer_scan();
        }

        // ─── CABEÇALHO DO OTIMIZADOR ───────────────────────────────────────
        ui.horizontal(|ui| {
            brand::logo_badge(ui, 18.0);
            ui.add_space(6.0);
            ui.vertical(|ui| {
                ui.horizontal(|ui| {
                    ui.label(
                        RichText::new(self.tr("⚡ Otimizador do PC & Diagnóstico de Sistema"))
                            .size(20.0)
                            .strong()
                            .color(Color32::WHITE),
                    );
                    draw_pill(ui, &self.tr("RRS Hardware & Gaming Engine"), Color32::from_rgb(220, 105, 30));
                });
                ui.label(
                    RichText::new(self.tr(
                        "Reconhecimento completo de hardware, inventário de programas e ajustes seguros para máximo FPS e produtividade.",
                    ))
                    .size(12.5)
                    .color(Color32::from_rgb(145, 160, 185)),
                );
            });

            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                if ui
                    .add_enabled(
                        !self.optimizer_is_scanning,
                        egui::Button::new(RichText::new(self.tr("🔄 Atualizar Diagnóstico")).size(12.5)),
                    )
                    .clicked()
                {
                    self.spawn_optimizer_scan();
                }
            });
        });

        ui.add_space(10.0);

        // Se estiver escaneando e ainda não tiver dados
        if self.optimizer_specs.is_none() && self.optimizer_is_scanning {
            ui.vertical_centered(|ui| {
                ui.add_space(40.0);
                ui.spinner();
                ui.add_space(10.0);
                ui.label(
                    RichText::new(self.tr("Examinando processador, memória, placa de vídeo e programas instalados..."))
                        .size(14.0)
                        .color(Color32::from_rgb(170, 185, 210)),
                );
            });
            return;
        }

        egui::ScrollArea::vertical()
            .id_salt("optimizer_main_scroll")
            .show(ui, |ui| {
                // ─── ABAS SUPERIORES ───────────────────────────────────────
                ui.horizontal(|ui| {
                    let prog_count = self.optimizer_programs.as_ref().map(|p| p.len()).unwrap_or(0);
                    let start_count = self.optimizer_startup.as_ref().map(|s| s.len()).unwrap_or(0);

                    if ui.selectable_label(self.optimizer_tab == 0, self.tr("🖥 Visão Geral & Hardware")).clicked() {
                        self.optimizer_tab = 0;
                    }
                    if ui.selectable_label(self.optimizer_tab == 1, format!("{} ({prog_count})", self.tr("💻 Programas Instalados"))).clicked() {
                        self.optimizer_tab = 1;
                    }
                    if ui.selectable_label(self.optimizer_tab == 2, format!("{} ({start_count})", self.tr("🚀 Inicialização do Windows"))).clicked() {
                        self.optimizer_tab = 2;
                    }
                });

                ui.add_space(8.0);
                ui.separator();
                ui.add_space(10.0);

                match self.optimizer_tab {
                    0 => self.render_optimizer_overview(ui),
                    1 => self.render_optimizer_installed_programs(ui),
                    2 => self.render_optimizer_startup_apps(ui),
                    _ => {}
                }
            });
    }

    /// Renderiza os cartões de Hardware e as Ações de Otimização recomendadas.
    fn render_optimizer_overview(&mut self, ui: &mut egui::Ui) {
        if let Some(specs) = self.optimizer_specs.clone() {
            ui.label(RichText::new(self.tr("📊 Especificações de Hardware Detectadas")).strong().size(15.0).color(Color32::WHITE));
            ui.add_space(6.0);

            // GRID DE 4 CARTÕES DE HARDWARE
            egui::Grid::new("hw_specs_grid")
                .num_columns(2)
                .spacing([14.0, 14.0])
                .show(ui, |ui| {
                    // Cartão 1: Processador (CPU)
                    egui::Frame::NONE
                        .fill(Color32::from_rgb(22, 26, 36))
                        .stroke(Stroke::new(1.0_f32, Color32::from_rgb(40, 50, 70)))
                        .corner_radius(CornerRadius::same(8))
                        .inner_margin(egui::Margin::same(12))
                        .show(ui, |ui| {
                            ui.set_width(360.0);
                            ui.horizontal(|ui| {
                                draw_pill(ui, "CPU", Color32::from_rgb(180, 50, 40));
                                ui.label(RichText::new(self.tr("Processador")).strong().color(Color32::WHITE));
                            });
                            ui.add_space(4.0);
                            ui.label(RichText::new(&specs.cpu_name).strong().size(14.0).color(Color32::WHITE));
                            ui.add_space(2.0);
                            ui.label(
                                RichText::new(format!(
                                    "{} {}, {} {}",
                                    specs.cpu_cores,
                                    self.tr("núcleos físicos"),
                                    specs.cpu_threads,
                                    self.tr("threads lógicas")
                                ))
                                .size(12.0)
                                .color(Color32::from_rgb(150, 165, 190)),
                            );
                        });

                    // Cartão 2: Memória RAM
                    egui::Frame::NONE
                        .fill(Color32::from_rgb(22, 26, 36))
                        .stroke(Stroke::new(1.0_f32, Color32::from_rgb(40, 50, 70)))
                        .corner_radius(CornerRadius::same(8))
                        .inner_margin(egui::Margin::same(12))
                        .show(ui, |ui| {
                            ui.set_width(360.0);
                            ui.horizontal(|ui| {
                                draw_pill(ui, "RAM", Color32::from_rgb(40, 110, 180));
                                ui.label(RichText::new(self.tr("Memória do Sistema")).strong().color(Color32::WHITE));
                            });
                            ui.add_space(4.0);
                            ui.label(RichText::new(format!("{:.1} GB Total", specs.ram_total_gb)).strong().size(14.0).color(Color32::WHITE));
                            ui.add_space(2.0);
                            ui.label(
                                RichText::new(format!(
                                    "{:.1} GB {} ({:.1} GB {})",
                                    specs.ram_free_gb,
                                    self.tr("livres atualmente"),
                                    specs.ram_total_gb - specs.ram_free_gb,
                                    self.tr("em uso")
                                ))
                                .size(12.0)
                                .color(Color32::from_rgb(150, 165, 190)),
                            );
                        });

                    ui.end_row();

                    // Cartão 3: Placa de Vídeo (GPU)
                    egui::Frame::NONE
                        .fill(Color32::from_rgb(22, 26, 36))
                        .stroke(Stroke::new(1.0_f32, Color32::from_rgb(40, 50, 70)))
                        .corner_radius(CornerRadius::same(8))
                        .inner_margin(egui::Margin::same(12))
                        .show(ui, |ui| {
                            ui.set_width(360.0);
                            ui.horizontal(|ui| {
                                draw_pill(ui, "GPU", Color32::from_rgb(40, 150, 90));
                                ui.label(RichText::new(self.tr("Placa de Vídeo")).strong().color(Color32::WHITE));
                            });
                            ui.add_space(4.0);
                            ui.label(RichText::new(&specs.gpu_name).strong().size(14.0).color(Color32::WHITE));
                            ui.add_space(2.0);
                            ui.label(
                                RichText::new(format!("{}: {}", self.tr("Driver"), specs.gpu_driver))
                                    .size(12.0)
                                    .color(Color32::from_rgb(150, 165, 190)),
                            );
                        });

                    // Cartão 4: Discos Locais
                    egui::Frame::NONE
                        .fill(Color32::from_rgb(22, 26, 36))
                        .stroke(Stroke::new(1.0_f32, Color32::from_rgb(40, 50, 70)))
                        .corner_radius(CornerRadius::same(8))
                        .inner_margin(egui::Margin::same(12))
                        .show(ui, |ui| {
                            ui.set_width(360.0);
                            ui.horizontal(|ui| {
                                draw_pill(ui, "DISCOS", Color32::from_rgb(160, 110, 40));
                                ui.label(RichText::new(self.tr("Armazenamento")).strong().color(Color32::WHITE));
                            });
                            ui.add_space(4.0);
                            for d in &specs.disks {
                                ui.horizontal(|ui| {
                                    ui.label(RichText::new(format!("Unidade {}", d.drive)).strong().size(12.5).color(Color32::WHITE));
                                    ui.label(
                                        RichText::new(format!("{:.0} GB livres de {:.0} GB ({:.0}% usado)", d.free_gb, d.total_gb, d.used_pct))
                                            .size(11.5)
                                            .color(Color32::from_rgb(160, 175, 195)),
                                    );
                                });
                            }
                        });

                    ui.end_row();
                });
        }

        ui.add_space(18.0);
        ui.separator();
        ui.add_space(14.0);

        // ─── RECOMENDAÇÕES DE OTIMIZAÇÃO ──────────────────────────────────
        ui.horizontal(|ui| {
            ui.label(RichText::new(self.tr("🚀 Otimizações Recomendadas para seu Computador")).strong().size(16.0).color(Color32::WHITE));
            draw_pill(ui, &self.tr("Ações de 1 Clique"), Color32::from_rgb(40, 130, 80));
        });
        ui.add_space(4.0);
        ui.label(
            RichText::new(self.tr(
                "Ajustes seguros e testados para extrair o máximo desempenho da CPU/GPU em jogos e no desenvolvimento com Rust.",
            ))
            .size(12.5)
            .color(Color32::from_rgb(150, 165, 190)),
        );

        ui.add_space(10.0);

        let actions = self.optimizer_actions.clone();
        for item in actions {
            egui::Frame::NONE
                .fill(Color32::from_rgb(18, 22, 32))
                .stroke(Stroke::new(
                    1.0_f32,
                    if item.is_optimal {
                        Color32::from_rgb(34, 46, 62)
                    } else {
                        Color32::from_rgb(80, 120, 60)
                    },
                ))
                .corner_radius(CornerRadius::same(8))
                .inner_margin(egui::Margin::same(14))
                .show(ui, |ui| {
                    ui.set_min_width(ui.available_width());
                    ui.horizontal(|ui| {
                        draw_pill(ui, item.category, Color32::from_rgb(40, 50, 75));
                        let impact_str = format!("{}: {}", self.tr("Impacto"), item.impact);
                        draw_pill(
                            ui,
                            &impact_str,
                            if item.impact.contains("Alto") {
                                Color32::from_rgb(180, 50, 40)
                            } else {
                                Color32::from_rgb(40, 110, 70)
                            },
                        );

                        ui.add_space(6.0);
                        ui.label(RichText::new(item.title).strong().size(14.5).color(Color32::WHITE));
                    });

                    ui.add_space(6.0);
                    ui.label(RichText::new(item.description).size(12.5).color(Color32::from_rgb(180, 192, 212)));

                    ui.add_space(6.0);
                    ui.horizontal(|ui| {
                        ui.label(RichText::new(format!("{}: ", self.tr("Estado Atual"))).size(12.0).color(Color32::from_rgb(140, 150, 170)));
                        ui.label(RichText::new(&item.current_status).strong().size(12.0).color(Color32::from_rgb(220, 230, 245)));

                        ui.add_space(12.0);
                        ui.label(RichText::new(format!("{}: ", self.tr("Recomendação"))).size(12.0).color(Color32::from_rgb(140, 150, 170)));
                        ui.label(RichText::new(&item.recommended_status).strong().size(12.0).color(Color32::from_rgb(110, 210, 140)));
                    });

                    // Ação em linha própria: alinhado à direita dentro da linha do título, o botão
                    // não recebia clique nem mouse por cima (defeito da versão anterior)
                    ui.add_space(8.0);
                    ui.horizontal(|ui| {
                        if item.is_optimal {
                            draw_pill(ui, &self.tr("✔ Configurado / Otimizado"), Color32::from_rgb(35, 110, 60));
                        } else if ui
                            .add_enabled(
                                self.busy_task.is_none() && self.optimizer_confirm.is_none(),
                                egui::Button::new(RichText::new(self.tr("⚡ Otimizar Agora")).strong())
                                    .fill(Color32::from_rgb(35, 130, 60)),
                            )
                            .on_hover_text(self.tr("Aplica esta recomendação no Windows."))
                            .clicked()
                        {
                            if item.confirm.is_some() {
                                self.optimizer_confirm = Some(item.id);
                            } else {
                                self.spawn_optimizer_action(item.id);
                            }
                        }
                    });

                    // Ação que apaga arquivos: segundo clique, com a quantidade à vista
                    if let (Some(question), Some(id)) = (&item.confirm, self.optimizer_confirm) {
                        if id == item.id {
                            ui.add_space(6.0);
                            ui.horizontal_wrapped(|ui| {
                                ui.label(RichText::new(format!("⚠ {question}")).strong().color(Color32::from_rgb(235, 190, 85)));
                                if ui
                                    .add(egui::Button::new(self.tr("Sim, apagar")).fill(Color32::from_rgb(150, 50, 40)))
                                    .clicked()
                                {
                                    self.optimizer_confirm = None;
                                    self.spawn_optimizer_action(item.id);
                                }
                                if ui.button(self.tr("Cancelar")).clicked() {
                                    self.optimizer_confirm = None;
                                }
                            });
                        }
                    }
                });

            ui.add_space(8.0);
        }
    }

    /// Renderiza a lista pesquisável de todos os programas instalados no Windows.
    fn render_optimizer_installed_programs(&mut self, ui: &mut egui::Ui) {
        let hint = self.tr("Filtrar programas instalados...");
        ui.horizontal(|ui| {
            ui.label(RichText::new(self.tr("💻 Inventário de Softwares Instalados no PC")).strong().size(16.0).color(Color32::WHITE));
            ui.add_space(10.0);
            ui.add(
                egui::TextEdit::singleline(&mut self.optimizer_program_search)
                    .hint_text(hint)
                    .desired_width(260.0),
            );
        });

        ui.add_space(8.0);

        let Some(programs) = &self.optimizer_programs else {
            ui.label(self.tr("Nenhum programa carregado."));
            return;
        };

        let query = self.optimizer_program_search.trim().to_lowercase();
        let filtered: Vec<&optimizer::InstalledProgram> = programs
            .iter()
            .filter(|p| {
                if query.is_empty() {
                    true
                } else {
                    p.name.to_lowercase().contains(&query)
                        || p.publisher.as_deref().unwrap_or("").to_lowercase().contains(&query)
                }
            })
            .collect();

        ui.label(
            RichText::new(format!(
                "{} {} {} ({} {})",
                self.tr("Exibindo"),
                filtered.len(),
                self.tr("programas"),
                programs.len(),
                self.tr("totais no registro")
            ))
            .size(12.0)
            .color(Color32::from_rgb(145, 160, 180)),
        );

        ui.add_space(8.0);

        egui::Frame::NONE
            .fill(Color32::from_rgb(16, 20, 28))
            .stroke(Stroke::new(1.0_f32, Color32::from_rgb(32, 40, 56)))
            .corner_radius(CornerRadius::same(6))
            .inner_margin(egui::Margin::same(10))
            .show(ui, |ui| {
                ui.horizontal(|ui| {
                    ui.label(RichText::new(self.tr("NOME DO PROGRAMA")).strong().size(12.0).color(Color32::from_rgb(130, 145, 170)));
                    ui.add_space(260.0);
                    ui.label(RichText::new(self.tr("VERSÃO")).strong().size(12.0).color(Color32::from_rgb(130, 145, 170)));
                    ui.add_space(120.0);
                    ui.label(RichText::new(self.tr("DESENVOLVEDOR")).strong().size(12.0).color(Color32::from_rgb(130, 145, 170)));
                });
                ui.separator();

                for p in filtered.iter().take(200) {
                    ui.horizontal(|ui| {
                        ui.set_min_height(22.0);
                        ui.label(RichText::new(&p.name).strong().size(12.5).color(Color32::WHITE));

                        if let Some(v) = &p.version {
                            ui.label(RichText::new(v).size(11.5).color(Color32::from_rgb(150, 170, 200)));
                        }

                        if let Some(publ) = &p.publisher {
                            ui.label(RichText::new(format!("• {publ}")).size(11.5).color(Color32::from_rgb(130, 145, 165)));
                        }
                    });
                }
            });
    }

    /// Renderiza os programas configurados para iniciar com o Windows.
    fn render_optimizer_startup_apps(&mut self, ui: &mut egui::Ui) {
        ui.label(RichText::new(self.tr("🚀 Aplicativos que Iniciam com o Windows")).strong().size(16.0).color(Color32::WHITE));
        ui.add_space(4.0);
        ui.label(
            RichText::new(self.tr(
                "Programas que iniciam automaticamente consomem memória RAM e prolongam o tempo de boot do computador.",
            ))
            .size(12.5)
            .color(Color32::from_rgb(150, 165, 190)),
        );

        ui.add_space(10.0);

        let Some(apps) = &self.optimizer_startup else {
            ui.label(self.tr("Nenhum aplicativo de inicialização detectado."));
            return;
        };

        for app in apps {
            egui::Frame::NONE
                .fill(Color32::from_rgb(20, 24, 34))
                .stroke(Stroke::new(1.0_f32, Color32::from_rgb(36, 44, 62)))
                .corner_radius(CornerRadius::same(6))
                .inner_margin(egui::Margin::same(10))
                .show(ui, |ui| {
                    ui.horizontal(|ui| {
                        draw_pill(ui, &app.scope, Color32::from_rgb(45, 60, 90));
                        ui.add_space(4.0);
                        ui.label(RichText::new(&app.name).strong().size(13.5).color(Color32::WHITE));
                    });
                    ui.add_space(2.0);
                    ui.label(
                        RichText::new(&app.command)
                            .monospace()
                            .size(11.0)
                            .color(Color32::from_rgb(130, 145, 170)),
                    );
                });
            ui.add_space(6.0);
        }
    }
}
