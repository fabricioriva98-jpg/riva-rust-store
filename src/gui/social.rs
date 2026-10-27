//! Central de bate-papo (GitHub Discussions) e avisos.

use super::{draw_pill, friendly_title, BgEvent, StoreGui};
use crate::db::App;
use crate::{community, social};
use eframe::egui::{self, Color32, CornerRadius, RichText, Stroke, Vec2};
use std::sync::Arc;

/// Avatar do GitHub no tamanho pedido (o endereço pode ou não ter `?`).
pub(super) fn sized_avatar(url: &str, size: u32) -> String {
    let sep = if url.contains('?') { '&' } else { '?' };
    format!("{url}{sep}s={size}")
}

impl StoreGui {
    /// Avisos no canto da tela por 6 segundos.
    pub(super) fn render_toasts(&mut self, ctx: &egui::Context) {
        let now = ctx.input(|i| i.time);
        for t in &mut self.toasts {
            t.1.get_or_insert(now);
        }
        self.toasts.retain(|(_, start)| start.is_none_or(|s| now - s < 6.0));
        if self.toasts.is_empty() {
            return;
        }
        ctx.request_repaint_after(std::time::Duration::from_millis(250));
        egui::Area::new(egui::Id::new("avisos_conquista"))
            .anchor(egui::Align2::RIGHT_BOTTOM, egui::vec2(-20.0, -40.0))
            .order(egui::Order::Foreground)
            .show(ctx, |ui| {
                for (text, _) in &self.toasts {
                    egui::Frame::NONE
                        .fill(Color32::from_rgb(40, 34, 12))
                        .stroke(Stroke::new(1.5_f32, Color32::from_rgb(230, 180, 60)))
                        .corner_radius(CornerRadius::same(8))
                        .inner_margin(egui::Margin::symmetric(14, 10))
                        .show(ui, |ui| {
                            ui.label(RichText::new(text).size(14.0).strong().color(Color32::from_rgb(255, 225, 140)));
                        });
                    ui.add_space(6.0);
                }
            });
    }

    // ─── Perfil ────────────────────────────────────────────────────────────

    /// Cartão pequeno no menu lateral: foto, nome da conta; abre as Configurações.
    pub(super) fn render_profile_chip(&mut self, ui: &mut egui::Ui) {
        let name = self
            .github_user
            .as_ref()
            .map(|u| format!("@{}", u.login))
            .unwrap_or_else(|| self.tr("Entrar com o GitHub"));
        let resp = egui::Frame::NONE
            .fill(Color32::from_rgb(26, 31, 42))
            .corner_radius(CornerRadius::same(8))
            .inner_margin(egui::Margin::symmetric(8, 6))
            .show(ui, |ui| {
                ui.set_min_width(ui.available_width());
                ui.horizontal(|ui| {
                    match self.github_user.as_ref().filter(|u| !u.avatar_url.is_empty()) {
                        Some(u) => {
                            ui.add(
                                egui::Image::new(sized_avatar(&u.avatar_url, 48))
                                    .fit_to_exact_size(Vec2::new(24.0, 24.0))
                                    .corner_radius(CornerRadius::same(12)),
                            );
                        }
                        None => {
                            ui.label(RichText::new("👤").size(18.0));
                        }
                    }
                    ui.vertical(|ui| {
                        ui.label(RichText::new(&name).size(12.0).strong().color(Color32::WHITE));
                        let sub = if self.github_user.is_some() {
                            self.tr("✔ Conectado")
                        } else {
                            self.tr("Conta do GitHub")
                        };
                        ui.label(RichText::new(sub).size(10.5).color(Color32::from_rgb(140, 160, 190)));
                    });
                });
            })
            .response;
        if super::clickable_card(resp, &name).clicked() {
            self.section = super::NavSection::Settings;
        }
    }

    // ─── Central de bate-papo ──────────────────────────────────────────────

    pub(super) fn open_community_for(&mut self, repo: Option<String>) {
        self.section = super::NavSection::Community;
        self.community_target = repo;
        self.community_list = None;
        self.community_thread = None;
    }

    fn community_repo_in_view(&self) -> Option<String> {
        self.community_target
            .clone()
            .or_else(|| (!self.community_repo.trim().is_empty()).then(|| self.community_repo.trim().to_string()))
    }

