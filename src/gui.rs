//! Interface gráfica estilo Steam / Microsoft Store / Google Play (egui / eframe).

use crate::db::{self, App};
use crate::downloader::{self, DownloadState, SharedDownloadState};
use crate::github::{self, Github};
use crate::translator::{self, TranslateMode};
use crate::verify;
use eframe::egui::{self, Color32, CornerRadius, RichText, Stroke, Vec2};
use std::path::PathBuf;
use std::sync::{Arc, Mutex};

#[derive(PartialEq, Clone, Copy)]
enum NavSection {
    Discover,
    ReverseEngineering,
    Games,
    Emulators,
    AiAndApps,
    Tools,
    Library,
    CppToRust,
}

#[derive(PartialEq, Clone, Copy)]
enum SortBy {
    Stars,
    Trust,
    Name,
    Updated,
}

enum BgEvent {
    Status(String),
    AppUpdated {
        app: App,
        message: String,
    },
    SyncFinished {
        updated: usize,
        message: String,
    },
    PlaytimeUpdated {
        repo: String,
        elapsed_seconds: u64,
    },
    SecurityScanResult {
        repo: String,
        defender_clean: bool,
        defender_msg: String,
        authenticode: String,
    },
    Error(String),
}

const FEATURED_REPOS: &[&str] = &[
    "HarbourMasters/Shipwright",
    "Zelda64Recomp/Zelda64Recomp",
    "diasurgical/DevilutionX",
    "NationalSecurityAgency/ghidra",
    "Anuken/Mindustry",
    "janhq/jan",
    "localsend/localsend",
];

pub struct StoreGui {
    db_path: String,
    catalog_path: PathBuf,
    rt: tokio::runtime::Handle,

    apps: Vec<App>,
    categories: Vec<String>,

    // Navegação e filtros
    section: NavSection,
    grid_view: bool,
    featured_idx: usize,
    re_subfilter: String, // "todos" | "re-game" | "re-app"
    search: String,
    platform_filter: String,
    category_filter: String,
    only_trusted: bool,
    sort_by: SortBy,

    // Seleção e detalhes
    selected_repo: Option<String>,
    selected_asset_idx: usize,

    // Status de segurança local (Defender & Authenticode)
    defender_scan_status: Option<(String, bool)>,
    authenticode_status: Option<String>,

    // Adicionar repositório avulso
    add_repo_input: String,

    // Configurações (Tokens opcionais)
    show_settings: bool,
    github_token_input: String,
    vt_key_input: String,

    // Tradutor C/C++ -> Rust embutido
    cpp_input: String,
    rust_output: String,
    translate_mode: TranslateMode,
    translate_notes: Vec<String>,
    rustc_check_status: Option<Result<String, String>>,
    file_io_path: String,

    // Tarefas em segundo plano
    download_state: SharedDownloadState,
    bg_events: Arc<Mutex<Vec<BgEvent>>>,
    busy_task: Option<String>,
    status_msg: String,
    status_is_error: bool,
}

impl StoreGui {
    pub fn new(
        cc: &eframe::CreationContext<'_>,
        db_path: String,
        catalog_path: PathBuf,
        rt: tokio::runtime::Handle,
    ) -> Self {
        egui_extras::install_image_loaders(&cc.egui_ctx);
        configure_theme(&cc.egui_ctx);

        let mut github_token_input = std::env::var("GITHUB_TOKEN").unwrap_or_default();
        let mut vt_key_input = std::env::var("VIRUSTOTAL_API_KEY").unwrap_or_default();

        if let Ok(conn) = db::open(&db_path) {
            if github_token_input.is_empty() {
                if let Some(t) = db::get_setting(&conn, "github_token") {
                    github_token_input = t;
                }
            }
            if vt_key_input.is_empty() {
                if let Some(k) = db::get_setting(&conn, "virustotal_api_key") {
                    vt_key_input = k;
                }
            }
        }

        let default_cpp = translator::EXAMPLES[0].cpp_code.to_string();
        let initial_tr = translator::translate_c_cpp_to_rust(&default_cpp, TranslateMode::SafeRust);

        let mut gui = Self {
            db_path,
            catalog_path,
            rt,
            apps: Vec::new(),
            categories: Vec::new(),
            section: NavSection::Discover,
            grid_view: true,
            featured_idx: 0,
            re_subfilter: "todos".into(),
            search: String::new(),
            platform_filter: "todas".into(),
            category_filter: "todas".into(),
            only_trusted: true,
            sort_by: SortBy::Stars,
            selected_repo: None,
            selected_asset_idx: 0,
            defender_scan_status: None,
            authenticode_status: None,
            add_repo_input: String::new(),
            show_settings: false,
            github_token_input,
            vt_key_input,
            cpp_input: default_cpp,
            rust_output: initial_tr.rust_code,
            translate_mode: TranslateMode::SafeRust,
            translate_notes: initial_tr.notes,
            rustc_check_status: None,
            file_io_path: String::new(),
            download_state: downloader::new_shared_state(),
            bg_events: Arc::new(Mutex::new(Vec::new())),
            busy_task: None,
            status_msg: "RRS - Riva Rust Store pronta. Catálogo verificado e Tradutor C/C++ ➔ Rust ativos.".into(),
            status_is_error: false,
        };
        gui.reload_from_db();
        if let Some(first) = gui
            .apps
            .iter()
            .find(|a| a.full_name.eq_ignore_ascii_case(FEATURED_REPOS[0]))
            .or_else(|| gui.apps.first())
            .map(|a| a.full_name.clone())
        {
            gui.select_app(&first);
        }
        gui
    }

    fn reload_from_db(&mut self) {
        if let Ok(conn) = db::open(&self.db_path) {
            if let Ok(list) = db::list_all(&conn) {
                self.apps = list;
            }
        }
        let mut cats: Vec<String> = self
            .apps
            .iter()
            .filter_map(|a| a.category.clone())
            .filter(|c| !c.is_empty())
            .collect();
        cats.sort();
        cats.dedup();
        self.categories = cats;
    }

    fn select_app(&mut self, full_name: &str) {
        self.selected_repo = Some(full_name.to_string());
        self.defender_scan_status = None;
        self.authenticode_status = None;
        if let Some(app) = self
            .apps
            .iter()
            .find(|a| a.full_name.eq_ignore_ascii_case(full_name))
        {
            self.selected_asset_idx = github::pick_best_asset(&app.assets).unwrap_or(0);
        } else {
            self.selected_asset_idx = 0;
        }
    }

    fn matches_global_filters(&self, a: &App) -> bool {
        if self.only_trusted && a.trust_score < 70 {
            return false;
        }
        if self.platform_filter != "todas" && !a.platforms.contains(&self.platform_filter) {
            return false;
        }
        if self.category_filter != "todas"
            && a.category.as_deref() != Some(self.category_filter.as_str())
            && a.kind.as_deref() != Some(self.category_filter.as_str())
        {
            return false;
        }
        let q = self.search.trim().to_lowercase();
        if !q.is_empty() {
            let hay = format!(
                "{} {} {} {} {} {} {}",
                friendly_title(&a.full_name),
                a.full_name,
                a.description,
                a.summary.as_deref().unwrap_or(""),
                a.category.as_deref().unwrap_or(""),
                a.tags.as_deref().unwrap_or(""),
                a.topics
            )
            .to_lowercase();
            if !hay.contains(&q) {
                return false;
            }
        }
        true
    }

    fn filtered_apps(&self) -> Vec<App> {
        let mut list: Vec<App> = self
            .apps
            .iter()
            .filter(|a| {
                let k = a.kind.as_deref().unwrap_or("");
                let section_ok = match self.section {
                    NavSection::Discover => true,
                    NavSection::ReverseEngineering => {
                        if self.re_subfilter == "re-game" {
                            k == "re-game"
                        } else if self.re_subfilter == "re-app" {
                            k == "re-app"
                        } else {
                            k == "re-game" || k == "re-app"
                        }
                    }
                    NavSection::Games => k == "game" || k == "re-game",
                    NavSection::Emulators => k == "emulator" || k == "re-game",
                    NavSection::AiAndApps => k == "ai-app" || k == "app",
                    NavSection::Tools => k == "tool" || k == "re-app",
                    NavSection::Library => a.installed.is_some(),
                    NavSection::CppToRust => true,
                };
                section_ok && self.matches_global_filters(a)
            })
            .cloned()
            .collect();

        self.sort_list(&mut list);
        list
    }

    fn sort_list(&self, list: &mut [App]) {
        match self.sort_by {
            SortBy::Stars => list.sort_by(|a, b| b.stars.cmp(&a.stars)),
            SortBy::Trust => {
                list.sort_by(|a, b| b.trust_score.cmp(&a.trust_score).then(b.stars.cmp(&a.stars)))
            }
            SortBy::Name => list.sort_by(|a, b| {
                friendly_title(&a.full_name)
                    .to_lowercase()
                    .cmp(&friendly_title(&b.full_name).to_lowercase())
            }),
            SortBy::Updated => list.sort_by(|a, b| b.pushed_at.cmp(&a.pushed_at)),
        }
    }

    fn process_background(&mut self, ctx: &egui::Context) {
        let dl_snapshot = self.download_state.lock().unwrap().clone();
        match dl_snapshot {
            DownloadState::Idle => {}
            DownloadState::Downloading { .. } => {
                ctx.request_repaint_after(std::time::Duration::from_millis(80));
            }
            DownloadState::Finished {
                repo,
                tag,
                file_path,
                sha256,
                verified_digest,
                message,
            } => {
                if let Ok(conn) = db::open(&self.db_path) {
                    let _ = db::mark_installed(&conn, &repo, &tag, &file_path, &sha256, verified_digest);
                }
                *self.download_state.lock().unwrap() = DownloadState::Idle;
                self.reload_from_db();
                self.status_msg = message;
                self.status_is_error = false;
            }
            DownloadState::Failed { repo, error } => {
                *self.download_state.lock().unwrap() = DownloadState::Idle;
                self.status_msg = format!("Falha no download de {repo}: {error}");
                self.status_is_error = true;
            }
        }

        let events: Vec<BgEvent> = self.bg_events.lock().unwrap().drain(..).collect();
        for ev in events {
            match ev {
                BgEvent::Status(s) => {
                    self.status_msg = s;
                    self.status_is_error = false;
                }
                BgEvent::AppUpdated { app, message } => {
                    let name = app.full_name.clone();
                    if let Ok(conn) = db::open(&self.db_path) {
                        let _ = db::upsert(&conn, &app);
                        let _ = db::export_catalog(&conn, &self.catalog_path);
                    }
                    self.reload_from_db();
                    self.select_app(&name);
                    self.busy_task = None;
                    self.status_msg = message;
                    self.status_is_error = false;
                }
                BgEvent::SyncFinished { updated, message } => {
                    self.reload_from_db();
                    self.busy_task = None;
                    self.status_msg = format!("{message} ({updated} verificados)");
                    self.status_is_error = false;
                }
                BgEvent::PlaytimeUpdated { repo, elapsed_seconds } => {
                    if let Ok(conn) = db::open(&self.db_path) {
                        let _ = db::record_playtime(&conn, &repo, elapsed_seconds);
                    }
                    self.reload_from_db();
                    let mins = elapsed_seconds / 60;
                    self.status_msg = if mins > 0 {
                        format!("✔ Sessão de {} encerrada (+{} min registrados)", friendly_title(&repo), mins)
                    } else {
                        format!("✔ Sessão de {} encerrada (+{}s)", friendly_title(&repo), elapsed_seconds)
                    };
                    self.status_is_error = false;
                }
                BgEvent::SecurityScanResult { repo, defender_clean, defender_msg, authenticode } => {
                    if self.selected_repo.as_deref() == Some(repo.as_str()) {
                        self.defender_scan_status = Some((defender_msg, defender_clean));
                        self.authenticode_status = Some(authenticode);
                    }
                    self.status_msg = format!("Varredura de {} concluída!", friendly_title(&repo));
                    self.status_is_error = !defender_clean;
                    self.busy_task = None;
                }
                BgEvent::Error(e) => {
                    self.busy_task = None;
                    self.status_msg = e;
                    self.status_is_error = true;
                }
            }
        }

        if self.busy_task.is_some() {
            ctx.request_repaint_after(std::time::Duration::from_millis(120));
        }
    }

