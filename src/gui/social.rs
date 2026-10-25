//! Perfil (nível, conquistas, amigos), central de bate-papo (GitHub Discussions), compartilhar e avisos
//! de conquista.

use super::{draw_pill, friendly_title, BgEvent, StoreGui};
use crate::achievements::{self, ACHIEVEMENTS};
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
    // ─── Conquistas: registrar eventos e avisar ────────────────────────────

    /// Soma um evento (download, verificação...) e mostra as conquistas desbloqueadas agora.
    pub(super) fn record_stat(&mut self, key: &str) {
        let Ok(conn) = crate::db::open(&self.db_path) else { return };
        if let Ok(new) = achievements::record(&conn, key, 1) {
            self.announce(new);
        }
        self.progress = achievements::progress(&conn).ok();
    }

    /// Confere conquistas sem somar evento (ex.: depois de downloads e na abertura).
    pub(super) fn check_achievements(&mut self) {
        let Ok(conn) = crate::db::open(&self.db_path) else { return };
        if let Ok(new) = achievements::check_new(&conn) {
            self.announce(new);
        }
        self.progress = achievements::progress(&conn).ok();
    }

    fn announce(&mut self, new: Vec<&'static achievements::Achievement>) {
        for a in new {
            let text = format!("🏆 {} {} (+{} XP)", self.tr("Conquista:"), self.tr(a.name), a.xp);
            self.status_msg = text.clone();
            self.status_is_error = false;
            self.toasts.push((text, None));
        }
    }

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

    /// Cartão pequeno no menu lateral: foto, nome e nível; abre o Perfil.
    pub(super) fn render_profile_chip(&mut self, ui: &mut egui::Ui) {
        if self.progress.is_none() {
            self.check_achievements();
        }
        let level = self.progress.as_ref().map(|p| p.level).unwrap_or(1);
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
                        ui.label(
                            RichText::new(format!(
                                "{} {} · {}",
                                self.tr("Nível"),
                                level,
                                self.tr(achievements::title_for(level))
                            ))
                            .size(10.5)
                            .color(Color32::from_rgb(230, 200, 120)),
                        );
                    });
                });
            })
            .response;
        if super::clickable_card(resp, &name).clicked() {
            self.section = if self.github_user.is_some() {
                super::NavSection::Profile
            } else {
                super::NavSection::Settings
            };
        }
    }

    pub(super) fn render_profile_section(&mut self, ui: &mut egui::Ui) {
        if self.progress.is_none() {
            self.check_achievements();
        }
        let progress = self.progress.clone().unwrap_or_default();
        let user = self.github_user.clone();

        egui::Frame::NONE
            .fill(Color32::from_rgb(25, 32, 48))
            .stroke(Stroke::new(1.0_f32, Color32::from_rgb(60, 90, 150)))
            .corner_radius(CornerRadius::same(12))
            .inner_margin(egui::Margin::same(18))
            .show(ui, |ui| {
                ui.set_min_width(ui.available_width());
                ui.horizontal(|ui| {
                    if let Some(u) = user.as_ref().filter(|u| !u.avatar_url.is_empty()) {
                        ui.add(
                            egui::Image::new(sized_avatar(&u.avatar_url, 128))
                                .fit_to_exact_size(Vec2::new(72.0, 72.0))
                                .corner_radius(CornerRadius::same(36)),
                        );
                    } else {
                        ui.label(RichText::new("👤").size(54.0));
                    }
                    ui.add_space(10.0);
                    ui.vertical(|ui| {
                        let name = user
                            .as_ref()
                            .map(|u| u.name.clone().unwrap_or_else(|| u.login.clone()))
                            .unwrap_or_else(|| self.tr("Jogador local"));
                        ui.label(RichText::new(name).size(22.0).strong().color(Color32::WHITE));
                        if let Some(u) = &user {
                            ui.label(RichText::new(format!("@{}", u.login)).color(Color32::GRAY));
                        }
                        ui.horizontal(|ui| {
                            draw_pill(
                                ui,
                                &format!("{} {}", self.tr("Nível"), progress.level),
                                Color32::from_rgb(35, 95, 160),
                            );
                            draw_pill(ui, &self.tr(achievements::title_for(progress.level)), Color32::from_rgb(120, 80, 20));
                            ui.label(RichText::new(format!("{} XP", progress.xp)).color(Color32::from_rgb(230, 200, 120)));
                        });
                    });
                });
                ui.add_space(8.0);
                let frac = progress.xp_in_level as f32 / progress.xp_level_size.max(1) as f32;
                ui.add(egui::ProgressBar::new(frac).text(format!(
                    "{}/{} XP {} {}",
                    progress.xp_in_level,
                    progress.xp_level_size,
                    self.tr("para o nível"),
                    progress.level + 1
                )));
                if user.is_none() {
                    ui.add_space(6.0);
                    ui.horizontal_wrapped(|ui| {
                        ui.label(
                            RichText::new(self.tr("Entre com o GitHub para ter nome, foto, amigos e perfil público."))
                                .color(Color32::from_rgb(150, 160, 180)),
                        );
                        if ui.button(self.tr("Entrar com o GitHub")).clicked() {
                            self.section = super::NavSection::Settings;
                        }
                    });
                }
            });

        // Números
        ui.add_space(10.0);
        let f = &progress.facts;
        ui.horizontal_wrapped(|ui| {
            for (label, value) in [
                (self.tr("Apps instalados"), f.installed.to_string()),
                (self.tr("Horas jogadas"), format!("{:.1}", f.playtime_seconds as f64 / 3600.0)),
                (self.tr("Downloads"), f.stat(achievements::STAT_DOWNLOADS).to_string()),
                (self.tr("Verificações"), f.stat(achievements::STAT_VERIFIES).to_string()),
                (self.tr("Traduções"), f.stat(achievements::STAT_TRANSLATIONS).to_string()),
            ] {
                egui::Frame::NONE
                    .fill(Color32::from_rgb(22, 26, 35))
                    .corner_radius(CornerRadius::same(8))
                    .inner_margin(egui::Margin::symmetric(14, 8))
                    .show(ui, |ui| {
                        ui.vertical(|ui| {
                            ui.label(RichText::new(value).size(20.0).strong().color(Color32::WHITE));
                            ui.label(RichText::new(label).size(11.5).color(Color32::GRAY));
                        });
                    });
            }
        });

        // Conquistas
        ui.add_space(14.0);
        let unlocked_count = progress.unlocked.len();
        ui.label(
            RichText::new(format!("🏆 {} ({unlocked_count}/{})", self.tr("Conquistas"), ACHIEVEMENTS.len()))
                .size(17.0)
                .strong()
                .color(Color32::WHITE),
        );
        ui.add_space(6.0);
        let cols = ((ui.available_width() / 250.0).floor() as usize).max(1);
        for row in ACHIEVEMENTS.chunks(cols) {
            ui.horizontal(|ui| {
                for a in row {
                    let when = progress.unlocked.iter().find(|(id, _)| id == a.id).map(|(_, at)| at.clone());
                    let on = when.is_some();
                    ui.vertical(|ui| {
                        egui::Frame::NONE
                            .fill(if on { Color32::from_rgb(40, 34, 12) } else { Color32::from_rgb(22, 24, 30) })
                            .stroke(Stroke::new(
                                1.0_f32,
                                if on { Color32::from_rgb(200, 160, 60) } else { Color32::from_rgb(40, 44, 54) },
                            ))
                            .corner_radius(CornerRadius::same(8))
                            .inner_margin(egui::Margin::same(10))
                            .show(ui, |ui| {
                                ui.set_width(220.0);
                                let color = if on { Color32::from_rgb(255, 225, 140) } else { Color32::from_rgb(120, 125, 135) };
                                ui.label(RichText::new(format!("{} {}", a.icon, self.tr(a.name))).strong().color(color));
                                ui.label(RichText::new(self.tr(a.desc)).size(11.0).color(Color32::from_rgb(160, 165, 175)));
                                let footer = match &when {
                                    Some(at) => format!("+{} XP · {}", a.xp, &at[..at.len().min(10)]),
                                    None => format!("+{} XP", a.xp),
                                };
                                ui.label(RichText::new(footer).size(10.5).color(Color32::GRAY));
                            });
                    });
                }
            });
            ui.add_space(6.0);
        }

        // Amigos
        ui.add_space(10.0);
        ui.horizontal_wrapped(|ui| {
            ui.label(RichText::new(self.tr("👥 Amigos")).size(17.0).strong().color(Color32::WHITE));
            if ui
                .add_enabled(
                    self.auth_token.is_some() && self.busy_task.is_none(),
                    egui::Button::new(if self.friends.is_some() { self.tr("🔄 Atualizar") } else { self.tr("Carregar amigos") }),
                )
                .clicked()
            {
                self.spawn_load_friends();
            }
        });
        ui.label(
            RichText::new(self.tr("Quem você segue no GitHub. Aparece o nível de quem também publicou o perfil da RRS."))
                .size(11.5)
                .color(Color32::from_rgb(150, 160, 180)),
        );
        let mine: Vec<String> = progress.unlocked.iter().map(|(id, _)| id.clone()).collect();
        match self.friends.clone() {
            None => {}
            Some(Err(e)) => {
                ui.label(RichText::new(e).color(Color32::from_rgb(255, 120, 120)));
            }
            Some(Ok(list)) if list.is_empty() => {
                ui.label(RichText::new(self.tr("Você ainda não segue ninguém no GitHub.")).color(Color32::GRAY));
            }
            Some(Ok(list)) => {
                for fr in &list {
                    egui::Frame::NONE
                        .fill(Color32::from_rgb(24, 28, 37))
                        .corner_radius(CornerRadius::same(6))
                        .inner_margin(egui::Margin::symmetric(10, 6))
                        .show(ui, |ui| {
                            ui.set_min_width(ui.available_width());
                            ui.horizontal_wrapped(|ui| {
                                if !fr.avatar_url.is_empty() {
                                    ui.add(
                                        egui::Image::new(sized_avatar(&fr.avatar_url, 48))
                                            .fit_to_exact_size(Vec2::new(26.0, 26.0))
                                            .corner_radius(CornerRadius::same(13)),
                                    );
                                }
                                ui.label(RichText::new(&fr.login).strong().color(Color32::WHITE));
                                match &fr.profile {
                                    Some(p) => {
                                        draw_pill(ui, &format!("{} {}", self.tr("Nível"), p.nivel), Color32::from_rgb(35, 95, 160));
                                        let common = p.conquistas.iter().filter(|c| mine.contains(c)).count();
                                        ui.label(
                                            RichText::new(format!(
                                                "{} {} · {} {}",
                                                p.conquistas.len(),
                                                self.tr("conquistas"),
                                                common,
                                                self.tr("em comum")
                                            ))
                                            .size(11.5)
                                            .color(Color32::GRAY),
                                        );
                                        let playing: Vec<String> =
                                            p.jogando.iter().take(3).map(|a| friendly_title(&a.full_name)).collect();
                                        if !playing.is_empty() {
                                            ui.label(
                                                RichText::new(format!("🎮 {}", playing.join(", ")))
                                                    .size(11.5)
                                                    .color(Color32::from_rgb(170, 190, 220)),
                                            );
                                        }
                                    }
                                    None => {
                                        ui.label(RichText::new(self.tr("não publicou perfil da RRS")).size(11.5).color(Color32::GRAY));
                                    }
                                }
                                if ui.small_button(self.tr("GitHub ↗")).clicked() {
                                    let _ = crate::downloader::open_url(&format!("https://github.com/{}", fr.login));
                                }
                            });
                        });
                    ui.add_space(4.0);
                }
            }
        }
    }

    fn spawn_load_friends(&mut self) {
        let Some(token) = self.auth_token.clone() else { return };
        self.busy_task = Some("Carregando amigos do GitHub...".into());
        let events = Arc::clone(&self.bg_events);
        self.rt.spawn(async move {
            let res = social::friends(&token).await.map_err(|e| format!("{e:#}"));
            events.lock().unwrap().push(BgEvent::FriendsLoaded(res));
        });
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
            self.record_stat(achievements::STAT_COMMUNITY);
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
                self.record_stat(achievements::STAT_SHARES);
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