    fn spawn_community_list(&mut self, repo: String) {
        let token = self.auth_token.clone().unwrap_or_default();
        self.busy_task = Some(format!("Carregando conversas de {repo}..."));
        let events = Arc::clone(&self.bg_events);
        self.rt.spawn(async move {
            let res = community::list_discussions(&token, &repo).await.map_err(|e| format!("{e:#}"));
            events.lock().unwrap().push(BgEvent::CommunityList { repo, result: res });
        });
    }

    fn spawn_community_thread(&mut self, repo: String, number: u64) {
        let token = self.auth_token.clone().unwrap_or_default();
        self.busy_task = Some("Abrindo conversa...".into());
        let events = Arc::clone(&self.bg_events);
        self.rt.spawn(async move {
            let res = community::get_thread(&token, &repo, number).await.map_err(|e| format!("{e:#}"));
            events.lock().unwrap().push(BgEvent::CommunityThread(res));
        });
    }

    pub(super) fn render_community_section(&mut self, ui: &mut egui::Ui) {
        if !self.community_visited {
            self.community_visited = true;
        }
        ui.label(RichText::new(self.tr("💬 Central de bate-papo")).size(21.0).strong().color(Color32::WHITE));
        ui.label(
            RichText::new(self.tr(
                "Conversas no GitHub Discussions. Ler é aqui; responder e abrir conversa nova é no site do GitHub.",
            ))
            .size(12.0)
            .color(Color32::from_rgb(150, 160, 180)),
        );
        ui.add_space(8.0);

        // Abas: comunidade geral e o app selecionado
        let selected_repo = self
            .selected_repo
            .clone()
            .filter(|r| !crate::forge::is_other_forge(r));
        ui.horizontal_wrapped(|ui| {
            let general = self.community_target.is_none();
            if ui.selectable_label(general, self.tr("🏠 Comunidade RRS")).clicked() && !general {
                self.open_community_for(None);
            }
            if let Some(repo) = &selected_repo {
                let on = self.community_target.as_deref() == Some(repo.as_str());
                if ui
                    .selectable_label(on, format!("{} {}", self.tr("💬 Sobre"), friendly_title(repo)))
                    .clicked()
                    && !on
                {
                    self.open_community_for(Some(repo.clone()));
                }
            }
        });
        ui.add_space(8.0);

        if self.auth_token.is_none() {
            ui.label(self.tr("Entre com o GitHub para ler as conversas aqui (a API de conversas do GitHub exige login)."));
            if ui.button(self.tr("Entrar com o GitHub")).clicked() {
                self.section = super::NavSection::Settings;
            }
            return;
        }
        let Some(repo) = self.community_repo_in_view() else {
            ui.label(self.tr(
                "Ainda não há repositório da comunidade RRS. Crie um repositório público no GitHub, ligue as Discussions e informe em Configurações › Central de bate-papo. As conversas de cada app já funcionam pela aba do app selecionado.",
            ));
            if ui.button(self.tr("⚙ Abrir Configurações")).clicked() {
                self.section = super::NavSection::Settings;
            }
            return;
        };

        ui.horizontal_wrapped(|ui| {
            ui.label(RichText::new(&repo).monospace().color(Color32::from_rgb(170, 190, 220)));
            if ui.button(self.tr("✏ Nova conversa no GitHub ↗")).clicked() {
                let _ = crate::downloader::open_url(&community::new_discussion_url(&repo));
            }
            if ui.button(self.tr("Abrir no GitHub ↗")).clicked() {
                let _ = crate::downloader::open_url(&community::discussions_url(&repo));
            }
            if ui.button(self.tr("🔄 Atualizar")).clicked() {
                self.community_list = None;
            }
        });
        ui.add_space(6.0);

        if let Some(thread) = self.community_thread.clone() {
            if ui.button(self.tr("← Voltar para a lista")).clicked() {
                self.community_thread = None;
                return;
            }
            match thread {
                Err(e) => {
                    ui.label(RichText::new(e).color(Color32::from_rgb(255, 120, 120)));
                }
                Ok(t) => {
                    ui.label(RichText::new(&t.title).size(18.0).strong().color(Color32::WHITE));
                    ui.label(RichText::new(format!("@{}", t.author)).color(Color32::GRAY));
                    ui.add_space(4.0);
                    message_box(ui, &t.author, "", &t.body);
                    for c in &t.comments {
                        message_box(ui, &c.author, &c.created_at, &c.body);
                    }
                    if ui.button(self.tr("↩ Responder no GitHub ↗")).clicked() {
                        let _ = crate::downloader::open_url(&t.url);
                    }
                }
            }
            return;
        }

        match self.community_list.clone() {
            None => {
                if self.busy_task.is_none() {
                    self.spawn_community_list(repo);
                }
                ui.label(self.tr("⏳ Carregando conversas..."));
            }
            Some((list_repo, _)) if list_repo != repo => {
                self.community_list = None;
            }
            Some((_, Err(e))) => {
                ui.label(RichText::new(e).color(Color32::from_rgb(255, 120, 120)));
            }
            Some((_, Ok((false, _)))) => {
                ui.label(self.tr("Este repositório não usa o GitHub Discussions. As conversas dele ficam nas Issues."));
                if ui.button(self.tr("Abrir Issues ↗")).clicked() {
                    let _ = crate::downloader::open_url(&format!("https://github.com/{repo}/issues"));
                }
            }
            Some((_, Ok((true, list)))) if list.is_empty() => {
                ui.label(self.tr("Nenhuma conversa ainda. Que tal abrir a primeira?"));
            }
            Some((_, Ok((true, list)))) => {
                for d in &list {
                    let resp = egui::Frame::NONE
                        .fill(Color32::from_rgb(24, 28, 37))
                        .stroke(Stroke::new(1.0_f32, Color32::from_rgb(38, 44, 56)))
                        .corner_radius(CornerRadius::same(6))
                        .inner_margin(egui::Margin::symmetric(12, 8))
                        .show(ui, |ui| {
                            ui.set_min_width(ui.available_width());
                            ui.horizontal_wrapped(|ui| {
                                if !d.category.is_empty() {
                                    draw_pill(ui, &d.category, Color32::from_rgb(52, 62, 82));
                                }
                                ui.label(RichText::new(&d.title).strong().color(Color32::WHITE));
                            });
                            ui.label(
                                RichText::new(format!(
                                    "@{} · {} {} · {}",
                                    d.author,
                                    d.comments,
                                    self.tr("respostas"),
                                    &d.updated_at[..d.updated_at.len().min(10)]
                                ))
                                .size(11.0)
                                .color(Color32::GRAY),
                            );
                        })
                        .response;
                    if super::clickable_card(resp, &d.title).clicked() {
                        self.spawn_community_thread(repo.clone(), d.number);
                    }
                    ui.add_space(4.0);
                }
            }
        }
    }