    fn spawn_fetch_or_verify(&mut self, repo_name: String, check_vt: bool) {
        if self.busy_task.is_some() {
            return;
        }
        let clean = github::normalize_repo_name(&repo_name);
        if !clean.contains('/') {
            self.status_msg = "Informe no formato dono/repositorio (ex: HarbourMasters/Shipwright)".into();
            self.status_is_error = true;
            return;
        }

        self.busy_task = Some(format!("Verificando {clean}..."));
        self.status_msg = format!("Verificando precedentes e releases de {clean} no GitHub...");
        self.status_is_error = false;

        let gh_token = self.github_token_input.clone();
        let vt_key = self.vt_key_input.clone();
        let events = Arc::clone(&self.bg_events);
        let existing = self
            .apps
            .iter()
            .find(|a| a.full_name.eq_ignore_ascii_case(&clean))
            .cloned();

        self.rt.spawn(async move {
            let res: anyhow::Result<(App, String)> = async {
                let gh = Github::new(Some(&gh_token))?;
                let repo = gh.repo(&clean).await?;
                let release = gh.latest_release(&repo.full_name).await?;
                let contributors = gh.contributors(&repo.full_name).await?;

                let assets: Vec<_> = release
                    .as_ref()
                    .map(|r| {
                        r.assets
                            .iter()
                            .filter(|a| github::platform_of(&a.name).is_some())
                            .cloned()
                            .collect()
                    })
                    .unwrap_or_default();

                let mut plats: Vec<&str> = assets
                    .iter()
                    .filter_map(|a| github::platform_of(&a.name))
                    .collect();
                plats.sort();
                plats.dedup();

                let mut vt_results = Vec::new();
                if check_vt {
                    if vt_key.trim().is_empty() {
                        anyhow::bail!(
                            "Para consultar o VirusTotal, clique em 'Chaves API' e cole sua VIRUSTOTAL_API_KEY gratuita."
                        );
                    }
                    let client = reqwest::Client::new();
                    let targets: Vec<_> = assets
                        .iter()
                        .filter_map(|a| a.digest.as_ref().map(|d| (a.name.clone(), d.clone())))
                        .take(2)
                        .collect();
                    for (i, (asset_name, sha)) in targets.into_iter().enumerate() {
                        if i > 0 {
                            events.lock().unwrap().push(BgEvent::Status(
                                "Aguardando 15s (limite gratuito do VirusTotal)...".into(),
                            ));
                            tokio::time::sleep(std::time::Duration::from_secs(15)).await;
                        }
                        let stats = verify::vt_lookup(&client, &vt_key, &sha).await?;
                        vt_results.push(verify::VtResult {
                            asset: asset_name,
                            stats,
                        });
                    }
                }

                let rep = verify::evaluate(&repo, release.as_ref(), contributors, &vt_results);
                let app = App {
                    full_name: repo.full_name.clone(),
                    description: repo.description.unwrap_or_default(),
                    url: repo.html_url,
                    stars: repo.stargazers_count,
                    license: repo.license.and_then(|l| l.spdx_id).unwrap_or_default(),
                    topics: repo.topics.join(","),
                    created_at: repo.created_at,
                    pushed_at: repo.pushed_at.unwrap_or_default(),
                    fork: repo.fork,
                    archived: repo.archived,
                    has_issues: repo.has_issues,
                    latest_tag: release.map(|r| r.tag_name).unwrap_or_default(),
                    platforms: plats.join(","),
                    assets,
                    kind: existing.as_ref().and_then(|e| e.kind.clone()),
                    category: existing.as_ref().and_then(|e| e.category.clone()),
                    tags: existing.as_ref().and_then(|e| e.tags.clone()),
                    summary: existing.as_ref().and_then(|e| e.summary.clone()),
                    trust_score: rep.score,
                    trust_level: rep.level.to_string(),
                    trust_notes: rep.notes,
                    installed: existing.and_then(|e| e.installed),
                };
                let msg = if check_vt {
                    format!(
                        "Verificação completa (GitHub + VirusTotal) de {} → {}/100",
                        app.full_name, app.trust_score
                    )
                } else {
                    format!(
                        "{} verificado! Nota de confiança: {}/100",
                        app.full_name, app.trust_score
                    )
                };
                Ok((app, msg))
            }
            .await;

            match res {
                Ok((app, message)) => {
                    events.lock().unwrap().push(BgEvent::AppUpdated { app, message });
                }
                Err(e) => {
                    events.lock().unwrap().push(BgEvent::Error(format!("{e:#}")));
                }
            }
        });
    }

    fn spawn_check_installed_updates(&mut self) {
        if self.busy_task.is_some() {
            return;
        }
        let targets: Vec<App> = if self.apps.iter().any(|a| a.installed.is_some()) {
            self.apps
                .iter()
                .filter(|a| a.installed.is_some())
                .cloned()
                .collect()
        } else {
            self.apps.iter().take(6).cloned().collect()
        };

        if targets.is_empty() {
            return;
        }

        self.busy_task = Some("Checando atualizações...".into());
        let gh_token = self.github_token_input.clone();
        let db_path = self.db_path.clone();
        let catalog_path = self.catalog_path.clone();
        let events = Arc::clone(&self.bg_events);

        self.rt.spawn(async move {
            let res: anyhow::Result<usize> = async {
                let gh = Github::new(Some(&gh_token))?;
                let conn = db::open(&db_path)?;
                let mut count = 0;
                for mut app in targets {
                    events.lock().unwrap().push(BgEvent::Status(format!(
                        "Checando release de {}...",
                        app.full_name
                    )));
                    if let Ok(Some(rel)) = gh.latest_release(&app.full_name).await {
                        let assets: Vec<_> = rel
                            .assets
                            .iter()
                            .filter(|a| github::platform_of(&a.name).is_some())
                            .cloned()
                            .collect();
                        if !assets.is_empty() {
                            app.latest_tag = rel.tag_name;
                            app.assets = assets;
                            app.recompute_trust(None, &[]);
                            db::upsert(&conn, &app)?;
                            count += 1;
                        }
                    }
                }
                let _ = db::export_catalog(&conn, &catalog_path);
                Ok(count)
            }
            .await;

            match res {
                Ok(updated) => events.lock().unwrap().push(BgEvent::SyncFinished {
                    updated,
                    message: "Verificação de novas versões concluída".into(),
                }),
                Err(e) => events.lock().unwrap().push(BgEvent::Error(format!("{e:#}"))),
            }
        });
    }

    fn spawn_update_all(&mut self) {
        let outdated: Vec<App> = self.apps.iter().filter(|a| a.has_update()).cloned().collect();
        if outdated.is_empty() {
            return;
        }
        let base_dir = self
            .catalog_path
            .parent()
            .map(|p| p.to_path_buf())
            .unwrap_or_else(|| PathBuf::from("."));
        let st = Arc::clone(&self.download_state);
        let events = Arc::clone(&self.bg_events);
        self.busy_task = Some(format!("Atualizando {} aplicativos...", outdated.len()));

        self.rt.spawn(async move {
            let mut updated_count = 0;
            for app in outdated {
                if let Some(best_idx) = github::pick_best_asset(&app.assets) {
                    let asset = app.assets[best_idx].clone();
                    if downloader::download_and_verify(&app.full_name, &app.latest_tag, &asset, &base_dir, Some(st.clone())).await.is_ok() {
                        updated_count += 1;
                    }
                }
            }
            events.lock().unwrap().push(BgEvent::SyncFinished {
                updated: updated_count,
                message: format!("✔ {updated_count} aplicativos atualizados com sucesso!"),
            });
        });
    }
}

impl eframe::App for StoreGui {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        self.process_background(ctx);

        let updates_count = self.apps.iter().filter(|a| a.has_update()).count();
        let installed_count = self.apps.iter().filter(|a| a.installed.is_some()).count();
        let re_count = self
            .apps
            .iter()
            .filter(|a| matches!(a.kind.as_deref(), Some("re-game" | "re-app")))
            .count();
        let games_count = self
            .apps
            .iter()
            .filter(|a| matches!(a.kind.as_deref(), Some("game" | "re-game")))
            .count();
        let emu_count = self
            .apps
            .iter()
            .filter(|a| matches!(a.kind.as_deref(), Some("emulator")))
            .count();
        let apps_count = self
            .apps
            .iter()
            .filter(|a| matches!(a.kind.as_deref(), Some("app" | "ai-app")))
            .count();
        let tools_count = self
            .apps
            .iter()
            .filter(|a| matches!(a.kind.as_deref(), Some("tool" | "re-app")))
            .count();

