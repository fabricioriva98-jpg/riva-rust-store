//! Página de Configurações: conta do GitHub, perfil público, comunidade, idioma, chaves de API,
//! catálogo online, biblioteca, catálogo local e informações do aplicativo.

use super::{BgEvent, LoginState, StoreGui};
use crate::i18n::{tr, Lang};
use crate::{achievements, db, github_auth, social};
use eframe::egui::{self, Color32, CornerRadius, RichText, Stroke};
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

/// Cartão com título usado em cada grupo da página.
fn card(ui: &mut egui::Ui, title: &str, add: impl FnOnce(&mut egui::Ui)) {
    egui::Frame::NONE
        .fill(Color32::from_rgb(22, 26, 35))
        .stroke(Stroke::new(1.0_f32, Color32::from_rgb(38, 45, 60)))
        .corner_radius(CornerRadius::same(8))
        .inner_margin(egui::Margin::same(14))
        .show(ui, |ui| {
            ui.set_min_width(ui.available_width());
            ui.label(RichText::new(title).size(15.5).strong().color(Color32::WHITE));
            ui.add_space(6.0);
            add(ui);
        });
    ui.add_space(10.0);
}

fn hint(ui: &mut egui::Ui, text: &str) {
    ui.label(RichText::new(text).size(11.5).color(Color32::from_rgb(150, 160, 180)));
}

impl StoreGui {
    pub(super) fn render_settings_section(&mut self, ui: &mut egui::Ui) {
        ui.label(RichText::new(self.tr("⚙ Configurações")).size(21.0).strong().color(Color32::WHITE));
        ui.add_space(10.0);
        self.settings_account(ui);
        self.settings_public_profile(ui);
        self.settings_community(ui);
        self.settings_language(ui);
        self.settings_keys(ui);
        self.settings_catalog_online(ui);
        self.settings_library(ui);
        self.settings_local_catalog(ui);
        self.settings_about(ui);
    }

    // ─── Conta do GitHub ───────────────────────────────────────────────────

