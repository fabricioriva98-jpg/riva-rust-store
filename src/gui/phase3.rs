//! Blocos da interface da fase 3: jogos de outras lojas, atestado conferido pelo `gh`, Windows Sandbox,
//! tradução com IA e ferramentas de terminal em Rust.

use super::{draw_pill, BgEvent, StoreGui};
use crate::attestation::GhVerify;
use crate::db::{App, InstalledInfo};
use crate::{attestation, crates_io, launchers, sandbox, translator_ai};
use eframe::egui::{self, Color32, CornerRadius, RichText, Stroke};
use std::sync::Arc;

impl StoreGui {
    // ─── Jogos de outras lojas ─────────────────────────────────────────────

    pub(super) fn spawn_external_scan(&mut self) {
        if self.busy_task.is_some() {
            return;
        }
        self.busy_task = Some(self.tr("Sincronizando todas as lojas de games do PC..."));
        let events = Arc::clone(&self.bg_events);
        std::thread::spawn(move || {
            events.lock().unwrap().push(BgEvent::ExternalGames(launchers::all_games()));
        });
    }

    pub(super) fn render_external_section(&mut self, ui: &mut egui::Ui) {
        ui.add_space(14.0);
        ui.horizontal_wrapped(|ui| {
            ui.label(
                RichText::new(self.tr("🎮 Sincronização de Lojas de Games"))
                    .size(17.0)
                    .strong()
                    .color(Color32::WHITE),
            );
            let label = if self.external_games.is_some() {
                self.tr("🔄 Sincronizar Lojas")
            } else {
                self.tr("🔎 Sincronizar Lojas de Games")
            };
            if ui.add_enabled(self.busy_task.is_none(), egui::Button::new(label)).clicked() {
                self.spawn_external_scan();
            }
        });
        let Some(games) = self.external_games.clone() else {
            ui.label(
                RichText::new(self.tr(
                    "Sincroniza todos os seus jogos da Steam, Epic Games, GOG, Xbox, EA App, Ubisoft, Battle.net e Riot Games.",
                ))
                .size(12.0)
                .color(Color32::from_rgb(145, 155, 175)),
            );
            return;
        };
        if games.is_empty() {
            ui.label(RichText::new(self.tr("Nenhum jogo encontrado nas lojas instaladas no PC.")).color(Color32::GRAY));
            return;
        }

        // Filtro por Loja
        ui.add_space(6.0);
        ui.horizontal_wrapped(|ui| {
            ui.label(RichText::new(self.tr("Loja:")).size(12.0).color(Color32::GRAY));
            if ui.selectable_label(self.external_store_filter.is_none(), format!("{} ({})", self.tr("Todas"), games.len())).clicked() {
                self.external_store_filter = None;
            }

            let stores = [
                (launchers::Store::Steam, "Steam"),
                (launchers::Store::Epic, "Epic Games"),
                (launchers::Store::Gog, "GOG"),
                (launchers::Store::Xbox, "Xbox"),
                (launchers::Store::Ea, "EA App"),
                (launchers::Store::Ubisoft, "Ubisoft"),
                (launchers::Store::BattleNet, "Battle.net"),
                (launchers::Store::Riot, "Riot Games"),
            ];

            for (st, label) in stores {
                let count = games.iter().filter(|g| g.store == st).count();
                if count > 0 {
                    let is_sel = self.external_store_filter == Some(st);
                    if ui.selectable_label(is_sel, format!("{label} ({count})")).clicked() {
                        self.external_store_filter = Some(st);
                    }
                }
            }
        });

        ui.add_space(8.0);
        let filtered: Vec<&launchers::ExternalGame> = games
            .iter()
            .filter(|g| self.external_store_filter.is_none() || self.external_store_filter == Some(g.store))
            .collect();

        for g in filtered {
            ui.horizontal_wrapped(|ui| {
                draw_pill(ui, g.store.label(), Color32::from_rgb(52, 62, 82));
                ui.label(RichText::new(&g.name).strong().color(Color32::WHITE));
                if ui.small_button(self.tr("▶ Jogar")).clicked() {
                    (self.status_msg, self.status_is_error) = match launchers::launch(g) {
                        Ok(()) => (format!("▶ {} ({})", g.name, g.store.label()), false),
                        Err(e) => (format!("{e:#}"), true),
                    };
                }
            });
        }
    }