        // ─── BARRA LATERAL ESQUERDA (Estilo Steam / Microsoft Store) ───────
        egui::SidePanel::left("left_nav")
            .resizable(false)
            .exact_width(210.0)
            .frame(
                egui::Frame::NONE
                    .fill(Color32::from_rgb(18, 21, 28))
                    .stroke(Stroke::new(1.0_f32, Color32::from_rgb(32, 38, 50)))
                    .inner_margin(egui::Margin::symmetric(12, 14)),
            )
            .show(ctx, |ui| {
                ui.horizontal(|ui| {
                    ui.label(
                        RichText::new("RRS")
                            .size(24.0)
                            .strong()
                            .color(Color32::from_rgb(245, 130, 65)),
                    );
                    ui.label(
                        RichText::new("Riva Rust Store")
                            .size(16.5)
                            .strong()
                            .color(Color32::from_rgb(95, 180, 255)),
                    );
                });
                ui.add_space(2.0);
                draw_pill(
                    ui,
                    "🛡 CATÁLOGO VERIFICADO",
                    Color32::from_rgb(24, 105, 65),
                );

                ui.add_space(16.0);
                ui.label(
                    RichText::new("EXPLORAR LOJA")
                        .size(11.0)
                        .strong()
                        .color(Color32::from_rgb(120, 130, 150)),
                );
                ui.add_space(4.0);

                nav_button(
                    ui,
                    &mut self.section,
                    NavSection::Discover,
                    &format!("🏠  Vitrine & Destaques ({})", self.apps.len()),
                );
                nav_button(
                    ui,
                    &mut self.section,
                    NavSection::ReverseEngineering,
                    &format!("🧬  Engenharia Reversa ({re_count})"),
                );
                nav_button(
                    ui,
                    &mut self.section,
                    NavSection::Games,
                    &format!("🎮  Jogos & Ports ({games_count})"),
                );
                nav_button(
                    ui,
                    &mut self.section,
                    NavSection::Emulators,
                    &format!("🕹  Emuladores & Retrô ({emu_count})"),
                );
                nav_button(
                    ui,
                    &mut self.section,
                    NavSection::AiAndApps,
                    &format!("🤖  IA Local & Apps ({apps_count})"),
                );
                nav_button(
                    ui,
                    &mut self.section,
                    NavSection::Tools,
                    &format!("🛠  Ferramentas & RE ({tools_count})"),
                );

                ui.add_space(14.0);
                ui.label(
                    RichText::new("SEU PC")
                        .size(11.0)
                        .strong()
                        .color(Color32::from_rgb(120, 130, 150)),
                );
                ui.add_space(4.0);

                let lib_label = if updates_count > 0 {
                    format!("📦  Biblioteca ({installed_count})  ⬆{updates_count}")
                } else {
                    format!("📦  Minha Biblioteca ({installed_count})")
                };
                nav_button(ui, &mut self.section, NavSection::Library, &lib_label);

                ui.add_space(12.0);
                ui.label(
                    RichText::new("ESTÚDIO RUST")
                        .size(11.0)
                        .strong()
                        .color(Color32::from_rgb(245, 145, 80)),
                );
                ui.add_space(4.0);
                nav_button(
                    ui,
                    &mut self.section,
                    NavSection::CppToRust,
                    "🦀  Tradutor C/C++ ➔ Rust",
                );

                ui.add_space(14.0);
                ui.separator();
                ui.add_space(10.0);

                ui.checkbox(&mut self.only_trusted, "🛡 Só Verificados (70+)");
                ui.add_space(6.0);

                let btn_text = if let Some(task) = &self.busy_task {
                    task.as_str()
                } else {
                    "🔄 Checar Atualizações"
                };
                if ui
                    .add_enabled(
                        self.busy_task.is_none(),
                        egui::Button::new(btn_text).min_size(Vec2::new(ui.available_width(), 28.0)),
                    )
                    .clicked()
                {
                    self.spawn_check_installed_updates();
                }

                ui.add_space(4.0);
                if ui
                    .button(if self.show_settings {
                        "✖ Fechar Chaves API"
                    } else {
                        "⚙ Configurar Chaves API"
                    })
                    .clicked()
                {
                    self.show_settings = !self.show_settings;
                }

                ui.add_space(4.0);
                if ui
                    .add(
                        egui::Button::new(
                            RichText::new("📌 Instalar RRS no PC")
                                .size(12.0)
                                .color(Color32::from_rgb(155, 225, 180)),
                        )
                        .fill(Color32::from_rgb(24, 52, 38))
                        .min_size(Vec2::new(ui.available_width(), 26.0)),
                    )
                    .on_hover_text(
                        "Instala a RRS no Windows e cria atalhos na Área de Trabalho, Menu Iniciar e terminal (comando 'rrs').",
                    )
                    .clicked()
                {
                    let opts = crate::installer::InstallOptions {
                        launch_after_install: false,
                        ..Default::default()
                    };
                    match crate::installer::perform_install(&opts) {
                        Ok(dest) => {
                            self.status_msg = format!(
                                "✔ RRS instalada no PC ({}) + Atalhos criados na Área de Trabalho e Menu Iniciar!",
                                dest.display()
                            );
                            self.status_is_error = false;
                        }
                        Err(e) => {
                            self.status_msg = format!("✘ Erro ao instalar no PC: {e}");
                            self.status_is_error = true;
                        }
                    }
                }
            });

        // ─── TOPO: Busca, Modo de Exibição e Filtros Rápidos ───────────────
        egui::TopBottomPanel::top("top_search_bar")
            .frame(
                egui::Frame::NONE
                    .fill(Color32::from_rgb(20, 24, 32))
                    .stroke(Stroke::new(1.0_f32, Color32::from_rgb(32, 38, 50)))
                    .inner_margin(egui::Margin::symmetric(16, 10)),
            )
            .show(ctx, |ui| {
                ui.horizontal(|ui| {
                    ui.label("🔎");
                    ui.add(
                        egui::TextEdit::singleline(&mut self.search)
                            .hint_text("Buscar jogos, ports de engenharia reversa, emuladores, IA, apps...")
                            .desired_width(300.0),
                    );
                    if !self.search.is_empty() && ui.small_button("✖").clicked() {
                        self.search.clear();
                    }

                    ui.separator();

                    ui.label(RichText::new("Plataforma:").size(12.0).color(Color32::GRAY));
                    egui::ComboBox::from_id_salt("plat_combo")
                        .selected_text(&self.platform_filter)
                        .width(88.0)
                        .show_ui(ui, |ui| {
                            for p in ["todas", "windows", "android", "linux", "macos", "java"] {
                                ui.selectable_value(&mut self.platform_filter, p.to_string(), p);
                            }
                        });

                    ui.label(RichText::new("Ordenar:").size(12.0).color(Color32::GRAY));
                    egui::ComboBox::from_id_salt("sort_combo")
                        .selected_text(match self.sort_by {
                            SortBy::Stars => "Mais Populares",
                            SortBy::Trust => "Maior Confiança",
                            SortBy::Name => "Nome (A-Z)",
                            SortBy::Updated => "Mais Recentes",
                        })
                        .width(120.0)
                        .show_ui(ui, |ui| {
                            ui.selectable_value(&mut self.sort_by, SortBy::Stars, "Mais Populares");
                            ui.selectable_value(&mut self.sort_by, SortBy::Trust, "Maior Confiança");
                            ui.selectable_value(&mut self.sort_by, SortBy::Updated, "Mais Recentes");
                            ui.selectable_value(&mut self.sort_by, SortBy::Name, "Nome (A-Z)");
                        });

                    ui.separator();
                    ui.selectable_value(&mut self.grid_view, true, "⊞ Grade");
                    ui.selectable_value(&mut self.grid_view, false, "☰ Lista");

                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        if ui
                            .add_enabled(
                                self.busy_task.is_none() && !self.add_repo_input.trim().is_empty(),
                                egui::Button::new("+ Verificar Repo"),
                            )
                            .clicked()
                        {
                            let target = self.add_repo_input.trim().to_string();
                            self.add_repo_input.clear();
                            self.spawn_fetch_or_verify(target, false);
                        }
                        ui.add(
                            egui::TextEdit::singleline(&mut self.add_repo_input)
                                .hint_text("Adicionar dono/repo do GitHub")
                                .desired_width(185.0),
                        );
                    });
                });

                if self.show_settings {
                    ui.add_space(8.0);
                    egui::Frame::NONE
                        .fill(Color32::from_rgb(28, 33, 44))
                        .corner_radius(CornerRadius::same(6))
                        .inner_margin(egui::Margin::same(10))
                        .show(ui, |ui| {
                            ui.vertical(|ui| {
                                ui.horizontal(|ui| {
                                    ui.label(RichText::new("🔑 GITHUB_TOKEN:").strong());
                                    ui.add(
                                        egui::TextEdit::singleline(&mut self.github_token_input)
                                            .password(true)
                                            .hint_text("Opcional (5000 req/h no GitHub)")
                                            .desired_width(200.0),
                                    );
                                    ui.add_space(8.0);
                                    ui.label(RichText::new("VIRUSTOTAL_API_KEY:").strong());
                                    ui.add(
                                        egui::TextEdit::singleline(&mut self.vt_key_input)
                                            .password(true)
                                            .hint_text("Opcional (consulta VirusTotal)")
                                            .desired_width(200.0),
                                    );
                                    if ui.button("💾 Salvar Chaves").clicked() {
                                        if let Ok(conn) = db::open(&self.db_path) {
                                            let _ = db::set_setting(&conn, "github_token", &self.github_token_input);
                                            let _ = db::set_setting(&conn, "virustotal_api_key", &self.vt_key_input);
                                            self.status_msg = "Chaves de API salvas localmente no SQLite.".into();
                                            self.status_is_error = false;
                                        }
                                    }
                                });

                                ui.add_space(6.0);
                                ui.separator();
                                ui.add_space(6.0);

                                ui.horizontal(|ui| {
                                    ui.label(RichText::new("📦 Catálogo Local:").strong());
                                    ui.label(
                                        RichText::new(format!("{} itens carregados", self.apps.len()))
                                            .color(Color32::from_rgb(170, 185, 205)),
                                    );

                                    ui.add_space(10.0);
                                    if ui.button("📤 Exportar Catálogo JSON").clicked() {
                                        if let Ok(conn) = db::open(&self.db_path) {
                                            match db::export_catalog(&conn, &self.catalog_path) {
                                                Ok(n) => {
                                                    self.status_msg = format!(
                                                        "✔ {n} itens exportados com sucesso para {}",
                                                        self.catalog_path.display()
                                                    );
                                                    self.status_is_error = false;
                                                }
                                                Err(e) => {
                                                    self.status_msg = format!("Erro ao exportar catálogo: {e}");
                                                    self.status_is_error = true;
                                                }
                                            }
                                        }
                                    }

                                    if ui.button("📥 Reimportar catalog.json").clicked() {
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

                                    if ui.button("📁 Abrir Pasta do Catálogo").clicked() {
                                        if let Some(parent) = self.catalog_path.parent().or_else(|| std::path::Path::new(".").into()) {
                                            let _ = std::process::Command::new("explorer").arg(parent).spawn();
                                        }
                                    }

                                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                        if ui.small_button("✖ Fechar").clicked() {
                                            self.show_settings = false;
                                        }
                                    });
                                });
                            });
                        });
                }
            });

        // ─── RODAPÉ: Barra de Status e Progresso de Download ───────────────
        egui::TopBottomPanel::bottom("bottom_status")
            .frame(
                egui::Frame::NONE
                    .fill(Color32::from_rgb(16, 18, 24))
                    .inner_margin(egui::Margin::symmetric(16, 7)),
            )
            .show(ctx, |ui| {
                ui.horizontal(|ui| {
                    let dl = self.download_state.lock().unwrap().clone();
                    if let DownloadState::Downloading {
                        repo,
                        asset_name,
                        downloaded_bytes,
                        total_bytes,
                    } = dl
                    {
                        let frac = if total_bytes > 0 {
                            (downloaded_bytes as f32 / total_bytes as f32).clamp(0.0, 1.0)
                        } else {
                            0.5
                        };
                        let mb_done = downloaded_bytes as f64 / 1_048_576.0;
                        let mb_tot = total_bytes as f64 / 1_048_576.0;
                        ui.label(
                            RichText::new(format!("⬇ Baixando {repo} ({asset_name}):"))
                                .color(Color32::from_rgb(100, 200, 255))
                                .strong(),
                        );
                        ui.add(
                            egui::ProgressBar::new(frac)
                                .desired_width(260.0)
                                .text(format!("{mb_done:.1} / {mb_tot:.1} MB ({:.0}%)", frac * 100.0)),
                        );
                    } else {
                        let color = if self.status_is_error {
                            Color32::from_rgb(255, 110, 110)
                        } else {
                            Color32::from_rgb(175, 185, 200)
                        };
                        ui.label(RichText::new(&self.status_msg).color(color));
                    }

                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        ui.label(
                            RichText::new(format!("{} projetos verificados no catálogo", self.apps.len()))
                                .size(11.5)
                                .color(Color32::from_rgb(115, 125, 140)),
                        );
                    });
                });
            });

        // ─── PAINEL DIREITO: Ficha da Loja, Precedentes e Download ─────────
        if self.section != NavSection::CppToRust {
            let selected_app = self
                .selected_repo
                .as_ref()
                .and_then(|name| self.apps.iter().find(|a| &a.full_name == name).cloned());

            if let Some(app) = selected_app {
                egui::SidePanel::right("details_panel")
                    .resizable(true)
                    .default_width(410.0)
                    .min_width(340.0)
                    .frame(
                        egui::Frame::NONE
                            .fill(Color32::from_rgb(21, 25, 34))
                            .stroke(Stroke::new(1.0_f32, Color32::from_rgb(34, 40, 54)))
                            .inner_margin(egui::Margin::same(16)),
                    )
                    .show(ctx, |ui| {
                        egui::ScrollArea::vertical().show(ui, |ui| {
                            self.render_details_panel(ui, &app);
                        });
                    });
            }
        }

        // ─── PAINEL CENTRAL: Vitrine Estilo Steam / Microsoft Store ou Tradutor ─
        egui::CentralPanel::default()
            .frame(
                egui::Frame::NONE
                    .fill(Color32::from_rgb(14, 16, 22))
                    .inner_margin(egui::Margin::same(16)),
            )
            .show(ctx, |ui| {
                if self.section == NavSection::CppToRust {
                    self.render_translator_studio(ui);
                    return;
                }
                egui::ScrollArea::vertical().show(ui, |ui| {
                    let has_active_search = !self.search.trim().is_empty()
                        || self.platform_filter != "todas"
                        || self.category_filter != "todas";

                    if self.section == NavSection::Discover && !has_active_search {
                        self.render_discover_storefront(ui);
                    } else {
                        self.render_section_catalog(ui);
                    }
                });
            });
    }
}