    fn settings_account(&mut self, ui: &mut egui::Ui) {
        let title = self.tr("👤 Conta do GitHub");
        card(ui, &title, |ui| {
            if let Some(user) = self.github_user.clone() {
                ui.horizontal(|ui| {
                    if !user.avatar_url.is_empty() {
                        ui.add(
                            egui::Image::new(super::social::sized_avatar(&user.avatar_url, 64))
                                .fit_to_exact_size(egui::vec2(40.0, 40.0))
                                .corner_radius(CornerRadius::same(20)),
                        );
                    }
                    ui.vertical(|ui| {
                        ui.label(
                            RichText::new(user.name.clone().unwrap_or_else(|| user.login.clone()))
                                .size(15.0)
                                .strong(),
                        );
                        ui.label(RichText::new(format!("@{}", user.login)).color(Color32::GRAY));
                    });
                });
                hint(ui, &self.tr("O token fica no Gerenciador de Credenciais do Windows, não no banco da RRS."));
                ui.horizontal_wrapped(|ui| {
                    if ui.button(self.tr("👤 Ver meu perfil")).clicked() {
                        self.section = super::NavSection::Profile;
                    }
                    if ui.button(self.tr("Sair da conta")).clicked() {
                        self.logout();
                    }
                });
                return;
            }

            hint(ui, &self.tr(
                "Entrar com o GitHub libera o perfil, os amigos, o compartilhamento por link e a central de bate-papo. A RRS pede só leitura do perfil e permissão de criar gists.",
            ));
            ui.add_space(6.0);
            match self.login_state.clone() {
                LoginState::WaitingCode { user_code, verification_uri } => {
                    ui.label(RichText::new(self.tr("Digite este código no GitHub:")).strong());
                    ui.label(RichText::new(&user_code).size(28.0).strong().monospace().color(Color32::from_rgb(120, 200, 255)));
                    ui.horizontal_wrapped(|ui| {
                        if ui.button(self.tr("🌐 Abrir github.com/login/device")).clicked() {
                            if let Err(e) = crate::downloader::open_url(&verification_uri) {
                                self.status_msg = format!("{e:#}");
                                self.status_is_error = true;
                            }
                        }
                        if ui.button(self.tr("📋 Copiar código")).clicked() {
                            ui.ctx().copy_text(user_code.clone());
                        }
                        if ui.button(self.tr("Cancelar")).clicked() {
                            self.login_cancel.store(true, Ordering::Relaxed);
                            self.login_state = LoginState::Idle;
                        }
                    });
                    hint(ui, &self.tr("Esperando você autorizar no navegador..."));
                }
                LoginState::Working => {
                    ui.label(self.tr("⏳ Entrando..."));
                }
                LoginState::Idle | LoginState::Failed(_) => {
                    if let LoginState::Failed(e) = &self.login_state {
                        ui.label(RichText::new(format!("✘ {e}")).color(Color32::from_rgb(255, 120, 120)));
                    }
                    ui.label(RichText::new(self.tr("Opção 1 — código no navegador (recomendado)")).strong());
                    ui.horizontal_wrapped(|ui| {
                        ui.label(self.tr("Client ID do OAuth App:"));
                        ui.add(egui::TextEdit::singleline(&mut self.oauth_client_id).hint_text("Ov23li...").desired_width(200.0));
                        let ready = !self.oauth_client_id.trim().is_empty();
                        if ui.add_enabled(ready, egui::Button::new(self.tr("Entrar com o GitHub"))).clicked() {
                            if let Ok(conn) = db::open(&self.db_path) {
                                let _ = db::set_setting(&conn, github_auth::SETTING_CLIENT_ID, self.oauth_client_id.trim());
                            }
                            self.start_device_login();
                        }
                    });
                    egui::CollapsingHeader::new(self.tr("Como registrar o OAuth App (uma vez, uns 5 minutos)"))
                        .id_salt("como_registrar_oauth")
                        .show(ui, |ui| {
                            for step in [
                                "1. No GitHub: Settings › Developer settings › OAuth Apps › New OAuth App.",
                                "2. Nome: RRS - Riva Rust Store. Homepage URL: o seu perfil (https://github.com/seu-usuario). Authorization callback URL: http://127.0.0.1 (não é usado).",
                                "3. Marque \"Enable Device Flow\" e clique em Register application.",
                                "4. Copie o Client ID (não é segredo) e cole no campo acima. Não é preciso gerar Client secret.",
                            ] {
                                hint(ui, &self.tr(step));
                            }
                        });
                    ui.add_space(6.0);
                    ui.label(RichText::new(self.tr("Opção 2 — token pessoal")).strong());
                    ui.horizontal_wrapped(|ui| {
                        ui.add(
                            egui::TextEdit::singleline(&mut self.pat_input)
                                .password(true)
                                .hint_text(tr(self.lang, "Token com os escopos read:user e gist"))
                                .desired_width(260.0),
                        );
                        if ui
                            .add_enabled(!self.pat_input.trim().is_empty(), egui::Button::new(self.tr("Entrar com token")))
                            .clicked()
                        {
                            let token = std::mem::take(&mut self.pat_input);
                            self.start_token_login(token);
                        }
                    });
                    hint(ui, &self.tr("Crie em github.com/settings/tokens (token clássico) marcando read:user e gist."));
                }
            }
        });
    }

    pub(super) fn start_device_login(&mut self) {
        let client_id = self.oauth_client_id.trim().to_string();
        let cancel = Arc::new(AtomicBool::new(false));
        self.login_cancel = Arc::clone(&cancel);
        self.login_state = LoginState::Working;
        let events = Arc::clone(&self.bg_events);
        self.rt.spawn(async move {
            let code = match github_auth::request_device_code(&client_id).await {
                Ok(c) => c,
                Err(e) => {
                    events.lock().unwrap().push(BgEvent::LoggedIn(Err(format!("{e:#}"))));
                    return;
                }
            };
            events.lock().unwrap().push(BgEvent::DeviceCode {
                user_code: code.user_code.clone(),
                verification_uri: code.verification_uri.clone(),
            });
            let res = async {
                let token = github_auth::wait_for_token(&client_id, &code, cancel).await?;
                let user = github_auth::fetch_user(&token).await?;
                github_auth::save_token(&token)?;
                Ok::<_, anyhow::Error>((token, user))
            }
            .await
            .map_err(|e| format!("{e:#}"));
            events.lock().unwrap().push(BgEvent::LoggedIn(res));
        });
    }