    // ─── Compartilhar um app ───────────────────────────────────────────────

    pub(super) fn render_share_buttons(&mut self, ui: &mut egui::Ui, app: &App) {
        ui.horizontal_wrapped(|ui| {
            if ui
                .button(self.tr("🔗 Compartilhar"))
                .on_hover_text(self.tr("Copia um texto com o nome, a descrição e o link do app para mandar a um amigo."))
                .clicked()
            {
                ui.ctx().copy_text(social::share_text(app, &friendly_title(&app.full_name)));
                self.status_msg = self.tr("🔗 Texto do app copiado; cole na conversa com o seu amigo.");
                self.status_is_error = false;
            }
            if !crate::forge::is_other_forge(&app.full_name)
                && ui.button(self.tr("💬 Conversas sobre este app")).clicked()
            {
                self.selected_repo = Some(app.full_name.clone());
                self.open_community_for(Some(app.full_name.clone()));
            }
        });
    }
}

/// Caixa de mensagem da conversa (texto puro: o Markdown do GitHub não é interpretado).
fn message_box(ui: &mut egui::Ui, author: &str, when: &str, body: &str) {
    egui::Frame::NONE
        .fill(Color32::from_rgb(20, 24, 32))
        .corner_radius(CornerRadius::same(6))
        .inner_margin(egui::Margin::same(10))
        .show(ui, |ui| {
            ui.set_min_width(ui.available_width());
            ui.label(
                RichText::new(format!("@{author}  {}", &when[..when.len().min(10)]))
                    .size(11.0)
                    .strong()
                    .color(Color32::from_rgb(150, 190, 240)),
            );
            ui.label(RichText::new(body).size(12.5).color(Color32::from_rgb(215, 222, 235)));
        });
    ui.add_space(4.0);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn avatar_no_tamanho_pedido() {
        assert_eq!(sized_avatar("https://avatars.githubusercontent.com/u/1?v=4", 64), "https://avatars.githubusercontent.com/u/1?v=4&s=64");
        assert_eq!(sized_avatar("https://avatars.githubusercontent.com/u/1", 64), "https://avatars.githubusercontent.com/u/1?s=64");
    }
}