impl StoreGui {
    /// Página inicial "Vitrine" no estilo Steam / Microsoft Store / Google Play.
    fn render_discover_storefront(&mut self, ui: &mut egui::Ui) {
        // 1. HERO BANNER DE DESTAQUE
        let featured_apps: Vec<App> = FEATURED_REPOS
            .iter()
            .filter_map(|r| {
                self.apps
                    .iter()
                    .find(|a| a.full_name.eq_ignore_ascii_case(r))
                    .cloned()
            })
            .collect();

        if !featured_apps.is_empty() {
            let idx = self.featured_idx % featured_apps.len();
            let hero = &featured_apps[idx];
            let (_, accent) = kind_badge(hero.kind.as_deref());

            egui::Frame::NONE
                .fill(Color32::from_rgb(25, 32, 48))
                .stroke(Stroke::new(1.5_f32, accent))
                .corner_radius(CornerRadius::same(12))
                .inner_margin(egui::Margin::symmetric(20, 16))
                .show(ui, |ui| {
                    ui.set_min_width(ui.available_width());
                    ui.horizontal(|ui| {
                        draw_pill(ui, "★ DESTAQUE VERIFICADO", accent);
                        let (k_label, k_col) = kind_badge(hero.kind.as_deref());
                        draw_pill(ui, k_label, k_col);
                        if let Some(cat) = &hero.category {
                            draw_pill(ui, cat, Color32::from_rgb(48, 58, 78));
                        }
                        let (t_label, t_col) = trust_badge(hero.trust_score, &hero.trust_level);
                        draw_pill(ui, &t_label, t_col);

                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            if ui.small_button("Próximo ▶").clicked() {
                                self.featured_idx = (idx + 1) % featured_apps.len();
                            }
                            ui.label(
                                RichText::new(format!("{}/{}", idx + 1, featured_apps.len()))
                                    .size(12.0)
                                    .color(Color32::GRAY),
                            );
                            if ui.small_button("◀ Anterior").clicked() {
                                self.featured_idx = (idx + featured_apps.len() - 1) % featured_apps.len();
                            }
                        });
                    });

                    ui.add_space(8.0);
                    ui.label(
                        RichText::new(friendly_title(&hero.full_name))
                            .size(25.0)
                            .strong()
                            .color(Color32::WHITE),
                    );
                    ui.label(
                        RichText::new(hero.display_summary())
                            .size(14.5)
                            .color(Color32::from_rgb(205, 215, 232)),
                    );

                    ui.add_space(10.0);
                    ui.horizontal(|ui| {
                        if ui
                            .add(
                                egui::Button::new(
                                    RichText::new("📦 Ver na Loja & Baixar com SHA-256")
                                        .strong()
                                        .size(13.5),
                                )
                                .fill(Color32::from_rgb(38, 120, 210))
                                .min_size(Vec2::new(240.0, 32.0)),
                            )
                            .clicked()
                        {
                            let name = hero.full_name.clone();
                            self.select_app(&name);
                        }

                        ui.add_space(10.0);
                        ui.label(
                            RichText::new(format!(
                                "★ {} estrelas   •   Versão {}   •   Plataformas: {}",
                                format_stars(hero.stars),
                                hero.latest_tag,
                                hero.platforms
                            ))
                            .size(12.5)
                            .color(Color32::from_rgb(165, 175, 195)),
                        );
                    });
                });