    fn start_token_login(&mut self, token: String) {
        self.login_state = LoginState::Working;
        let events = Arc::clone(&self.bg_events);
        self.rt.spawn(async move {
            let res = async {
                let user = github_auth::fetch_user(&token).await?;
                github_auth::save_token(&token)?;
                Ok::<_, anyhow::Error>((token, user))
            }
            .await
            .map_err(|e| format!("{e:#}"));
            events.lock().unwrap().push(BgEvent::LoggedIn(res));
        });
    }

    pub(super) fn logout(&mut self) {
        let _ = github_auth::delete_token();
        if let Ok(conn) = db::open(&self.db_path) {
            let _ = db::set_setting(&conn, github_auth::SETTING_PROFILE_CACHE, "");
        }
        self.auth_token = None;
        self.github_user = None;
        self.friends = None;
        self.login_state = LoginState::Idle;
        self.status_msg = self.tr("Você saiu da conta do GitHub (o token foi removido do Gerenciador de Credenciais).");
        self.status_is_error = false;
    }

    // ─── Perfil público ────────────────────────────────────────────────────

    fn settings_public_profile(&mut self, ui: &mut egui::Ui) {
        let title = self.tr("🌐 Perfil público (opcional)");
        card(ui, &title, |ui| {
            hint(ui, &self.tr(
                "Desligado, nada sai deste PC. Ligado, a RRS publica num Gist público da sua conta: nível, título, conquistas e os 10 apps mais usados com as horas. Não publica caminhos, hashes nem chaves.",
            ));
            let logged = self.auth_token.is_some();
            ui.horizontal_wrapped(|ui| {
                let published = self.profile_gist_id.is_some();
                if !published {
                    if ui
                        .add_enabled(logged && self.busy_task.is_none(), egui::Button::new(self.tr("🌐 Publicar meu perfil")))
                        .clicked()
                    {
                        self.spawn_publish_profile();
                    }
                } else {
                    ui.label(RichText::new(self.tr("✔ Perfil publicado")).color(Color32::from_rgb(120, 230, 150)));
                    if ui
                        .add_enabled(logged && self.busy_task.is_none(), egui::Button::new(self.tr("🔄 Atualizar agora")))
                        .clicked()
                    {
                        self.spawn_publish_profile();
                    }
                    let confirming = self.unpublish_confirm;
                    let label = if confirming { self.tr("⚠ Confirmar despublicação") } else { self.tr("Despublicar") };
                    if ui
                        .add_enabled(logged && self.busy_task.is_none(), egui::Button::new(label))
                        .on_hover_text(self.tr("Apaga o Gist do perfil na sua conta; os amigos deixam de ver o seu nível e conquistas."))
                        .clicked()
                    {
                        if confirming {
                            self.unpublish_confirm = false;
                            self.spawn_unpublish_profile();
                        } else {
                            self.unpublish_confirm = true;
                        }
                    }
                }
            });
            if !logged {
                hint(ui, &self.tr("Entre com o GitHub (acima) para publicar."));
            }
        });
    }

    pub(super) fn spawn_publish_profile(&mut self) {
        let (Some(token), Some(user)) = (self.auth_token.clone(), self.github_user.clone()) else { return };
        let Ok(conn) = db::open(&self.db_path) else { return };
        let Ok(progress) = achievements::progress(&conn) else { return };
        let profile = social::build_profile(&user.login, &progress, &self.apps);
        let existing = self.profile_gist_id.clone();
        self.busy_task = Some("Publicando perfil no GitHub...".into());
        let events = Arc::clone(&self.bg_events);
        self.rt.spawn(async move {
            let res = social::publish_profile(&token, existing.as_deref(), &profile)
                .await
                .map_err(|e| format!("{e:#}"));
            events.lock().unwrap().push(BgEvent::ProfilePublished(res));
        });
    }

    fn spawn_unpublish_profile(&mut self) {
        let (Some(token), Some(id)) = (self.auth_token.clone(), self.profile_gist_id.clone()) else { return };
        self.busy_task = Some("Despublicando perfil...".into());
        let events = Arc::clone(&self.bg_events);
        self.rt.spawn(async move {
            let res = social::unpublish_profile(&token, &id).await.map_err(|e| format!("{e:#}"));
            events.lock().unwrap().push(BgEvent::ProfileUnpublished(res));
        });
    }

    // ─── Comunidade ────────────────────────────────────────────────────────