    // ─── Atestado conferido pelo GitHub CLI e Windows Sandbox ──────────────

    pub(super) fn render_security_extras(&mut self, ui: &mut egui::Ui, app: &App, inst: &InstalledInfo) {
        let path = self.installed_path(&inst.file_path);
        ui.horizontal_wrapped(|ui| {
            if app.attestation == crate::github::ATTESTATION_PRESENT
                && ui
                    .add_enabled(self.busy_task.is_none(), egui::Button::new(self.tr("🔏 Conferir atestado (gh)")))
                    .on_hover_text(self.tr(
                        "Usa o GitHub CLI para conferir a assinatura do atestado de build deste arquivo e o repositório de origem.",
                    ))
                    .clicked()
            {
                self.busy_task = Some("Conferindo atestado com o GitHub CLI...".into());
                let events = Arc::clone(&self.bg_events);
                let repo = app.full_name.clone();
                let file = std::path::PathBuf::from(&path);
                std::thread::spawn(move || {
                    let result = attestation::verify_with_gh(&file, &repo).map_err(|e| format!("{e:#}"));
                    events.lock().unwrap().push(BgEvent::GhVerified { repo, result });
                });
            }
            if self.sandbox_available
                && ui
                    .button(self.tr("🧪 Abrir no Windows Sandbox"))
                    .on_hover_text(self.tr(
                        "Abre o instalador numa máquina descartável do Windows; ao fechar o Sandbox, tudo o que rodou lá some.",
                    ))
                    .clicked()
            {
                (self.status_msg, self.status_is_error) =
                    match sandbox::open_in_sandbox(&self.base_dir(), std::path::Path::new(&path)) {
                        Ok(_) => (self.tr("🧪 Windows Sandbox aberto com o arquivo (pasta só para leitura)."), false),
                        Err(e) => (format!("{e:#}"), true),
                    };
            }
            if app.kind.as_deref() == Some("cli")
                && ui
                    .button(self.tr("⌨ Adicionar ao terminal"))
                    .on_hover_text(self.tr("Copia o executável para a pasta bin da RRS e coloca essa pasta no PATH do usuário."))
                    .clicked()
            {
                (self.status_msg, self.status_is_error) =
                    match crates_io::add_to_terminal(&self.base_dir(), std::path::Path::new(&path)) {
                        Ok(t) => (
                            format!("✔ {} {}", t.display(), self.tr("pronto; abra um terminal novo para usar.")),
                            false,
                        ),
                        Err(e) => (format!("{e:#}"), true),
                    };
            }
        });
        if let Some((repo, result)) = &self.gh_verify_status {
            if repo == &app.full_name {
                let (text, color) = match result {
                    Ok(GhVerify::Verified) => (
                        self.tr("✔ Atestado conferido pelo GitHub CLI: assinatura válida e arquivo gerado por este repositório."),
                        Color32::from_rgb(120, 230, 150),
                    ),
                    Ok(GhVerify::Failed(msg)) => (format!("✘ {} {msg}", self.tr("O GitHub CLI recusou o atestado:")), Color32::from_rgb(255, 120, 120)),
                    Ok(GhVerify::Unavailable(msg)) => (format!("⚠ {} {msg}", self.tr("Não conferido:")), Color32::from_rgb(235, 190, 85)),
                    Err(e) => (format!("⚠ {e}"), Color32::from_rgb(235, 190, 85)),
                };
                ui.label(RichText::new(text).size(11.5).color(color));
            }
        }
    }

    // ─── Tradução com IA ───────────────────────────────────────────────────