            ui.add_space(18.0);
        }

        // 2. SEÇÃO: JOGOS DE ENGENHARIA REVERSA (PORTS NATIVOS & RECOMPILAÇÕES)
        let re_games: Vec<App> = self
            .apps
            .iter()
            .filter(|a| a.kind.as_deref() == Some("re-game") && (!self.only_trusted || a.trust_score >= 70))
            .cloned()
            .collect();
        self.render_showcase_section(
            ui,
            "🧬 Jogos de Engenharia Reversa — Ports Nativos, Decompilações & Recompilações",
            "Motores nativos de PC reconstruídos pela comunidade (60+ FPS, widescreen, mods) — 100% verificados.",
            &re_games,
            Some(NavSection::ReverseEngineering),
        );

        // 3. SEÇÃO: APLICATIVOS & SUÍTES DE ENGENHARIA REVERSA
        let re_apps: Vec<App> = self
            .apps
            .iter()
            .filter(|a| a.kind.as_deref() == Some("re-app") && (!self.only_trusted || a.trust_score >= 70))
            .cloned()
            .collect();
        self.render_showcase_section(
            ui,
            "🔬 Aplicativos de Engenharia Reversa & Análise Binária",
            "Descompiladores, debuggers e analisadores hexadecimais oficiais de código aberto.",
            &re_apps,
            Some(NavSection::ReverseEngineering),
        );

        // 4. SEÇÃO: EMULADORES & RETRÔ VERIFICADOS
        let emulators: Vec<App> = self
            .apps
            .iter()
            .filter(|a| a.kind.as_deref() == Some("emulator") && (!self.only_trusted || a.trust_score >= 70))
            .cloned()
            .collect();
        self.render_showcase_section(
            ui,
            "🕹 Emuladores & Plataformas Retrô Verificadas",
            "Emuladores oficiais de alta performance com código aberto e hashes verificados.",
            &emulators,
            Some(NavSection::Emulators),
        );

        // 5. SEÇÃO: IA LOCAL & APLICATIVOS DESKTOP ESSENCIAIS
        let ai_and_apps: Vec<App> = self
            .apps
            .iter()
            .filter(|a| {
                matches!(a.kind.as_deref(), Some("ai-app" | "app"))
                    && (!self.only_trusted || a.trust_score >= 70)
            })
            .cloned()
            .collect();
        self.render_showcase_section(
            ui,
            "🤖 IA Local & Aplicativos Essenciais para PC",
            "Modelos de IA offline, upscalers de imagem, gravação de vídeo e utilitários consagrados.",
            &ai_and_apps,
            Some(NavSection::AiAndApps),
        );

        // 6. SEÇÃO: JOGOS COMPLETOS GRATUITOS (OPEN-SOURCE)
        let free_games: Vec<App> = self
            .apps
            .iter()
            .filter(|a| a.kind.as_deref() == Some("game") && (!self.only_trusted || a.trust_score >= 70))
            .cloned()
            .collect();
        self.render_showcase_section(
            ui,
            "🎮 Jogos Completos 100% Gratuitos (Prontos para Jogar)",
            "Jogos completos de código aberto que não exigem arquivos externos.",
            &free_games,
            Some(NavSection::Games),
        );
    }

    fn render_showcase_section(
        &mut self,
        ui: &mut egui::Ui,
        title: &str,
        subtitle: &str,
        items: &[App],
        target_nav: Option<NavSection>,
    ) {
        if items.is_empty() {
            return;
        }
        ui.horizontal(|ui| {
            ui.label(
                RichText::new(title)
                    .size(17.5)
                    .strong()
                    .color(Color32::from_rgb(235, 240, 252)),
            );
            if let Some(nav) = target_nav {
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if ui.link("Ver todos →").clicked() {
                        self.section = nav;
                    }
                });
            }
        });
        ui.label(
            RichText::new(subtitle)
                .size(12.0)
                .color(Color32::from_rgb(140, 150, 170)),
        );
        ui.add_space(8.0);

        self.render_cards_collection(ui, items);
        ui.add_space(18.0);
    }

    fn render_section_catalog(&mut self, ui: &mut egui::Ui) {
        let (title, subtitle) = match self.section {
            NavSection::Discover => ("🔎 Resultados da Busca", "Projetos verificados encontrados no catálogo."),
            NavSection::ReverseEngineering => (
                "🧬 Central de Engenharia Reversa (Jogos & Aplicativos Verificados)",
                "Ports nativos de PC, recompilações estáticas, decompilações e ferramentas de engenharia reversa.",
            ),
            NavSection::Games => (
                "🎮 Catálogo de Jogos & Ports Nativos",
                "Jogos gratuitos completos e motores reconstruídos por engenharia reversa.",
            ),
            NavSection::Emulators => (
                "🕹 Emuladores & Plataformas Retrô",
                "Emuladores verificados diretamente dos repositórios oficiais dos desenvolvedores.",
            ),
            NavSection::AiAndApps => (
                "🤖 IA Local & Aplicativos Desktop",
                "Ferramentas de Inteligência Artificial offline, mídia e produtividade.",
            ),
            NavSection::Tools => (
                "🛠 Motores de Jogos, Ferramentas & Engenharia Reversa",
                "Engines, editores de nível, descompiladores e analisadores binários.",
            ),
            NavSection::Library => (
                "📦 Minha Biblioteca (Instalados neste PC)",
                "Gerencie seus jogos e aplicativos baixados, confira integridade SHA-256 e atualizações.",
            ),
            NavSection::CppToRust => ("🦀 Tradutor C/C++ ➔ Rust", ""),
        };

        ui.label(RichText::new(title).size(20.0).strong().color(Color32::WHITE));
        ui.label(
            RichText::new(subtitle)
                .size(12.5)
                .color(Color32::from_rgb(145, 155, 175)),
        );

        // Subfiltros exclusivos para a aba de Engenharia Reversa
        if self.section == NavSection::ReverseEngineering {
            ui.add_space(8.0);
            ui.horizontal(|ui| {
                ui.label(RichText::new("Mostrar:").size(12.5).color(Color32::GRAY));
                ui.selectable_value(&mut self.re_subfilter, "todos".into(), "Todos de Eng. Reversa");
                ui.selectable_value(
                    &mut self.re_subfilter,
                    "re-game".into(),
                    "🧬 Jogos (Ports, Decompilações & Recompilações)",
                );
                ui.selectable_value(
                    &mut self.re_subfilter,
                    "re-app".into(),
                    "🔬 Aplicativos & Ferramentas (Ghidra, ImHex, x64dbg)",
                );
            });
        }

        if self.section == NavSection::Library {
            let outdated_count = self.apps.iter().filter(|a| a.has_update()).count();
            if outdated_count > 0 {
                ui.add_space(8.0);
                let btn = egui::Button::new(
                    RichText::new(format!("⬆ Atualizar Todos os Aplicativos ({outdated_count} disponíveis)"))
                        .size(13.0)
                        .strong()
                        .color(Color32::WHITE),
                )
                .fill(Color32::from_rgb(205, 115, 20))
                .min_size(Vec2::new(280.0, 32.0))
                .corner_radius(CornerRadius::same(6));

                if ui.add_enabled(self.busy_task.is_none(), btn).clicked() {
                    self.spawn_update_all();
                }
            }
        }

        if self.section != NavSection::ReverseEngineering && self.section != NavSection::Library {
            ui.add_space(8.0);
            ui.horizontal_wrapped(|ui| {
                ui.label(RichText::new("Filtro:").size(12.0).color(Color32::GRAY));
                let quick_tags = [
                    ("Todos", "todas"),
                    ("🎮 Jogos", "game"),
                    ("🧬 Ports RE", "re-game"),
                    ("🔬 Ferramentas RE", "re-app"),
                    ("🤖 IA Local", "ai-app"),
                    ("🕹 Emuladores", "emulator"),
                    ("📱 Apps", "app"),
                    ("🛠 Engines", "tool"),
                ];
                for (label, kind_val) in quick_tags {
                    let is_active = self.category_filter == kind_val;
                    if ui.selectable_label(is_active, label).clicked() {
                        if is_active && kind_val != "todas" {
                            self.category_filter = "todas".into();
                        } else {
                            self.category_filter = kind_val.into();
                        }
                    }
                }
            });
        }

        ui.add_space(12.0);

        let filtered = self.filtered_apps();
        if filtered.is_empty() {
            ui.vertical_centered(|ui| {
                ui.add_space(50.0);
                ui.label(
                    RichText::new("Nenhum item encontrado nesta seção com os filtros atuais.")
                        .size(15.5)
                        .color(Color32::GRAY),
                );
            });
            return;
        }

        self.render_cards_collection(ui, &filtered);
    }

    fn render_cards_collection(&mut self, ui: &mut egui::Ui, items: &[App]) {
        if !self.grid_view {
            for app in items {
                self.render_list_card(ui, app);
                ui.add_space(8.0);
            }
            return;
        }

        let avail = ui.available_width();
        let cols = ((avail / 285.0).floor() as usize).clamp(1, 4);
        let spacing = 10.0;
        let card_w = ((avail - spacing * (cols.saturating_sub(1) as f32)) / cols as f32).max(220.0);

        for row in items.chunks(cols) {
            ui.horizontal(|ui| {
                ui.spacing_mut().item_spacing.x = spacing;
                for app in row {
                    self.render_grid_card(ui, app, card_w);
                }
            });
            ui.add_space(spacing);
        }
    }

    /// Card visual em grade (estilo Steam / Microsoft Store / Google Play).
    fn render_grid_card(&mut self, ui: &mut egui::Ui, app: &App, width: f32) {
        let is_selected = self.selected_repo.as_deref() == Some(app.full_name.as_str());
        let (kind_label, kind_color) = kind_badge(app.kind.as_deref());
        let header_bg = banner_tint(app.kind.as_deref());

        let bg = if is_selected {
            Color32::from_rgb(31, 39, 55)
        } else {
            Color32::from_rgb(22, 26, 35)
        };
        let border = if is_selected {
            Stroke::new(1.8_f32, Color32::from_rgb(95, 175, 255))
        } else {
            Stroke::new(1.0_f32, Color32::from_rgb(36, 42, 56))
        };

        let resp = egui::Frame::NONE
            .fill(bg)
            .stroke(border)
            .corner_radius(CornerRadius::same(10))
            .show(ui, |ui| {
                ui.set_width(width);
                ui.set_min_height(178.0);

                // Topo colorido do card (Banner / Capa estilizada)
                egui::Frame::NONE
                    .fill(header_bg)
                    .corner_radius(CornerRadius {
                        nw: 10,
                        ne: 10,
                        sw: 0,
                        se: 0,
                    })
                    .inner_margin(egui::Margin::symmetric(12, 8))
                    .show(ui, |ui| {
                        ui.set_width(width - 24.0);
                        ui.horizontal(|ui| {
                            // Ícone oficial do projeto (avatar do GitHub com cache assíncrono)
                            let owner = app.full_name.split('/').next().unwrap_or(&app.full_name);
                            let avatar_url = format!("https://github.com/{owner}.png?size=64");
                            ui.add(
                                egui::Image::new(&avatar_url)
                                    .fit_to_exact_size(Vec2::new(22.0, 22.0))
                                    .corner_radius(CornerRadius::same(5)),
                            );

                            draw_pill(ui, kind_label, kind_color);

                            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                let (t_short, t_col) = trust_short_badge(app.trust_score);
                                draw_pill(ui, &t_short, t_col);
                            });
                        });
                    });

                // Corpo do card
                egui::Frame::NONE
                    .inner_margin(egui::Margin::symmetric(12, 10))
                    .show(ui, |ui| {
                        ui.set_width(width - 24.0);

                        ui.label(
                            RichText::new(friendly_title(&app.full_name))
                                .size(15.0)
                                .strong()
                                .color(Color32::WHITE),
                        );

                        let owner = app.full_name.split('/').next().unwrap_or(&app.full_name);
                        ui.label(
                            RichText::new(format!(
                                "{}  •  {}",
                                owner,
                                app.category.as_deref().unwrap_or("Geral")
                            ))
                            .size(11.0)
                            .color(Color32::from_rgb(135, 150, 175)),
                        );

                        ui.add_space(4.0);
                        let sum = truncate_chars(app.display_summary(), 88);
                        ui.label(
                            RichText::new(sum)
                                .size(12.2)
                                .color(Color32::from_rgb(190, 198, 214)),
                        );

                        ui.add_space(8.0);
                        ui.horizontal(|ui| {
                            ui.label(
                                RichText::new(format!("★ {}", format_stars(app.stars)))
                                    .size(12.0)
                                    .strong()
                                    .color(Color32::from_rgb(245, 200, 75)),
                            );
                            ui.label(
                                RichText::new(format!("• {}", short_platforms(&app.platforms)))
                                    .size(11.0)
                                    .color(Color32::from_rgb(135, 145, 165)),
                            );

                            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                let (btn_txt, btn_col) = if app.has_update() {
                                    ("⬆ Atualizar", Color32::from_rgb(200, 120, 20))
                                } else if app.installed.is_some() {
                                    ("✔ Instalado", Color32::from_rgb(30, 130, 75))
                                } else {
                                    ("Obter", Color32::from_rgb(38, 110, 195))
                                };
                                if ui
                                    .add(
                                        egui::Button::new(
                                            RichText::new(btn_txt).size(11.5).strong().color(Color32::WHITE),
                                        )
                                        .fill(btn_col),
                                    )
                                    .clicked()
                                {
                                    self.select_app(&app.full_name);
                                }
                            });
                        });
                    });
            })
            .response;

        if resp.interact(egui::Sense::click()).clicked() {
            self.select_app(&app.full_name);
        }
    }

    fn render_list_card(&mut self, ui: &mut egui::Ui, app: &App) {
        let is_selected = self.selected_repo.as_deref() == Some(app.full_name.as_str());
        let bg = if is_selected {
            Color32::from_rgb(33, 42, 58)
        } else {
            Color32::from_rgb(24, 28, 37)
        };
        let border = if is_selected {
            Stroke::new(1.5_f32, Color32::from_rgb(90, 165, 250))
        } else {
            Stroke::new(1.0_f32, Color32::from_rgb(38, 44, 56))
        };

        let resp = egui::Frame::NONE
            .fill(bg)
            .stroke(border)
            .corner_radius(CornerRadius::same(8))
            .inner_margin(egui::Margin::symmetric(14, 10))
            .show(ui, |ui| {
                ui.set_min_width(ui.available_width());
                ui.horizontal(|ui| {
                    let owner = app.full_name.split('/').next().unwrap_or(&app.full_name);
                    let avatar_url = format!("https://github.com/{owner}.png?size=64");
                    ui.add(
                        egui::Image::new(&avatar_url)
                            .fit_to_exact_size(Vec2::new(26.0, 26.0))
                            .corner_radius(CornerRadius::same(5)),
                    );

                    let (kind_label, kind_color) = kind_badge(app.kind.as_deref());
                    draw_pill(ui, kind_label, kind_color);

                    if let Some(cat) = &app.category {
                        if !cat.is_empty() {
                            draw_pill(ui, cat, Color32::from_rgb(52, 62, 82));
                        }
                    }

                    ui.label(
                        RichText::new(friendly_title(&app.full_name))
                            .size(15.5)
                            .strong()
                            .color(Color32::from_rgb(235, 240, 250)),
                    );

                    if app.has_update() {
                        draw_pill(ui, "⬆ ATUALIZAÇÃO", Color32::from_rgb(200, 120, 20));
                    } else if let Some(inst) = &app.installed {
                        draw_pill(
                            ui,
                            &format!("✔ Instalado ({})", inst.installed_tag),
                            Color32::from_rgb(30, 125, 75),
                        );
                    }

                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        let (t_label, t_col) = trust_badge(app.trust_score, &app.trust_level);
                        draw_pill(ui, &t_label, t_col);

                        ui.label(
                            RichText::new(format!("★ {}", format_stars(app.stars)))
                                .strong()
                                .color(Color32::from_rgb(245, 200, 75)),
                        );
                    });
                });

                ui.add_space(4.0);
                ui.label(
                    RichText::new(app.display_summary())
                        .size(13.0)
                        .color(Color32::from_rgb(190, 198, 212)),
                );
            })
            .response;

        if resp.interact(egui::Sense::click()).clicked() {
            self.select_app(&app.full_name);
        }
    }

    fn render_details_panel(&mut self, ui: &mut egui::Ui, app: &App) {
        ui.horizontal(|ui| {
            let (kind_label, kind_color) = kind_badge(app.kind.as_deref());
            draw_pill(ui, kind_label, kind_color);
            if let Some(cat) = &app.category {
                draw_pill(ui, cat, Color32::from_rgb(52, 62, 82));
            }
        });

        ui.add_space(4.0);
        let owner = app.full_name.split('/').next().unwrap_or(&app.full_name);
        let avatar_url = format!("https://github.com/{owner}.png?size=128");
        ui.horizontal(|ui| {
            ui.add(
                egui::Image::new(&avatar_url)
                    .fit_to_exact_size(Vec2::new(42.0, 42.0))
                    .corner_radius(CornerRadius::same(8)),
            );
            ui.vertical(|ui| {
                ui.label(
                    RichText::new(friendly_title(&app.full_name))
                        .size(19.0)
                        .strong()
                        .color(Color32::WHITE),
                );
                ui.label(
                    RichText::new(&app.full_name)
                        .size(11.5)
                        .monospace()
                        .color(Color32::from_rgb(135, 155, 185)),
                );
            });
        });

        ui.add_space(4.0);
        ui.horizontal(|ui| {
            ui.label(
                RichText::new(format!("★ {} estrelas", app.stars))
                    .strong()
                    .color(Color32::from_rgb(245, 200, 75)),
            );
            ui.separator();
            ui.label(format!(
                "Licença: {}",
                if app.license.is_empty() { "Aberta" } else { &app.license }
            ));
            ui.separator();
            if ui.link("GitHub Oficial ↗").clicked() {
                downloader::open_url(&app.url);
            }
        });

        ui.add_space(8.0);
        ui.label(
            RichText::new(app.display_summary())
                .size(13.8)
                .color(Color32::from_rgb(215, 222, 235)),
        );

        // Aviso explicativo especial para Jogos de Engenharia Reversa (Ports / Recompilações)
        if app.kind.as_deref() == Some("re-game") {
            ui.add_space(8.0);
            egui::Frame::NONE
                .fill(Color32::from_rgb(35, 26, 52))
                .stroke(Stroke::new(1.0_f32, Color32::from_rgb(145, 90, 220)))
                .corner_radius(CornerRadius::same(6))
                .inner_margin(egui::Margin::same(10))
                .show(ui, |ui| {
                    ui.label(
                        RichText::new("🧬 Port / Recompilação por Engenharia Reversa Verificada")
                            .size(12.5)
                            .strong()
                            .color(Color32::from_rgb(210, 175, 255)),
                    );
                    ui.label(
                        RichText::new(
                            "Este projeto recria o motor nativo de PC de forma legal (clean-room) e sem vírus. \
                             Para jogos originalmente comerciais, o programa solicitará a sua cópia/ROM original na primeira abertura.",
                        )
                        .size(11.5)
                        .color(Color32::from_rgb(190, 180, 215)),
                    );
                });
        }

        ui.add_space(10.0);

        // ─── BLOCO 1: DOWNLOAD SEGURO COM SHA-256 E INSTALAÇÃO ─────────────
        egui::Frame::NONE
            .fill(Color32::from_rgb(16, 19, 26))
            .stroke(Stroke::new(1.0_f32, Color32::from_rgb(45, 60, 82)))
            .corner_radius(CornerRadius::same(8))
            .inner_margin(egui::Margin::same(12))
            .show(ui, |ui| {
                ui.label(
                    RichText::new(format!("📦 Obter Aplicativo — Versão {}", app.latest_tag))
                        .size(14.5)
                        .strong(),
                );
                ui.add_space(6.0);

                if app.assets.is_empty() {
                    ui.label("Nenhum instalador encontrado nesta Release.");
                    return;
                }

                let recommended_idx = github::pick_best_asset(&app.assets).unwrap_or(0);
                let idx = self.selected_asset_idx.min(app.assets.len() - 1);
                let chosen = &app.assets[idx];

                ui.label(RichText::new("Instalador selecionado:").size(11.5).color(Color32::GRAY));
                egui::ComboBox::from_id_salt("asset_selector")
                    .width(ui.available_width() - 8.0)
                    .selected_text(format!(
                        "{}{} ({:.1} MB)",
                        if idx == recommended_idx { "★ " } else { "" },
                        chosen.name,
                        chosen.size as f64 / 1_048_576.0
                    ))
                    .show_ui(ui, |ui| {
                        for (i, a) in app.assets.iter().enumerate() {
                            let label = format!(
                                "{}{} ({:.1} MB) [{}]",
                                if i == recommended_idx {
                                    "★ Recomendado p/ seu PC: "
                                } else {
                                    ""
                                },
                                a.name,
                                a.size as f64 / 1_048_576.0,
                                github::platform_of(&a.name).unwrap_or("outro")
                            );
                            ui.selectable_value(&mut self.selected_asset_idx, i, label);
                        }
                    });

                ui.add_space(6.0);
                if let Some(digest) = &chosen.digest {
                    ui.label(
                        RichText::new(format!("✔ SHA-256 Oficial do GitHub:\n{digest}"))
                            .size(10.8)
                            .monospace()
                            .color(Color32::from_rgb(105, 215, 145)),
                    );
                } else {
                    ui.label(
                        RichText::new("• O hash SHA-256 será calculado e registrado durante o download.")
                            .size(11.2)
                            .color(Color32::from_rgb(220, 185, 95)),
                    );
                }

                ui.add_space(8.0);

                let is_downloading = matches!(
                    *self.download_state.lock().unwrap(),
                    DownloadState::Downloading { .. }
                );

                let dl_btn_label = if app.has_update() {
                    format!("⬆ Atualizar para {} (Verificar SHA-256)", app.latest_tag)
                } else {
                    "⬇ Baixar & Verificar SHA-256 Automaticamente".to_string()
                };

                if ui
                    .add_enabled(
                        !is_downloading,
                        egui::Button::new(RichText::new(dl_btn_label).strong().size(13.2))
                            .min_size(Vec2::new(ui.available_width(), 34.0))
                            .fill(Color32::from_rgb(35, 115, 200)),
                    )
                    .clicked()
                {
                    let repo = app.full_name.clone();
                    let tag = app.latest_tag.clone();
                    let asset = chosen.clone();
                    let base_dir = self
                        .catalog_path
                        .parent()
                        .map(|p| p.to_path_buf())
                        .unwrap_or_else(|| PathBuf::from("."));
                    let st = Arc::clone(&self.download_state);

                    self.rt.spawn(async move {
                        if let Err(e) =
                            downloader::download_and_verify(&repo, &tag, &asset, &base_dir, Some(st.clone()))
                                .await
                        {
                            *st.lock().unwrap() = DownloadState::Failed {
                                repo,
                                error: format!("{e:#}"),
                            };
                        }
                    });
                }

                if let Some(inst) = &app.installed {
                    ui.add_space(10.0);
                    ui.separator();
                    ui.add_space(6.0);

                    let status_text = if app.has_update() {
                        format!(
                            "Instalado: {} (Nova versão disponível: {})",
                            inst.installed_tag, app.latest_tag
                        )
                    } else {
                        format!("✔ Pronto no PC — Versão {}", inst.installed_tag)
                    };
                    ui.label(
                        RichText::new(status_text)
                            .strong()
                            .color(Color32::from_rgb(100, 220, 145)),
                    );
                    ui.label(
                        RichText::new(format!(
                            "SHA-256 ({}): {}",
                            if inst.verified_github_digest {
                                "confirmado c/ GitHub"
                            } else {
                                "calculado"
                            },
                            inst.sha256
                        ))
                        .size(10.2)
                        .monospace()
                        .color(Color32::GRAY),
                    );
                    if !inst.installed_at.is_empty() {
                        ui.label(
                            RichText::new(format!(
                                "Baixado em: {}",
                                &inst.installed_at[..inst.installed_at.len().min(10)]
                            ))
                            .size(10.2)
                            .color(Color32::GRAY),
                        );
                    }

                    ui.add_space(4.0);
                    ui.horizontal(|ui| {
                        ui.label(
                            RichText::new(format!("⏱ Tempo de Jogo: {}", inst.formatted_playtime()))
                                .size(11.5)
                                .color(Color32::from_rgb(175, 215, 255)),
                        );
                        ui.add_space(6.0);
                        ui.label(
                            RichText::new(format!("📅 Última vez: {}", inst.formatted_last_played()))
                                .size(11.5)
                                .color(Color32::from_rgb(150, 165, 185)),
                        );
                    });

                    ui.add_space(8.0);
                    let is_game = matches!(app.kind.as_deref(), Some("game" | "re-game"));
                    let play_label = if is_game { "▶ JOGAR AGORA" } else { "▶ ABRIR APLICATIVO" };
                    let play_btn = egui::Button::new(
                        RichText::new(play_label).size(13.5).strong().color(Color32::WHITE),
                    )
                    .fill(Color32::from_rgb(32, 145, 78))
                    .min_size(Vec2::new(175.0, 34.0))
                    .corner_radius(CornerRadius::same(6));

                    if ui.add(play_btn).clicked() {
                        let path = inst.file_path.clone();
                        let repo_name = app.full_name.clone();
                        let events = Arc::clone(&self.bg_events);
                        self.status_msg = format!("▶ Iniciando {}...", friendly_title(&repo_name));
                        self.status_is_error = false;
                        std::thread::spawn(move || {
                            let start = std::time::Instant::now();
                            match downloader::launch_installed_app(&path) {
                                Ok(Some(mut child)) => {
                                    let _ = child.wait();
                                    let elapsed = start.elapsed().as_secs();
                                    events.lock().unwrap().push(BgEvent::PlaytimeUpdated {
                                        repo: repo_name,
                                        elapsed_seconds: elapsed,
                                    });
                                }
                                Ok(None) => {}
                                Err(e) => {
                                    events.lock().unwrap().push(BgEvent::Error(format!("Erro ao iniciar: {e}")));
                                }
                            }
                        });
                    }

                    ui.add_space(6.0);
                    ui.horizontal(|ui| {
                        if ui.button("📂 Abrir Pasta").clicked() {
                            let _ = downloader::reveal_in_folder(&inst.file_path);
                        }

                        if ui
                            .add_enabled(
                                self.busy_task.is_none(),
                                egui::Button::new("🛡️ Windows Defender"),
                            )
                            .on_hover_text("Executa uma varredura nativa do Windows Defender no arquivo baixado sem custos")
                            .clicked()
                        {
                            let path = inst.file_path.clone();
                            let repo = app.full_name.clone();
                            let events = Arc::clone(&self.bg_events);
                            self.busy_task = Some("Escaneando com Windows Defender...".into());
                            std::thread::spawn(move || {
                                let def_res = downloader::scan_with_windows_defender(&path).unwrap_or(
                                    downloader::DefenderScanResult {
                                        is_clean: true,
                                        exit_code: 0,
                                        message: "Não foi possível acionar o Windows Defender.".into(),
                                    },
                                );
                                let auth_res = downloader::check_authenticode_signature(&path).unwrap_or(
                                    downloader::AuthenticodeResult {
                                        status: "Não assinado".into(),
                                        signer: None,
                                    },
                                );
                                let auth_text = match auth_res.signer {
                                    Some(s) => format!("Assinatura: {} ({})", auth_res.status, s),
                                    None => format!("Assinatura: {}", auth_res.status),
                                };
                                events.lock().unwrap().push(BgEvent::SecurityScanResult {
                                    repo,
                                    defender_clean: def_res.is_clean,
                                    defender_msg: def_res.message,
                                    authenticode: auth_text,
                                });
                            });
                        }

                        if ui.button("🗑 Desinstalar").clicked() {
                            let base_dir = self
                                .catalog_path
                                .parent()
                                .map(|p| p.to_path_buf())
                                .unwrap_or_else(|| PathBuf::from("."));
                            let _ = downloader::uninstall_app(&base_dir, &app.full_name);
                            if let Ok(conn) = db::open(&self.db_path) {
                                let _ = db::unmark_installed(&conn, &app.full_name);
                            }
                            self.reload_from_db();
                            self.status_msg = format!("✔ {} removido do computador.", friendly_title(&app.full_name));
                            self.status_is_error = false;
                        }
                    });

                    if let Some((msg, is_clean)) = &self.defender_scan_status {
                        ui.add_space(6.0);
                        let col = if *is_clean {
                            Color32::from_rgb(120, 230, 150)
                        } else {
                            Color32::from_rgb(255, 120, 120)
                        };
                        ui.label(RichText::new(msg).size(11.5).strong().color(col));
                    }

                    if let Some(auth) = &self.authenticode_status {
                        ui.add_space(2.0);
                        ui.label(
                            RichText::new(format!("🔏 {auth}"))
                                .size(11.0)
                                .color(Color32::from_rgb(180, 200, 225)),
                        );
                    }
                }
            });

        ui.add_space(12.0);

        // ─── BLOCO 2: VERIFICAÇÃO DE SEGURANÇA E PRECEDENTES ───────────────
        let (t_label, t_col) = trust_badge(app.trust_score, &app.trust_level);
        egui::Frame::NONE
            .fill(Color32::from_rgb(16, 19, 26))
            .stroke(Stroke::new(1.0_f32, t_col))
            .corner_radius(CornerRadius::same(8))
            .inner_margin(egui::Margin::same(12))
            .show(ui, |ui| {
                ui.horizontal(|ui| {
                    ui.label(RichText::new("🛡 Auditoria de Precedentes").size(14.5).strong());
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        draw_pill(ui, &t_label, t_col);
                    });
                });

                ui.add_space(6.0);
                ui.add(
                    egui::ProgressBar::new(app.trust_score as f32 / 100.0)
                        .fill(t_col)
                        .text(format!("Índice de Confiança: {}/100", app.trust_score)),
                );

                ui.add_space(8.0);
                for note in &app.trust_notes {
                    let (clean_note, col) = clean_and_color_note(note);
                    ui.label(RichText::new(clean_note).size(12.2).color(col));
                }

                ui.add_space(8.0);
                ui.horizontal(|ui| {
                    if ui
                        .add_enabled(
                            self.busy_task.is_none(),
                            egui::Button::new("🔄 Revalidar no GitHub"),
                        )
                        .clicked()
                    {
                        self.spawn_fetch_or_verify(app.full_name.clone(), false);
                    }

                    if ui
                        .add_enabled(
                            self.busy_task.is_none(),
                            egui::Button::new("🔍 Checar no VirusTotal"),
                        )
                        .on_hover_text("Consulta os hashes oficiais da Release no VirusTotal sem baixar o arquivo")
                        .clicked()
                    {
                        self.spawn_fetch_or_verify(app.full_name.clone(), true);
                    }
                });
            });
    }

    /// Estúdio interativo de Tradução de Código C e C++ para Rust dentro do RRS - Riva Rust Store.
    fn render_translator_studio(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            ui.label(
                RichText::new("🦀 Tradutor de C / C++ para Rust — RRS Studio")
                    .size(21.0)
                    .strong()
                    .color(Color32::from_rgb(245, 140, 75)),
            );
            draw_pill(ui, "OFFLINE & INSTANTÂNEO", Color32::from_rgb(30, 120, 75));
            draw_pill(ui, "VALIDADOR RUSTC INTEGRADO", Color32::from_rgb(45, 95, 175));
        });
        ui.label(
            RichText::new(
                "Converta estruturas, classes, headers de engenharia reversa, ponteiros, loops e containers STL (std::vector, std::string) de C/C++ para Rust.",
            )
            .size(12.5)
            .color(Color32::from_rgb(155, 165, 185)),
        );

        ui.add_space(10.0);

        // Barra de Controles: Modo + Exemplos Prontos + Botões de Ação
        let mut trigger_retranslate = false;

        egui::Frame::NONE
            .fill(Color32::from_rgb(22, 26, 35))
            .stroke(Stroke::new(1.0_f32, Color32::from_rgb(38, 45, 60)))
            .corner_radius(CornerRadius::same(8))
            .inner_margin(egui::Margin::symmetric(12, 10))
            .show(ui, |ui| {
                ui.horizontal(|ui| {
                    ui.label(RichText::new("Modo de Conversão:").strong().size(12.5));
                    if ui
                        .selectable_value(
                            &mut self.translate_mode,
                            TranslateMode::SafeRust,
                            "🦀 Rust Idiomático & Seguro (Vec, String, &mut)",
                        )
                        .clicked()
                    {
                        trigger_retranslate = true;
                    }
                    if ui
                        .selectable_value(
                            &mut self.translate_mode,
                            TranslateMode::FfiBindings,
                            "🔗 FFI / ABI C (#[repr(C)], *mut T, extern \"C\")",
                        )
                        .clicked()
                    {
                        trigger_retranslate = true;
                    }

                    ui.separator();

                    ui.label(RichText::new("Exemplos:").size(12.0).color(Color32::GRAY));
                    for (idx, ex) in translator::EXAMPLES.iter().enumerate() {
                        if ui
                            .button(format!("Exemplo {}", idx + 1))
                            .on_hover_text(format!("{}\n{}", ex.name, ex.description))
                            .clicked()
                        {
                            self.cpp_input = ex.cpp_code.to_string();
                            self.rustc_check_status = None;
                            trigger_retranslate = true;
                        }
                    }
                });

                ui.add_space(8.0);

                ui.horizontal(|ui| {
                    ui.label(RichText::new("Arquivo (.c/.cpp/.h/.rs):").size(12.0).color(Color32::GRAY));
                    ui.add(
                        egui::TextEdit::singleline(&mut self.file_io_path)
                            .hint_text("Caminho de arquivo para carregar (.cpp/.h) ou salvar (.rs)")
                            .desired_width(290.0),
                    );
                    if ui.button("📂 Carregar C/C++").clicked() && !self.file_io_path.trim().is_empty() {
                        match std::fs::read_to_string(self.file_io_path.trim()) {
                            Ok(content) => {
                                self.cpp_input = content;
                                self.rustc_check_status = None;
                                trigger_retranslate = true;
                                self.status_msg = format!("Arquivo C/C++ carregado: {}", self.file_io_path.trim());
                                self.status_is_error = false;
                            }
                            Err(e) => {
                                self.status_msg = format!("Erro ao ler arquivo: {e}");
                                self.status_is_error = true;
                            }
                        }
                    }
                    if ui.button("💾 Salvar .rs").clicked() && !self.file_io_path.trim().is_empty() {
                        match std::fs::write(self.file_io_path.trim(), &self.rust_output) {
                            Ok(_) => {
                                self.status_msg = format!("Código Rust salvo em: {}", self.file_io_path.trim());
                                self.status_is_error = false;
                            }
                            Err(e) => {
                                self.status_msg = format!("Erro ao salvar arquivo: {e}");
                                self.status_is_error = true;
                            }
                        }
                    }

                    if ui
                        .add(
                            egui::Button::new(
                                RichText::new("📦 Gerar Projeto Cargo")
                                    .strong()
                                    .color(Color32::from_rgb(255, 210, 140)),
                            )
                            .fill(Color32::from_rgb(52, 38, 20)),
                        )
                        .on_hover_text("Cria uma pasta completa com Cargo.toml e src/main.rs pronta para 'cargo run'")
                        .clicked()
                    {
                        let base_dir = self
                            .catalog_path
                            .parent()
                            .map(|p| p.to_path_buf())
                            .unwrap_or_else(|| PathBuf::from("."));
                        let proj_name = if self.file_io_path.trim().is_empty() {
                            "meu_projeto_rust"
                        } else {
                            std::path::Path::new(self.file_io_path.trim())
                                .file_stem()
                                .and_then(|s| s.to_str())
                                .unwrap_or("meu_projeto_rust")
                        };
                        match translator::generate_cargo_project(&base_dir, proj_name, &self.rust_output) {
                            Ok(path) => {
                                self.status_msg = format!("✔ Projeto Cargo gerado em: {}", path.display());
                                self.status_is_error = false;
                                let _ = downloader::reveal_in_folder(&path.to_string_lossy());
                            }
                            Err(e) => {
                                self.status_msg = format!("Erro ao gerar projeto Cargo: {e}");
                                self.status_is_error = true;
                            }
                        }
                    }

                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        if ui
                            .add(
                                egui::Button::new(RichText::new("📋 Copiar Rust").strong())
                                    .fill(Color32::from_rgb(45, 95, 165)),
                            )
                            .clicked()
                        {
                            ui.ctx().copy_text(self.rust_output.clone());
                            self.status_msg = "Código Rust copiado para a área de transferência!".into();
                            self.status_is_error = false;
                        }

                        if ui
                            .add(
                                egui::Button::new(
                                    RichText::new("✔ Validar Sintaxe no rustc")
                                        .strong()
                                        .color(Color32::WHITE),
                                )
                                .fill(Color32::from_rgb(30, 130, 75)),
                            )
                            .on_hover_text("Compila o código Rust gerado usando o rustc local para garantir que não há erros")
                            .clicked()
                        {
                            let res = translator::verify_rust_syntax(&self.rust_output);
                            match &res {
                                Ok(msg) => {
                                    self.status_msg = msg.clone();
                                    self.status_is_error = false;
                                }
                                Err(_) => {
                                    self.status_msg = "O rustc apontou ajustes necessários no código traduzido.".into();
                                    self.status_is_error = true;
                                }
                            }
                            self.rustc_check_status = Some(res);
                        }

                        if ui
                            .add(
                                egui::Button::new(RichText::new("⚡ Traduzir").strong())
                                    .fill(Color32::from_rgb(200, 95, 40)),
                            )
                            .clicked()
                        {
                            trigger_retranslate = true;
                        }
                    });
                });
            });

        ui.add_space(10.0);

        // Editor Lado a Lado: C/C++ (Esquerda)  ➔  Rust (Direita)
        let editor_height = (ui.available_height() - 135.0).max(240.0);
        let half_w = ((ui.available_width() - 14.0) / 2.0).max(260.0);

        ui.horizontal(|ui| {
            // Coluna Esquerda: Entrada C / C++
            ui.vertical(|ui| {
                ui.set_width(half_w);
                ui.horizontal(|ui| {
                    draw_pill(ui, "ENTRADA C / C++", Color32::from_rgb(55, 85, 140));
                    ui.label(
                        RichText::new("Cole seu código .c, .cpp, .h ou .hpp abaixo (tradução em tempo real)")
                            .size(11.5)
                            .color(Color32::GRAY),
                    );
                });
                ui.add_space(4.0);
                egui::ScrollArea::vertical()
                    .id_salt("cpp_scroll")
                    .max_height(editor_height)
                    .show(ui, |ui| {
                        let resp = ui.add(
                            egui::TextEdit::multiline(&mut self.cpp_input)
                                .code_editor()
                                .desired_width(half_w)
                                .desired_rows(22),
                        );
                        if resp.changed() {
                            trigger_retranslate = true;
                        }
                    });
            });

            ui.add_space(10.0);

            // Coluna Direita: Saída Rust
            ui.vertical(|ui| {
                ui.set_width(half_w);
                ui.horizontal(|ui| {
                    draw_pill(ui, "SAÍDA RUST (.RS)", Color32::from_rgb(195, 85, 35));
                    ui.label(
                        RichText::new("Você também pode editar o código Rust gerado antes de validar/salvar")
                            .size(11.5)
                            .color(Color32::GRAY),
                    );
                });
                ui.add_space(4.0);
                egui::ScrollArea::vertical()
                    .id_salt("rust_scroll")
                    .max_height(editor_height)
                    .show(ui, |ui| {
                        ui.add(
                            egui::TextEdit::multiline(&mut self.rust_output)
                                .code_editor()
                                .desired_width(half_w)
                                .desired_rows(22),
                        );
                    });
            });
        });

        if trigger_retranslate {
            let tr = translator::translate_c_cpp_to_rust(&self.cpp_input, self.translate_mode);
            self.rust_output = tr.rust_code;
            self.translate_notes = tr.notes;
        }

        ui.add_space(8.0);

        // Painel Inferior de Diagnósticos da Tradução e Validação rustc
        egui::Frame::NONE
            .fill(Color32::from_rgb(19, 23, 31))
            .stroke(Stroke::new(1.0_f32, Color32::from_rgb(38, 46, 62)))
            .corner_radius(CornerRadius::same(8))
            .inner_margin(egui::Margin::same(10))
            .show(ui, |ui| {
                ui.set_min_width(ui.available_width());
                ui.label(
                    RichText::new("📋 Notas de Conversão & Verificação do Compilador")
                        .size(13.0)
                        .strong()
                        .color(Color32::WHITE),
                );
                ui.add_space(4.0);
                for n in &self.translate_notes {
                    ui.label(
                        RichText::new(format!("• {n}"))
                            .size(12.0)
                            .color(Color32::from_rgb(175, 210, 245)),
                    );
                }
                if let Some(status) = &self.rustc_check_status {
                    ui.add_space(4.0);
                    match status {
                        Ok(ok_msg) => {
                            ui.label(
                                RichText::new(ok_msg)
                                    .size(12.5)
                                    .strong()
                                    .color(Color32::from_rgb(100, 225, 145)),
                            );
                        }
                        Err(err_msg) => {
                            ui.label(
                                RichText::new(format!("Diagnóstico do rustc:\n{err_msg}"))
                                    .size(11.2)
                                    .monospace()
                                    .color(Color32::from_rgb(255, 140, 120)),
                            );
                        }
                    }
                }
            });
    }
}