    fn settings_community(&mut self, ui: &mut egui::Ui) {
        let title = self.tr("💬 Central de bate-papo");
        card(ui, &title, |ui| {
            hint(ui, &self.tr(
                "Usa o GitHub Discussions: as conversas de cada app ficam no repositório dele; a comunidade geral da RRS precisa de um repositório público com Discussions ligadas.",
            ));
            ui.horizontal_wrapped(|ui| {
                ui.label(self.tr("Repositório da comunidade:"));
                ui.add(egui::TextEdit::singleline(&mut self.community_repo).hint_text("dono/comunidade-rrs").desired_width(220.0));
                if ui.button(self.tr("Salvar")).clicked() {
                    let repo = crate::github::normalize_repo_name(&self.community_repo);
                    self.community_repo = repo.clone();
                    if let Ok(conn) = db::open(&self.db_path) {
                        let _ = db::set_setting(&conn, crate::community::SETTING_COMMUNITY_REPO, &repo);
                    }
                    self.community_list = None;
                    self.status_msg = self.tr("✔ Repositório da comunidade salvo.");
                    self.status_is_error = false;
                }
            });
        });
    }

    // ─── Idioma ────────────────────────────────────────────────────────────

    fn settings_language(&mut self, ui: &mut egui::Ui) {
        let title = self.tr("🌍 Idioma");
        card(ui, &title, |ui| {
            ui.horizontal_wrapped(|ui| {
                for (lang, label) in [(Lang::Pt, "Português"), (Lang::En, "English")] {
                    if ui.selectable_label(self.lang == lang, label).clicked() && self.lang != lang {
                        self.lang = lang;
                        if let Ok(conn) = db::open(&self.db_path) {
                            let _ = db::set_setting(&conn, crate::i18n::SETTING_LANG, lang.code());
                        }
                    }
                }
            });
            if self.lang == Lang::En {
                hint(ui, "App summaries come from each project's GitHub description.");
            }
        });
    }

    // ─── Chaves de API ─────────────────────────────────────────────────────

    fn settings_keys(&mut self, ui: &mut egui::Ui) {
        let title = self.tr("🔑 Chaves de API (opcionais)");
        card(ui, &title, |ui| {
            hint(ui, &self.tr(
                "Ficam no banco local da RRS (texto puro). Com a conta do GitHub conectada, o token do GitHub abaixo não é necessário.",
            ));
            egui::Grid::new("chaves_api").num_columns(2).spacing([10.0, 6.0]).show(ui, |ui| {
                ui.label(RichText::new(tr(self.lang, "🔑 GITHUB_TOKEN:")).strong());
                ui.add(
                    egui::TextEdit::singleline(&mut self.github_token_input)
                        .password(true)
                        .hint_text(tr(self.lang, "Opcional (5000 req/h no GitHub)"))
                        .desired_width(260.0),
                );
                ui.end_row();
                ui.label(RichText::new("VIRUSTOTAL_API_KEY:").strong());
                ui.add(
                    egui::TextEdit::singleline(&mut self.vt_key_input)
                        .password(true)
                        .hint_text(tr(self.lang, "Opcional (consulta VirusTotal)"))
                        .desired_width(260.0),
                );
                ui.end_row();
                ui.label(RichText::new("ANTHROPIC_API_KEY:").strong());
                ui.add(
                    egui::TextEdit::singleline(&mut self.anthropic_key_input)
                        .password(true)
                        .hint_text(tr(self.lang, "Opcional (tradutor com IA)"))
                        .desired_width(260.0),
                );
                ui.end_row();
            });
            if ui.button(tr(self.lang, "💾 Salvar Chaves")).clicked() {
                if let Ok(conn) = db::open(&self.db_path) {
                    let _ = db::set_setting(&conn, crate::translator_ai::SETTING_KEY, &self.anthropic_key_input);
                    let _ = db::set_setting(&conn, "github_token", &self.github_token_input);
                    let _ = db::set_setting(&conn, "virustotal_api_key", &self.vt_key_input);
                    self.status_msg = self.tr("Chaves de API salvas no banco local.");
                    self.status_is_error = false;
                }
            }
        });
    }

    // ─── Catálogo online ───────────────────────────────────────────────────

