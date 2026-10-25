//! Blocos da interface da fase 2: apps instalados no Windows, backup de saves e busca no winget.

use super::{draw_pill, friendly_title, BgEvent, StoreGui};
use crate::db::{App, InstalledInfo};
use crate::{saves, windows_apps, winget};
use eframe::egui::{self, Color32, CornerRadius, RichText, Stroke};
use std::path::PathBuf;
use std::sync::Arc;

impl StoreGui {
    // ─── Apps instalados no Windows fora da RRS ────────────────────────────

    pub(super) fn spawn_windows_scan(&mut self) {
        if self.busy_task.is_some() {
            return;
        }
        self.busy_task = Some("Procurando apps instalados no Windows...".into());
        let events = Arc::clone(&self.bg_events);
        std::thread::spawn(move || {
            let programs = windows_apps::list_installed_programs();
            events.lock().unwrap().push(BgEvent::WindowsScan(programs));
        });
    }

    pub(super) fn refresh_windows_matches(&mut self) {
        if let Some(programs) = &self.windows_programs {
            self.windows_matches = windows_apps::match_catalog(&self.apps, programs);
        }
    }

    pub(super) fn render_windows_section(&mut self, ui: &mut egui::Ui) {
        ui.add_space(14.0);
        ui.horizontal_wrapped(|ui| {
            ui.label(
                RichText::new(self.tr("🖥 Instalados no Windows fora da RRS"))
                    .size(17.0)
                    .strong()
                    .color(Color32::WHITE),
            );
            let label = if self.windows_programs.is_some() {
                self.tr("🔄 Procurar de novo")
            } else {
                self.tr("🔎 Procurar apps instalados no Windows")
            };
            if ui.add_enabled(self.busy_task.is_none(), egui::Button::new(label)).clicked() {
                self.spawn_windows_scan();
            }
        });
        let Some(programs) = &self.windows_programs else {
            ui.label(
                RichText::new(self.tr(
                    "Lê a lista de programas do Windows e mostra os que também estão no catálogo, com aviso de versão nova.",
                ))
                .size(12.0)
                .color(Color32::from_rgb(145, 155, 175)),
            );
            return;
        };
        ui.label(
            RichText::new(format!(
                "{} {} · {} {}",
                programs.len(),
                self.tr("programas no Windows"),
                self.windows_matches.len(),
                self.tr("reconhecidos no catálogo")
            ))
            .size(12.0)
            .color(Color32::from_rgb(145, 155, 175)),
        );
        ui.add_space(6.0);
        let matches = self.windows_matches.clone();
        for m in &matches {
            let latest = self
                .apps
                .iter()
                .find(|a| a.full_name == m.full_name)
                .map(|a| a.latest_tag.clone())
                .unwrap_or_default();
            egui::Frame::NONE
                .fill(Color32::from_rgb(24, 28, 37))
                .stroke(Stroke::new(1.0_f32, Color32::from_rgb(38, 44, 56)))
                .corner_radius(CornerRadius::same(6))
                .inner_margin(egui::Margin::symmetric(10, 6))
                .show(ui, |ui| {
                    ui.set_min_width(ui.available_width());
                    ui.horizontal_wrapped(|ui| {
                        ui.label(RichText::new(friendly_title(&m.full_name)).strong().color(Color32::WHITE));
                        ui.label(
                            RichText::new(format!(
                                "{} {} · {} {}",
                                self.tr("no PC:"),
                                if m.program.version.is_empty() { "?" } else { &m.program.version },
                                self.tr("catálogo:"),
                                latest
                            ))
                            .size(11.5)
                            .color(Color32::GRAY),
                        );
                        match m.outdated {
                            Some(true) => draw_pill(ui, &self.tr("⬆ versão nova"), Color32::from_rgb(200, 120, 20)),
                            Some(false) => draw_pill(ui, &self.tr("✔ em dia"), Color32::from_rgb(30, 125, 75)),
                            None => {}
                        }
                        if ui.small_button(self.tr("Ver na loja")).clicked() {
                            let name = m.full_name.clone();
                            self.select_app(&name);
                        }
                    });
                });
            ui.add_space(4.0);
        }
    }

    // ─── Backup de saves ───────────────────────────────────────────────────

    pub(super) fn load_save_dir_input(&mut self, full_name: &str) {
        self.restore_confirm = None;
        self.save_dir_input = crate::db::open(&self.db_path)
            .ok()
            .and_then(|c| saves::save_dir(&c, full_name))
            .unwrap_or_default();
    }