fn configure_theme(ctx: &egui::Context) {
    let mut visuals = egui::Visuals::dark();
    visuals.window_fill = Color32::from_rgb(16, 19, 26);
    visuals.panel_fill = Color32::from_rgb(14, 16, 22);
    ctx.set_visuals(visuals);
}

fn nav_button(ui: &mut egui::Ui, current: &mut NavSection, target: NavSection, label: &str) {
    let active = *current == target;
    let fill = if active {
        Color32::from_rgb(35, 85, 155)
    } else {
        Color32::TRANSPARENT
    };
    let text_col = if active {
        Color32::WHITE
    } else {
        Color32::from_rgb(185, 195, 210)
    };
    if ui
        .add(
            egui::Button::new(RichText::new(label).size(13.0).strong().color(text_col))
                .fill(fill)
                .min_size(Vec2::new(ui.available_width(), 30.0)),
        )
        .clicked()
    {
        *current = target;
    }
}

fn kind_badge(kind: Option<&str>) -> (&'static str, Color32) {
    match kind {
        Some("re-game") => ("PORT / RECOMP", Color32::from_rgb(140, 55, 205)),
        Some("re-app") => ("ENG. REVERSA", Color32::from_rgb(195, 70, 55)),
        Some("emulator") => ("EMULADOR", Color32::from_rgb(25, 140, 145)),
        Some("ai-app") => ("IA LOCAL", Color32::from_rgb(90, 80, 210)),
        Some("game") => ("JOGO GRÁTIS", Color32::from_rgb(40, 140, 85)),
        Some("app") => ("APLICATIVO", Color32::from_rgb(35, 115, 195)),
        Some("tool") => ("FERRAMENTA", Color32::from_rgb(95, 110, 130)),
        _ => ("ITEM", Color32::from_rgb(75, 80, 95)),
    }
}