    fn settings_catalog_online(&mut self, ui: &mut egui::Ui) {
        let title = self.tr("🌐 Catálogo online assinado");
        card(ui, &title, |ui| {
            ui.horizontal_wrapped(|ui| {
                ui.add(
                    egui::TextEdit::singleline(&mut self.catalog_url)
                        .hint_text(tr(self.lang, "https://.../catalog.json (com catalog.json.sig ao lado)"))
                        .desired_width(320.0),
                );
                if ui
                    .add_enabled(
                        self.busy_task.is_none() && !self.catalog_url.trim().is_empty(),
                        egui::Button::new(tr(self.lang, "🔄 Salvar e atualizar")),
                    )
                    .clicked()
                {
                    if let Ok(conn) = db::open(&self.db_path) {
                        let _ = db::set_setting(&conn, crate::catalog_sign::SETTING_URL, self.catalog_url.trim());
                    }
                    self.spawn_catalog_update();
                }
            });
            hint(ui, &self.tr(
                "Só importa se a assinatura for de uma chave confiável, a versão não for mais antiga que a atual e a validade não tiver vencido. Confere uma vez por dia ao abrir.",
            ));
        });
    }

    // ─── Biblioteca ────────────────────────────────────────────────────────

    fn settings_library(&mut self, ui: &mut egui::Ui) {
        let title = self.tr("📚 Biblioteca");
        card(ui, &title, |ui| {
            ui.horizontal_wrapped(|ui| {
                ui.add(
                    egui::TextEdit::singleline(&mut self.library_path)
                        .hint_text(tr(self.lang, "Arquivo rrs-biblioteca.json"))
                        .desired_width(300.0),
                );
                if ui
                    .button(tr(self.lang, "📤 Exportar biblioteca"))
                    .on_hover_text(tr(self.lang, "Salva a lista de apps instalados e o tempo de jogo, para reinstalar em outro PC"))
                    .clicked()
                {
                    let path = PathBuf::from(self.library_path.trim());
                    let res = db::open(&self.db_path).and_then(|c| db::export_library(&c, &path));
                    (self.status_msg, self.status_is_error) = match res {
                        Ok(n) => (format!("✔ Biblioteca com {n} app(s) salva em {}", path.display()), false),
                        Err(e) => (format!("Erro ao exportar a biblioteca: {e:#}"), true),
                    };
                    if !self.status_is_error {
                        self.record_stat(achievements::STAT_LIBRARY_EXPORT);
                    }
                }
                if ui
                    .button(tr(self.lang, "📥 Importar biblioteca"))
                    .on_hover_text(tr(self.lang, "Lê um rrs-biblioteca.json de outro PC: os apps entram em Biblioteca › Para reinstalar"))
                    .clicked()
                {
                    let path = PathBuf::from(self.library_path.trim());
                    let res = db::open(&self.db_path).and_then(|c| db::import_library(&c, &path));
                    self.show_import_result(res);
                }
            });
            ui.add_space(6.0);
            ui.label(RichText::new(self.tr("Compartilhar com amigos por link")).strong());
            ui.horizontal_wrapped(|ui| {
                if ui
                    .add_enabled(
                        self.auth_token.is_some() && self.busy_task.is_none(),
                        egui::Button::new(self.tr("🔗 Criar link da minha biblioteca")),
                    )
                    .on_hover_text(self.tr("Cria um Gist secreto na sua conta (só quem tiver o link vê) e copia o link."))
                    .clicked()
                {
                    self.spawn_share_library();
                }
                ui.add(
                    egui::TextEdit::singleline(&mut self.import_link_input)
                        .hint_text(tr(self.lang, "Link recebido de um amigo (gist.github.com/...)"))
                        .desired_width(280.0),
                );
                if ui
                    .add_enabled(
                        self.busy_task.is_none() && social::gist_id_from_url(&self.import_link_input).is_some(),
                        egui::Button::new(self.tr("📥 Importar do link")),
                    )
                    .clicked()
                {
                    self.spawn_import_library_link();
                }
            });
            if self.auth_token.is_none() {
                hint(ui, &self.tr("Criar link exige entrar com o GitHub; importar de um link funciona sem conta."));
            }
        });
    }

    pub(super) fn show_import_result(&mut self, res: anyhow::Result<db::LibraryImport>) {
        (self.status_msg, self.status_is_error) = match res {
            Ok(rep) => (
                format!(
                    "✔ {} app(s) em Biblioteca › Para reinstalar; {} já instalado(s) aqui{}",
                    rep.to_reinstall,
                    rep.merged,
                    if rep.not_in_catalog.is_empty() {
                        String::new()
                    } else {
                        format!("; {} fora do catálogo", rep.not_in_catalog.len())
                    }
                ),
                false,
            ),
            Err(e) => (format!("Erro ao importar a biblioteca: {e:#}"), true),
        };
        self.reload_from_db();
    }