    pub(super) fn render_saves_block(&mut self, ui: &mut egui::Ui, app: &App, _inst: &InstalledInfo) {
        ui.add_space(8.0);
        egui::Frame::NONE
            .fill(Color32::from_rgb(19, 23, 31))
            .stroke(Stroke::new(1.0_f32, Color32::from_rgb(45, 60, 82)))
            .corner_radius(CornerRadius::same(6))
            .inner_margin(egui::Margin::same(10))
            .show(ui, |ui| {
                ui.label(RichText::new(self.tr("💾 Saves e configurações")).size(13.0).strong());
                ui.label(
                    RichText::new(self.tr(
                        "Aponte a pasta onde o jogo guarda os saves. A RRS faz uma cópia compactada quando o jogo fecha.",
                    ))
                    .size(11.0)
                    .color(Color32::GRAY),
                );
                ui.horizontal_wrapped(|ui| {
                    ui.add(
                        egui::TextEdit::singleline(&mut self.save_dir_input)
                            .hint_text("C:\\Users\\...\\Saved Games\\...")
                            .desired_width(230.0),
                    );
                    if ui.small_button(self.tr("Salvar pasta")).clicked() {
                        let res = crate::db::open(&self.db_path)
                            .and_then(|c| saves::set_save_dir(&c, &app.full_name, &self.save_dir_input));
                        (self.status_msg, self.status_is_error) = match res {
                            Ok(()) => (self.tr("✔ Pasta de saves guardada."), false),
                            Err(e) => (format!("{e:#}"), true),
                        };
                    }
                    let has_dir = !self.save_dir_input.trim().is_empty();
                    if ui
                        .add_enabled(has_dir, egui::Button::new(self.tr("Fazer backup agora")).small())
                        .clicked()
                    {
                        let dir = PathBuf::from(self.save_dir_input.trim());
                        (self.status_msg, self.status_is_error) =
                            match saves::backup(&self.base_dir(), &app.full_name, &dir, None) {
                                Ok(zip) => (format!("✔ Backup salvo: {}", zip.display()), false),
                                Err(e) => (format!("Backup não feito: {e:#}"), true),
                            };
                        if !self.status_is_error {
                            self.record_stat(crate::achievements::STAT_SAVES);
                        }
                    }
                });

                let backups = saves::list_backups(&self.base_dir(), &app.full_name);
                if backups.is_empty() {
                    return;
                }
                ui.label(
                    RichText::new(format!("{} {}", backups.len(), self.tr("backup(s); os mais recentes:")))
                        .size(11.0)
                        .color(Color32::GRAY),
                );
                for zip in backups.iter().take(3) {
                    ui.horizontal(|ui| {
                        let name = zip.file_name().map(|n| n.to_string_lossy().to_string()).unwrap_or_default();
                        ui.label(RichText::new(&name).monospace().size(11.0));
                        let confirming = self.restore_confirm.as_ref() == Some(zip);
                        let label = if confirming { self.tr("⚠ Confirmar") } else { self.tr("Restaurar") };
                        if ui
                            .add_enabled(!self.save_dir_input.trim().is_empty(), egui::Button::new(label).small())
                            .on_hover_text(self.tr(
                                "Substitui os saves atuais pelos do backup. Antes, guarda os atuais num backup \"antes-de-restaurar\".",
                            ))
                            .clicked()
                        {
                            if !confirming {
                                self.restore_confirm = Some(zip.clone());
                            } else {
                                self.restore_confirm = None;
                                let dir = PathBuf::from(self.save_dir_input.trim());
                                (self.status_msg, self.status_is_error) =
                                    match saves::restore(&self.base_dir(), &app.full_name, zip, &dir) {
                                        Ok(_) => (format!("✔ Saves restaurados de {name}"), false),
                                        Err(e) => (format!("Restauração não feita: {e:#}"), true),
                                    };
                            }
                        }
                    });
                }
            });
    }

    // ─── Busca no winget ───────────────────────────────────────────────────

    pub(super) fn spawn_winget_search(&mut self, query: String) {
        if self.busy_task.is_some() {
            return;
        }
        self.busy_task = Some(format!("Buscando \"{query}\" no winget..."));
        let events = Arc::clone(&self.bg_events);
        std::thread::spawn(move || {
            let res = winget::search(&query).map_err(|e| format!("{e:#}"));
            events.lock().unwrap().push(BgEvent::WingetResults { query, packages: res });
        });
    }

    pub(super) fn render_winget_section(&mut self, ui: &mut egui::Ui) {
        let query = self.search.trim().to_string();
        if query.is_empty() {
            return;
        }
        ui.add_space(14.0);
        ui.horizontal_wrapped(|ui| {
            ui.label(RichText::new(self.tr("📦 Também no winget")).size(16.0).strong().color(Color32::WHITE));
            let already = self.winget_query == query && self.winget_results.is_some();
            if !already
                && ui
                    .add_enabled(self.busy_task.is_none(), egui::Button::new(self.tr("Buscar no winget")))
                    .on_hover_text(self.tr("Procura o termo no catálogo do winget, o gerenciador de pacotes do Windows."))
                    .clicked()
            {
                self.spawn_winget_search(query.clone());
            }
        });
        ui.label(
            RichText::new(self.tr(
                "O winget baixa e confere o SHA-256 do manifesto da Microsoft. Esses pacotes não passam pela auditoria de precedentes da RRS.",
            ))
            .size(11.5)
            .color(Color32::from_rgb(240, 195, 85)),
        );
        if self.winget_query != query {
            return;
        }
        match self.winget_results.clone() {
            None => {}
            Some(Err(e)) => {
                ui.label(RichText::new(e).color(Color32::from_rgb(255, 120, 120)));
            }
            Some(Ok(list)) if list.is_empty() => {
                ui.label(RichText::new(self.tr("Nada encontrado no winget.")).color(Color32::GRAY));
            }
            Some(Ok(list)) => {
                for p in list.iter().take(15) {
                    ui.horizontal_wrapped(|ui| {
                        ui.label(RichText::new(&p.name).strong());
                        ui.label(RichText::new(&p.id).monospace().size(11.0).color(Color32::GRAY));
                        ui.label(RichText::new(&p.version).size(11.0).color(Color32::GRAY));
                        let confirming = self.winget_confirm.as_deref() == Some(p.id.as_str());
                        let label = if confirming {
                            self.tr("⚠ Confirmar instalação")
                        } else {
                            self.tr("Instalar via winget")
                        };
                        if ui.small_button(label).clicked() {
                            if !confirming {
                                self.winget_confirm = Some(p.id.clone());
                            } else {
                                self.winget_confirm = None;
                                (self.status_msg, self.status_is_error) = match winget::install(&p.id) {
                                    Ok(()) => (format!("▶ winget install {} aberto numa janela de console.", p.id), false),
                                    Err(e) => (format!("{e:#}"), true),
                                };
                            }
                        }
                    });
                }
            }
        }
    }
}