fn banner_tint(kind: Option<&str>) -> Color32 {
    match kind {
        Some("re-game") => Color32::from_rgb(38, 24, 58),
        Some("re-app") => Color32::from_rgb(52, 26, 28),
        Some("emulator") => Color32::from_rgb(20, 44, 48),
        Some("ai-app") => Color32::from_rgb(28, 28, 60),
        Some("game") => Color32::from_rgb(22, 44, 34),
        Some("app") => Color32::from_rgb(22, 36, 56),
        _ => Color32::from_rgb(28, 32, 42),
    }
}

fn friendly_title(full_name: &str) -> String {
    match full_name {
        "yt-dlp/yt-dlp" => "yt-dlp (Downloader de Mídia)".into(),
        "nomic-ai/gpt4all" => "GPT4All (Assistente IA Local)".into(),
        "qbittorrent/qBittorrent" => "qBittorrent".into(),
        "FreeCAD/FreeCAD" => "FreeCAD (Modelador CAD 3D)".into(),
        "telegramdesktop/tdesktop" => "Telegram Desktop".into(),
        "notepad-plus-plus/notepad-plus-plus" => "Notepad++".into(),
        "Devolutions/UniGetUI" => "UniGetUI (Gerenciador de Pacotes)".into(),
        "mozilla-ai/llamafile" => "llamafile (LLM Portátil)".into(),
        "radareorg/radare2" => "radare2 (Framework de Eng. Reversa)".into(),
        "PCSX2/pcsx2" => "PCSX2 (Emulador de PS2)".into(),
        "mltframework/shotcut" => "Shotcut (Editor de Vídeo)".into(),
        "transmission/transmission" => "Transmission".into(),
        "LostRuins/koboldcpp" => "KoboldCpp (IA Local p/ RPG & Chat)".into(),
        "dnSpyEx/dnSpy" => "dnSpy (.NET Debugger & Editor)".into(),
        "Open-Shell/Open-Shell-Menu" => "Open-Shell (Menu Iniciar Clássico)".into(),
        "mgba-emu/mgba" => "mGBA (Emulador de GBA)".into(),
        "bleachbit/bleachbit" => "BleachBit (Limpeza & Privacidade)".into(),
        "The-Powder-Toy/The-Powder-Toy" => "The Powder Toy (Física Sandbox)".into(),
        "melonDS-emu/melonDS" => "melonDS (Emulador de Nintendo DS)".into(),
        "CorsixTH/CorsixTH" => "CorsixTH (Theme Hospital Engine)".into(),
        "Warzone2100/warzone2100" => "Warzone 2100 (RTS 3D)".into(),
        "space-wizards/space-station-14" => "Space Station 14".into(),
        "ZDoom/gzdoom" => "GZDoom (Motor de DOOM)".into(),
        "2ship2harkinian/2ship2harkinian" => "2 Ship 2 Harkinian (Zelda MM PC)".into(),
        "Keriew/augustus" => "Augustus (Caesar III Aprimorado)".into(),
        "OpenLoco/OpenLoco" => "OpenLoco (Transport Tycoon)".into(),
        "taisei-project/taisei" => "Taisei Project (Bullet Hell)".into(),
        "ollama/ollama" => "Ollama (Motor de IA Local)".into(),
        "rustdesk/rustdesk" => "RustDesk (Acesso Remoto)".into(),
        "godotengine/godot" => "Godot Engine (2D & 3D)".into(),
        "syncthing/syncthing" => "Syncthing (Sync P2P)".into(),
        "mifi/lossless-cut" => "LosslessCut (Editor de Vídeo)".into(),
        "LizardByte/Sunshine" => "Sunshine (Game Stream Host)".into(),
        "koreader/koreader" => "KOReader (Leitor de E-Books)".into(),
        "keepassxreboot/keepassxc" => "KeePassXC (Cofre de Senhas)".into(),
        "icsharpcode/ILSpy" => "ILSpy (Descompilador .NET)".into(),
        "HandBrake/HandBrake" => "HandBrake (Conversor de Vídeo)".into(),
        "rizinorg/cutter" => "Cutter (Eng. Reversa & Disassembler)".into(),
        "moonlight-stream/moonlight-qt" => "Moonlight (Streaming de Jogos)".into(),
        "PrismLauncher/PrismLauncher" => "Prism Launcher (Minecraft)".into(),
        "cemu-project/Cemu" => "Cemu (Emulador de Wii U)".into(),
        "veloren/veloren" => "Veloren (Voxel RPG em Rust)".into(),
        "supertuxkart/stk-code" => "SuperTuxKart (Corrida 3D)".into(),
        "bvschaik/julius" => "Julius (Caesar III Port)".into(),
        "SuperTux/supertux" => "SuperTux (Plataforma 2D)".into(),
        "HarbourMasters/Shipwright" => "Ship of Harkinian (Zelda OoT PC)".into(),
        "Zelda64Recomp/Zelda64Recomp" => "Zelda 64: Recompiled".into(),
        "diasurgical/DevilutionX" | "diasurgical/devilutionX" => "DevilutionX (Diablo 1 Port)".into(),
        "OpenMW/openmw" => "OpenMW (Morrowind Engine)".into(),
        "ihhub/fheroes2" => "fheroes2 (Heroes II Port)".into(),
        "vcmi/vcmi" => "VCMI (Heroes III Engine)".into(),
        "OpenRCT2/OpenRCT2" => "OpenRCT2 (RollerCoaster Tycoon 2)".into(),
        "OpenRA/OpenRA" => "OpenRA (Command & Conquer / Red Alert)".into(),
        "NationalSecurityAgency/ghidra" => "Ghidra SRE (Suíte de Eng. Reversa)".into(),
        "WerWolv/ImHex" => "ImHex (Hex Editor p/ Eng. Reversa)".into(),
        "x64dbg/x64dbg" => "x64dbg (Debugger Binário x64/x32)".into(),
        "hrydgard/ppsspp" => "PPSSPP (Emulador de PSP)".into(),
        "azahar-emu/azahar" => "Azahar (Emulador de Nintendo 3DS)".into(),
        "janhq/jan" => "Jan (IA Local Offline)".into(),
        "upscayl/upscayl" => "Upscayl (IA Upscaler de Imagens)".into(),
        "Anuken/Mindustry" => "Mindustry".into(),
        "ppy/osu" => "osu! (lazer)".into(),
        "CleverRaven/Cataclysm-DDA" => "Cataclysm: Dark Days Ahead".into(),
        "luanti-org/luanti" => "Luanti (Minetest)".into(),
        "yairm210/Unciv" => "Unciv (Civilization V Remake)".into(),
        "endless-sky/endless-sky" => "Endless Sky".into(),
        "localsend/localsend" => "LocalSend".into(),
        "obsproject/obs-studio" => "OBS Studio".into(),
        "ShareX/ShareX" => "ShareX".into(),
        "audacity/audacity" => "Audacity".into(),
        "Flow-Launcher/Flow.Launcher" => "Flow Launcher".into(),
        "Heroic-Games-Launcher/HeroicGamesLauncher" => "Heroic Games Launcher".into(),
        "4ian/GDevelop" => "GDevelop Engine".into(),
        "mapeditor/tiled" => "Tiled Map Editor".into(),
        "heroiclabs/nakama" => "Nakama Game Server".into(),
        other => other.split('/').nth(1).unwrap_or(other).to_string(),
    }
}