    pub(super) fn render_ai_translate(&mut self, ui: &mut egui::Ui) {
        let has_key = !self.anthropic_key_input.trim().is_empty();
        ui.horizontal_wrapped(|ui| {
            let label = if self.ai_busy {
                self.tr("⏳ Traduzindo com IA...")
            } else {
                self.tr("🤖 Traduzir com IA (Claude)")
            };
            let btn = ui
                .add_enabled(
                    has_key && !self.ai_busy,
                    egui::Button::new(RichText::new(label).strong()).fill(Color32::from_rgb(110, 70, 170)),
                )
                .on_hover_text(self.tr(
                    "Envia o código para a API do Claude com a sua chave (cada tradução é cobrada), compila com o rustc e devolve os erros ao modelo até compilar (até 4 tentativas).",
                ));
            if btn.clicked() {
                self.ai_busy = true;
                let key = self.anthropic_key_input.clone();
                let code = self.cpp_input.clone();
                let mode = self.translate_mode;
                let events = Arc::clone(&self.bg_events);
                self.rt.spawn(async move {
                    let res = translator_ai::translate_with_claude(&key, &code, mode, translator_ai::DEFAULT_ROUNDS)
                        .await
                        .map_err(|e| format!("{e:#}"));
                    events.lock().unwrap().push(BgEvent::AiTranslated(res));
                });
            }
            if !has_key {
                ui.label(
                    RichText::new(self.tr("Para usar, salve a chave da API do Claude em Configurações."))
                        .size(11.5)
                        .color(Color32::GRAY),
                );
            }
        });
    }

    pub(super) fn apply_ai_result(&mut self, res: Result<translator_ai::AiTranslation, String>) {
        self.ai_busy = false;
        match res {
            Ok(t) => {
                self.rust_output = t.code.clone();
                let mut notes = vec![format!(
                    "🤖 {} {} ({} {})",
                    self.tr("Traduzido com IA:"),
                    translator_ai::MODEL,
                    t.rounds,
                    self.tr("rodada(s)")
                )];
                if t.compiled {
                    notes.push(self.tr("✔ Compila no rustc (verificado pela RRS)."));
                } else {
                    notes.push(self.tr("✘ Não compilou depois de todas as tentativas; veja o diagnóstico abaixo."));
                }
                notes.push(format!("{} {}", self.tr("Ocorrências de unsafe:"), t.unsafe_count));
                self.translate_notes = notes;
                self.rustc_check_status = Some(match t.last_error {
                    None => Ok(self.tr("✔ Código Rust validado pelo rustc.")),
                    Some(e) => Err(e),
                });
                self.status_msg = self.tr("Tradução com IA concluída.");
                self.status_is_error = !t.compiled;
            }
            Err(e) => {
                self.status_msg = format!("{} {e}", self.tr("Tradução com IA falhou:"));
                self.status_is_error = true;
            }
        }
    }

    // ─── Ferramentas de terminal em Rust ───────────────────────────────────

    pub(super) fn render_crate_installer(&mut self, ui: &mut egui::Ui) {
        egui::Frame::NONE
            .fill(Color32::from_rgb(20, 28, 22))
            .stroke(Stroke::new(1.0_f32, Color32::from_rgb(50, 90, 60)))
            .corner_radius(CornerRadius::same(8))
            .inner_margin(egui::Margin::same(10))
            .show(ui, |ui| {
                ui.set_min_width(ui.available_width());
                ui.label(
                    RichText::new(self.tr("Adicionar ferramenta pelo nome do crate (crates.io)"))
                        .strong()
                        .color(Color32::WHITE),
                );
                ui.horizontal_wrapped(|ui| {
                    ui.add(
                        egui::TextEdit::singleline(&mut self.crate_input)
                            .hint_text("ripgrep, bat, fd-find, hyperfine...")
                            .desired_width(220.0),
                    );
                    if ui
                        .add_enabled(
                            self.busy_task.is_none() && crates_io::is_valid_crate_name(self.crate_input.trim()),
                            egui::Button::new(self.tr("Buscar no crates.io")),
                        )
                        .on_hover_text(self.tr(
                            "Descobre o repositório do crate e verifica a release como qualquer app da loja; baixa o binário pronto, sem compilar.",
                        ))
                        .clicked()
                    {
                        let name = self.crate_input.trim().to_string();
                        self.busy_task = Some(format!("Procurando {name} no crates.io..."));
                        let events = Arc::clone(&self.bg_events);
                        self.rt.spawn(async move {
                            let res = crates_io::crate_repository(&name).await.map_err(|e| format!("{e:#}"));
                            events.lock().unwrap().push(BgEvent::CrateResolved(res));
                        });
                    }
                });
            });
        ui.add_space(8.0);
    }
}