    fn spawn_share_library(&mut self) {
        let Some(token) = self.auth_token.clone() else { return };
        let Ok(conn) = db::open(&self.db_path) else { return };
        let items = match db::library_items(&conn) {
            Ok(items) => items,
            Err(e) => {
                self.status_msg = format!("{e:#}");
                self.status_is_error = true;
                return;
            }
        };
        let file = db::LibraryFile {
            formato: db::LIBRARY_FORMAT.into(),
            versao: 1,
            exportado_em: chrono::Utc::now().to_rfc3339(),
            apps: items,
        };
        let Ok(json) = serde_json::to_string_pretty(&file) else { return };
        self.busy_task = Some("Criando link da biblioteca...".into());
        let events = Arc::clone(&self.bg_events);
        self.rt.spawn(async move {
            let res = social::share_library(&token, &json).await.map_err(|e| format!("{e:#}"));
            events.lock().unwrap().push(BgEvent::LibraryShared(res));
        });
    }

    fn spawn_import_library_link(&mut self) {
        let Some(id) = social::gist_id_from_url(&self.import_link_input) else { return };
        let token = self.auth_token.clone();
        let db_path = self.db_path.clone();
        self.busy_task = Some("Importando biblioteca do link...".into());
        let events = Arc::clone(&self.bg_events);
        self.rt.spawn(async move {
            let res = async {
                let text = social::fetch_gist_file(&id, social::LIBRARY_GIST_FILE, token.as_deref()).await?;
                let conn = db::open(&db_path)?;
                db::import_library_str(&conn, &text)
            }
            .await;
            events.lock().unwrap().push(BgEvent::LibraryImported(res.map_err(|e| format!("{e:#}"))));
        });
    }

    // ─── Catálogo local ────────────────────────────────────────────────────

    fn settings_local_catalog(&mut self, ui: &mut egui::Ui) {
        let title = tr(self.lang, "📦 Catálogo Local:");
        card(ui, &title, |ui| {
            ui.horizontal_wrapped(|ui| {
                ui.label(
                    RichText::new(format!("{} {}", self.apps.len(), tr(self.lang, "itens carregados")))
                        .color(Color32::from_rgb(170, 185, 205)),
                );
                if ui.button(tr(self.lang, "📤 Exportar Catálogo JSON")).clicked() {
                    if let Ok(conn) = db::open(&self.db_path) {
                        (self.status_msg, self.status_is_error) = match db::export_catalog(&conn, &self.catalog_path) {
                            Ok(n) => (format!("✔ {n} itens exportados para {}", self.catalog_path.display()), false),
                            Err(e) => (format!("Erro ao exportar catálogo: {e}"), true),
                        };
                    }
                }
                if ui.button(tr(self.lang, "📥 Reimportar catalog.json")).clicked() {
                    if let Ok(conn) = db::open(&self.db_path) {
                        match db::import_catalog(&conn, &self.catalog_path) {
                            Ok(n) => {
                                self.reload_from_db();
                                self.status_msg = format!("✔ {n} itens reimportados do catalog.json!");
                                self.status_is_error = false;
                            }
                            Err(e) => {
                                self.status_msg = format!("Erro ao importar catálogo: {e}");
                                self.status_is_error = true;
                            }
                        }
                    }
                }
                if ui.button(tr(self.lang, "📁 Abrir Pasta do Catálogo")).clicked() {
                    let _ = std::process::Command::new("explorer").arg(self.base_dir()).spawn();
                }
            });
        });
    }

    // ─── Sobre ─────────────────────────────────────────────────────────────

    fn settings_about(&mut self, ui: &mut egui::Ui) {
        let title = self.tr("ℹ Sobre");
        card(ui, &title, |ui| {
            ui.label(format!("RRS - Riva Rust Store {}", env!("CARGO_PKG_VERSION")));
            ui.label(
                RichText::new(format!("{} {}", self.tr("Pasta de dados:"), self.base_dir().display()))
                    .size(11.5)
                    .color(Color32::GRAY),
            );
            hint(ui, &self.tr("A nota de confiança é um indicador de precedentes, não uma garantia de segurança."));
        });
    }
}