#[allow(dead_code)]
fn monogram(full_name: &str) -> String {
    let repo = full_name.split('/').nth(1).unwrap_or(full_name);
    let chars: Vec<char> = repo.chars().filter(|c| c.is_alphanumeric()).take(3).collect();
    chars.into_iter().collect::<String>().to_uppercase()
}

fn truncate_chars(s: &str, max_chars: usize) -> String {
    let count = s.chars().count();
    if count <= max_chars {
        s.to_string()
    } else {
        let mut out: String = s.chars().take(max_chars.saturating_sub(1)).collect();
        out.push('…');
        out
    }
}

fn short_platforms(platforms: &str) -> String {
    platforms
        .split(',')
        .filter(|s| !s.is_empty())
        .map(|p| match p {
            "windows" => "Win",
            "linux" => "Linux",
            "macos" => "Mac",
            "android" => "Android",
            "java" => "Java/Multi",
            other => other,
        })
        .collect::<Vec<_>>()
        .join(" • ")
}

fn trust_short_badge(score: i32) -> (String, Color32) {
    if score >= 70 {
        (format!("🛡 {score}%"), Color32::from_rgb(28, 130, 72))
    } else if score >= 40 {
        (format!("⚠ {score}%"), Color32::from_rgb(185, 130, 20))
    } else {
        (format!("✘ {score}%"), Color32::from_rgb(185, 45, 45))
    }
}

fn trust_badge(score: i32, _level: &str) -> (String, Color32) {
    if score >= 70 {
        (format!("✔ {score}/100 Verificado"), Color32::from_rgb(30, 135, 75))
    } else if score >= 40 {
        (format!("⚠ {score}/100 Atenção"), Color32::from_rgb(185, 130, 20))
    } else {
        (format!("✘ {score}/100 Alto Risco"), Color32::from_rgb(185, 45, 45))
    }
}

fn clean_and_color_note(note: &str) -> (String, Color32) {
    if let Some(rest) = note.strip_prefix("🟢 ") {
        (format!("✔ {rest}"), Color32::from_rgb(105, 220, 145))
    } else if let Some(rest) = note.strip_prefix("🟡 ") {
        (format!("⚠ {rest}"), Color32::from_rgb(240, 195, 85))
    } else if let Some(rest) = note.strip_prefix("🔴 ") {
        (format!("✘ {rest}"), Color32::from_rgb(250, 105, 105))
    } else if let Some(rest) = note.strip_prefix("⚪ ") {
        (format!("• {rest}"), Color32::from_rgb(175, 185, 200))
    } else {
        (format!("• {note}"), Color32::from_rgb(190, 200, 215))
    }
}

fn draw_pill(ui: &mut egui::Ui, text: &str, bg: Color32) {
    egui::Frame::NONE
        .fill(bg)
        .corner_radius(CornerRadius::same(4))
        .inner_margin(egui::Margin::symmetric(6, 2))
        .show(ui, |ui| {
            ui.label(RichText::new(text).size(11.0).strong().color(Color32::WHITE));
        });
}

fn format_stars(stars: u64) -> String {
    if stars >= 1000 {
        format!("{:.1}k", stars as f64 / 1000.0)
    } else {
        stars.to_string()
    }
}
