//! Interface gráfica estilo Steam / Microsoft Store / Google Play (egui / eframe).

use crate::db::{self, App};
use crate::downloader::{self, JobState, ScanVerdict};
use crate::forge::Forge;
use crate::github;
use crate::translator::{self, TranslateMode};
use crate::verify;
use eframe::egui::{self, Color32, CornerRadius, RichText, Stroke, Vec2};
use crate::i18n::{tr, Lang};
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};

mod extras;
mod learn;
mod optimizer;
mod phase3;
mod settings;
mod social;
mod rust_lib;
mod terminal_studio;
mod tv;

/// Etapas do login com o GitHub (código no navegador ou token pessoal).
#[derive(Debug, Clone, PartialEq)]
enum LoginState {
    Idle,
    Working,
    WaitingCode { user_code: String, verification_uri: String },
    Failed(String),
}

#[derive(PartialEq, Clone, Copy)]
enum NavSection {
    Discover,
    MadeInRust,
    ReverseEngineering,
    Games,
    Emulators,
    AiAndApps,
    Tools,
    Library,
    Terminal,
    Profile,
    Community,
    Settings,
    CppToRust,
    LearnRust,
    RustLib,
    TerminalStudio,
    Optimizer,
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
        app: Box<App>,
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
        defender_verdict: ScanVerdict,
        defender_msg: String,
        authenticode: String,
    },
    ReleasesLoaded {
        repo: String,
        releases: Vec<github::Release>,
    },
    WindowsScan(Vec<crate::windows_apps::InstalledProgram>),
    WingetResults {
        query: String,
        packages: Result<Vec<crate::winget::WingetPackage>, String>,
    },
    ExternalGames(Vec<crate::launchers::ExternalGame>),
    GhVerified {
        repo: String,
        result: Result<crate::attestation::GhVerify, String>,
    },
    AiTranslated(Result<crate::translator_ai::AiTranslation, String>),
    CrateResolved(Result<String, String>),
    DeviceCode {
        user_code: String,
        verification_uri: String,
    },
    LoggedIn(Result<(String, crate::github_auth::GithubUser), String>),
    ProfilePublished(Result<String, String>),
    ProfileUnpublished(Result<(), String>),
    FriendsLoaded(Result<Vec<crate::social::Friend>, String>),
    LibraryShared(Result<String, String>),
    LibraryImported(Result<db::LibraryImport, String>),
    CommunityList {
        repo: String,
        result: Result<(bool, Vec<crate::community::DiscussionSummary>), String>,
    },
    CommunityThread(Result<crate::community::DiscussionThread, String>),
    LearnRunFinished(crate::learn::RunResult),
    OptimizerScanFinished {
        specs: crate::optimizer::SystemSpecs,
        programs: Vec<crate::optimizer::InstalledProgram>,
        startup: Vec<crate::optimizer::StartupApp>,
        actions: Vec<crate::optimizer::OptimizationItem>,
    },
    OptimizerActionFinished(Result<String, String>),
    TerminalCommandFinished(crate::terminal_studio::TerminalOutput),
    /// Evento de conquista vindo de uma tarefa em segundo plano.
    Stat(&'static str),
    Error(String),
}

/// Coleções curadas da vitrine (todos os repositórios precisam existir no catálogo embutido; há teste).
const COLLECTIONS: &[(&str, &str, &[&str])] = &[
    (
        "🏛 Clássicos dos anos 90, recriados",
        "Motores refeitos pela comunidade para jogar os originais em PCs atuais.",
        &[
            "OpenRCT2/OpenRCT2",
            "OpenRA/OpenRA",
            "diasurgical/DevilutionX",
            "ihhub/fheroes2",
            "vcmi/vcmi",
            "Keriew/augustus",
            "CorsixTH/CorsixTH",
            "OpenMW/openmw",
        ],
    ),
    (
        "🧠 IA no seu PC, sem nuvem",
        "Modelos e ferramentas de IA que rodam localmente.",
        &[
            "ollama/ollama",
            "janhq/jan",
            "nomic-ai/gpt4all",
            "LostRuins/koboldcpp",
            "mozilla-ai/llamafile",
            "upscayl/upscayl",
        ],
    ),
    (
        "🎬 Kit do criador de conteúdo",
        "Gravar, editar, converter e publicar vídeo e áudio.",
        &[
            "obsproject/obs-studio",
            "mltframework/shotcut",
            "mifi/lossless-cut",
            "HandBrake/HandBrake",
            "audacity/audacity",
            "ShareX/ShareX",
        ],
    ),
    (
        "🔐 Privacidade e controle dos seus dados",
        "Senhas, sincronização e acesso remoto sem depender de serviços fechados.",
        &[
            "keepassxreboot/keepassxc",
            "syncthing/syncthing",
            "localsend/localsend",
            "rustdesk/rustdesk",
            "bleachbit/bleachbit",
        ],
    ),
];

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
    /// Apps da lista "Para reinstalar" (biblioteca importada).
    wishlist: Vec<String>,
    /// Releases carregadas no painel "Versões anteriores", por repositório.
    releases_cache: HashMap<String, Vec<github::Release>>,
    versions_open: bool,
    library_path: String,
    /// URL (ou caminho) do catálogo online assinado.
    catalog_url: String,
    lang: Lang,
    windows_programs: Option<Vec<crate::windows_apps::InstalledProgram>>,
    windows_matches: Vec<crate::windows_apps::CatalogMatch>,
    save_dir_input: String,
    restore_confirm: Option<PathBuf>,
    winget_query: String,
    winget_results: Option<Result<Vec<crate::winget::WingetPackage>, String>>,
    winget_confirm: Option<String>,
    uninstall_confirm: Option<String>,
    external_games: Option<Vec<crate::launchers::ExternalGame>>,
    gh_verify_status: Option<(String, Result<crate::attestation::GhVerify, String>)>,
    sandbox_available: bool,
    anthropic_key_input: String,
    ai_busy: bool,
    /// Repositório vindo do crates.io: ao verificar, entra no catálogo como ferramenta de terminal.
    pending_cli: Option<String>,
    crate_input: String,
    tv_mode: bool,
    tv_index: usize,
    gamepads: Option<gilrs::Gilrs>,
    tv_stick: (i8, i8),
    login_state: LoginState,
    login_cancel: Arc<std::sync::atomic::AtomicBool>,
    oauth_client_id: String,
    pat_input: String,
    /// Token do login (Gerenciador de Credenciais do Windows).
    auth_token: Option<String>,
    github_user: Option<crate::github_auth::GithubUser>,
    profile_gist_id: Option<String>,
    unpublish_confirm: bool,
    community_repo: String,
    /// `None` = comunidade geral da RRS; `Some(repo)` = conversas do app.
    community_target: Option<String>,
    community_list: Option<(String, Result<(bool, Vec<crate::community::DiscussionSummary>), String>)>,
    community_thread: Option<Result<crate::community::DiscussionThread, String>>,
    community_visited: bool,
    friends: Option<Result<Vec<crate::social::Friend>, String>>,
    import_link_input: String,
    progress: Option<crate::achievements::Progress>,
    /// Avisos de conquista (texto, instante em que apareceu).
    toasts: Vec<(String, Option<f64>)>,

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
    defender_scan_status: Option<(String, ScanVerdict)>,
    authenticode_status: Option<String>,

    // Adicionar repositório avulso
    add_repo_input: String,

    // Configurações (Tokens opcionais)
    github_token_input: String,
    vt_key_input: String,

    // Tradutor C/C++ -> Rust embutido
    cpp_input: String,
    rust_output: String,
    translate_mode: TranslateMode,
    translate_notes: Vec<String>,
    rustc_check_status: Option<Result<String, String>>,
    file_io_path: String,

    // Estúdio Aprender & Programar em Rust
    learn_lesson_idx: usize,
    learn_code_input: String,
    learn_output: Option<crate::learn::RunResult>,
    learn_is_running: bool,
    learn_show_cheatsheet: bool,
    learn_level_filter: usize,

    // Otimizador do PC e Diagnóstico de Sistema
    optimizer_specs: Option<crate::optimizer::SystemSpecs>,
    optimizer_programs: Option<Vec<crate::optimizer::InstalledProgram>>,
    optimizer_startup: Option<Vec<crate::optimizer::StartupApp>>,
    optimizer_actions: Vec<crate::optimizer::OptimizationItem>,
    optimizer_is_scanning: bool,
    optimizer_program_search: String,
    optimizer_tab: usize,
    external_store_filter: Option<crate::launchers::Store>,

    // Modo de Desempenho
    performance_mode: crate::performance::PerformanceMode,

    // Biblioteca Rust (std & Crates)
    rust_lib_selected_idx: usize,
    rust_lib_search: String,
    rust_lib_category_filter: String,
    rust_lib_active_tab: usize,

    // Terminal Integrado & Espelho de IDEs
    terminal_logs: Vec<(crate::terminal_studio::TerminalLineKind, String)>,
    terminal_working_dir: PathBuf,
    terminal_cmd_input: String,
    terminal_is_running: bool,
    terminal_repl_input: String,
    cargo_new_name: String,
    cargo_new_is_bin: bool,
    rust_env_info: Option<crate::terminal_studio::RustEnvInfo>,

    // Temas e Aparência
    theme: crate::theme::AppTheme,
    accent_color: crate::theme::AccentColor,

    // Tarefas em segundo plano
    downloads: downloader::DownloadManager,
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
        configure_fonts(&cc.egui_ctx);
        let library_path = catalog_path
            .parent()
            .unwrap_or_else(|| std::path::Path::new("."))
            .join(db::LIBRARY_FILE_NAME)
            .to_string_lossy()
            .to_string();

        let mut theme = crate::theme::AppTheme::Antigravity;
        let mut accent_color = crate::theme::AccentColor::NeonCyan;
        let mut performance_mode = crate::performance::PerformanceMode::Normal;
        let mut github_token_input = std::env::var("GITHUB_TOKEN").unwrap_or_default();
        let mut vt_key_input = std::env::var("VIRUSTOTAL_API_KEY").unwrap_or_default();
        let mut catalog_url = String::new();
        let mut catalog_due = false;
        let mut lang = Lang::Pt;
        let mut anthropic_key_input = std::env::var("ANTHROPIC_API_KEY").unwrap_or_default();
        let mut oauth_client_id = String::new();
        let mut github_user = None;
        let mut profile_gist_id = None;
        let mut community_repo = String::new();

        if let Ok(conn) = db::open(&db_path) {
            if let Some(t) = db::get_setting(&conn, crate::theme::SETTING_THEME) {
                theme = crate::theme::AppTheme::from_id(&t);
            }
            if let Some(a) = db::get_setting(&conn, crate::theme::SETTING_ACCENT) {
                accent_color = crate::theme::AccentColor::from_id(&a);
            }
            if let Some(p) = db::get_setting(&conn, crate::performance::SETTING_PERFORMANCE) {
                performance_mode = crate::performance::PerformanceMode::from_id(&p);
            }
            if anthropic_key_input.is_empty() {
                anthropic_key_input = db::get_setting(&conn, crate::translator_ai::SETTING_KEY).unwrap_or_default();
            }
            oauth_client_id = db::get_setting(&conn, crate::github_auth::SETTING_CLIENT_ID).unwrap_or_default();
            github_user = db::get_setting(&conn, crate::github_auth::SETTING_PROFILE_CACHE)
                .and_then(|j| serde_json::from_str::<crate::github_auth::GithubUser>(&j).ok());
            profile_gist_id = db::get_setting(&conn, crate::social::SETTING_PROFILE_GIST);
            community_repo = db::get_setting(&conn, crate::community::SETTING_COMMUNITY_REPO).unwrap_or_default();
            lang = Lang::from_setting(db::get_setting(&conn, crate::i18n::SETTING_LANG).as_deref());
            catalog_url = db::get_setting(&conn, crate::catalog_sign::SETTING_URL).unwrap_or_default();
            // Catálogo online: confere no máximo uma vez por dia ao abrir
            let last = db::get_setting(&conn, "catalog_last_check")
                .and_then(|t| chrono::DateTime::parse_from_rfc3339(&t).ok())
                .map(|t| t.with_timezone(&chrono::Utc));
            catalog_due = !catalog_url.is_empty()
                && last.is_none_or(|t| chrono::Utc::now() - t > chrono::Duration::hours(24));
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

        theme.apply(&cc.egui_ctx, accent_color);

        let default_cpp = translator::EXAMPLES[0].cpp_code.to_string();
        let initial_tr = translator::translate_c_cpp_to_rust(&default_cpp, TranslateMode::SafeRust);

        let mut gui = Self {
            db_path,
            catalog_path,
            rt,
            apps: Vec::new(),
            categories: Vec::new(),
            wishlist: Vec::new(),
            releases_cache: HashMap::new(),
            versions_open: false,
            library_path,
            catalog_url,
            lang,
            windows_programs: None,
            windows_matches: Vec::new(),
            save_dir_input: String::new(),
            restore_confirm: None,
            winget_query: String::new(),
            winget_results: None,
            winget_confirm: None,
            uninstall_confirm: None,
            external_games: None,
            gh_verify_status: None,
            sandbox_available: crate::sandbox::available(),
            anthropic_key_input,
            ai_busy: false,
            pending_cli: None,
            crate_input: String::new(),
            tv_mode: false,
            tv_index: 0,
            gamepads: None,
            tv_stick: (0, 0),
            login_state: LoginState::Idle,
            login_cancel: Arc::new(std::sync::atomic::AtomicBool::new(false)),
            oauth_client_id,
            pat_input: String::new(),
            auth_token: crate::github_auth::load_token(),
            github_user,
            profile_gist_id,
            unpublish_confirm: false,
            community_repo,
            community_target: None,
            community_list: None,
            community_thread: None,
            community_visited: false,
            friends: None,
            import_link_input: String::new(),
            progress: None,
            toasts: Vec::new(),
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
            github_token_input,
            vt_key_input,
            cpp_input: default_cpp,
            rust_output: initial_tr.rust_code,
            translate_mode: TranslateMode::SafeRust,
            translate_notes: initial_tr.notes,
            rustc_check_status: None,
            file_io_path: String::new(),
            learn_lesson_idx: 0,
            learn_code_input: crate::learn::LESSONS[0].initial_code.to_string(),
            learn_output: None,
            learn_is_running: false,
            learn_show_cheatsheet: false,
            learn_level_filter: 0,
            optimizer_specs: None,
            optimizer_programs: None,
            optimizer_startup: None,
            optimizer_actions: Vec::new(),
            optimizer_is_scanning: false,
            optimizer_program_search: String::new(),
            optimizer_tab: 0,
            external_store_filter: None,
            performance_mode,
            rust_lib_selected_idx: 0,
            rust_lib_search: String::new(),
            rust_lib_category_filter: "todas".to_string(),
            rust_lib_active_tab: 0,
            terminal_logs: Vec::new(),
            terminal_working_dir: PathBuf::new(),
            terminal_cmd_input: String::new(),
            terminal_is_running: false,
            terminal_repl_input: String::new(),
            cargo_new_name: String::new(),
            cargo_new_is_bin: true,
            rust_env_info: None,
            theme,
            accent_color,
            downloads: downloader::DownloadManager::default(),
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
        if catalog_due {
            gui.spawn_catalog_update();
        }
        gui
    }

    /// Altera o tema visual da loja e persiste a escolha no banco de dados.
    pub fn set_theme(&mut self, ctx: &egui::Context, theme: crate::theme::AppTheme) {
        self.theme = theme;
        self.theme.apply(ctx, self.accent_color);
        if let Ok(conn) = db::open(&self.db_path) {
            let _ = db::set_setting(&conn, crate::theme::SETTING_THEME, theme.id());
        }
        self.status_msg = format!("{} {}", self.tr("Tema aplicado:"), self.tr(theme.label()));
        self.status_is_error = false;
    }

    /// Altera a cor de destaque (acentos) e persiste a escolha no banco de dados.
    pub fn set_accent_color(&mut self, ctx: &egui::Context, accent: crate::theme::AccentColor) {
        self.accent_color = accent;
        self.theme.apply(ctx, self.accent_color);
        if let Ok(conn) = db::open(&self.db_path) {
            let _ = db::set_setting(&conn, crate::theme::SETTING_ACCENT, accent.id());
        }
        self.status_msg = format!("{} {}", self.tr("Cor de destaque aplicada:"), self.tr(accent.label()));
        self.status_is_error = false;
    }

    /// Altera o modo de desempenho (Eco, Normal, Performance) e persiste no banco de dados.
    pub fn set_performance_mode(&mut self, mode: crate::performance::PerformanceMode) {
        self.performance_mode = mode;
        if let Ok(conn) = db::open(&self.db_path) {
            let _ = db::set_setting(&conn, crate::performance::SETTING_PERFORMANCE, mode.id());
        }
        self.status_msg = format!("{} {}", self.tr("Modo de desempenho alterado para:"), self.tr(mode.label()));
        self.status_is_error = false;
    }

    /// Retorna a paleta calculada para o tema e acento atuais.
    pub fn palette(&self) -> crate::theme::ThemePalette {
        self.theme.palette(self.accent_color)
    }

    /// Baixa o catálogo online assinado e importa se a assinatura, a versão e a validade conferirem.
    fn spawn_catalog_update(&mut self) {
        let location = self.catalog_url.trim().to_string();
        if location.is_empty() || self.busy_task.is_some() {
            return;
        }
        self.busy_task = Some("Conferindo catálogo online...".into());
        let db_path = self.db_path.clone();
        let catalog_path = self.catalog_path.clone();
        let events = Arc::clone(&self.bg_events);
        self.rt.spawn(async move {
            let res = async {
                let (bytes, sig) = crate::catalog_sign::fetch_signed(&location).await?;
                let conn = db::open(&db_path)?;
                let _ = db::set_setting(&conn, "catalog_last_check", &chrono::Utc::now().to_rfc3339());
                let n = crate::catalog_sign::apply_signed(&conn, &catalog_path, &bytes, &sig)?;
                Ok::<_, anyhow::Error>((n, sig.versao))
            }
            .await;
            let ev = match res {
                Ok((n, versao)) => BgEvent::SyncFinished {
                    updated: n,
                    message: format!("✔ Catálogo online versão {versao} conferido (assinatura válida)"),
                },
                Err(e) => BgEvent::Error(format!("Catálogo online não atualizado: {e:#}")),
            };
            events.lock().unwrap().push(ev);
        });
    }

    /// Token para a API do GitHub: o informado em Chaves de API ou, se vazio, o do login.
    fn api_token(&self) -> String {
        if !self.github_token_input.trim().is_empty() {
            self.github_token_input.clone()
        } else {
            self.auth_token.clone().unwrap_or_default()
        }
    }

    /// Texto no idioma escolhido (ver `i18n.rs`).
    fn tr(&self, pt: &str) -> String {
        crate::i18n::tr(self.lang, pt)
    }

    /// Resumo do app: o curado em português, ou a descrição do GitHub (em geral em inglês) no modo inglês.
    fn summary_of(&self, app: &App) -> String {
        if self.lang == Lang::En && !app.description.trim().is_empty() {
            app.description.clone()
        } else {
            app.display_summary().to_string()
        }
    }

    /// Pasta da RRS (catálogo, banco e `downloads/`).
    fn base_dir(&self) -> PathBuf {
        self.catalog_path
            .parent()
            .filter(|p| !p.as_os_str().is_empty())
            .map(|p| p.to_path_buf())
            .unwrap_or_else(|| PathBuf::from("."))
    }

    /// Caminho real de um app instalado (o banco guarda relativo à pasta da RRS no modo portátil).
    fn installed_path(&self, stored: &str) -> String {
        db::resolve_stored_path(&self.base_dir(), stored)
            .to_string_lossy()
            .to_string()
    }

    fn reload_from_db(&mut self) {
        if let Ok(conn) = db::open(&self.db_path) {
            if let Ok(list) = db::list_all(&conn) {
                self.apps = list;
            }
            if let Ok(w) = db::wishlist(&conn) {
                self.wishlist = w;
            }
        }
        self.refresh_windows_matches();
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
        if self.selected_repo.as_deref() != Some(full_name) {
            self.versions_open = false;
            self.load_save_dir_input(full_name);
        }
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
                    NavSection::MadeInRust => a.is_rust(),
                    NavSection::Terminal => k == "cli",
                    NavSection::Profile | NavSection::Community | NavSection::Settings => false,
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
                    NavSection::CppToRust | NavSection::LearnRust | NavSection::RustLib | NavSection::TerminalStudio | NavSection::Optimizer => true,
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
        // Fila de downloads: o registro no banco já foi feito pela própria tarefa; aqui só atualiza a tela.
        if self.downloads.has_active() {
            ctx.request_repaint_after(std::time::Duration::from_millis(80));
        }
        let finished = self.downloads.take_finished();
        if !finished.is_empty() {
            self.reload_from_db();
            self.check_achievements();
            for job in finished {
                match job.state {
                    JobState::Finished { message } => {
                        self.status_msg = message;
                        self.status_is_error = false;
                    }
                    JobState::Failed { error } => {
                        self.status_msg = format!("Falha no download de {} {}: {error}", job.repo, job.tag);
                        self.status_is_error = true;
                    }
                    _ => {}
                }
            }
        }

        let events: Vec<BgEvent> = self.bg_events.lock().unwrap().drain(..).collect();
        for ev in events {
            match ev {
                BgEvent::Status(s) => {
                    self.status_msg = s;
                    self.status_is_error = false;
                }
                BgEvent::AppUpdated { mut app, message } => {
                    self.record_stat(crate::achievements::STAT_VERIFIES);
                    let name = app.full_name.clone();
                    // Veio do crates.io: entra no catálogo como ferramenta de terminal
                    if self.pending_cli.as_deref().is_some_and(|p| p.eq_ignore_ascii_case(&name)) {
                        self.pending_cli = None;
                        if app.kind.is_none() {
                            app.kind = Some("cli".into());
                            app.category = Some("Terminal • Rust".into());
                        }
                        self.section = NavSection::Terminal;
                        self.record_stat(crate::achievements::STAT_CLI);
                    }
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
                BgEvent::SecurityScanResult { repo, defender_verdict, defender_msg, authenticode } => {
                    self.record_stat(crate::achievements::STAT_DEFENDER);
                    if self.selected_repo.as_deref() == Some(repo.as_str()) {
                        self.defender_scan_status = Some((defender_msg, defender_verdict));
                        self.authenticode_status = Some(authenticode);
                    }
                    self.status_msg = if defender_verdict == ScanVerdict::NotVerified {
                        format!("Varredura de {} não concluída: arquivo NÃO verificado.", friendly_title(&repo))
                    } else {
                        format!("Varredura de {} concluída!", friendly_title(&repo))
                    };
                    self.status_is_error = defender_verdict != ScanVerdict::Clean;
                    self.busy_task = None;
                }
                BgEvent::WindowsScan(programs) => {
                    self.record_stat(crate::achievements::STAT_WINDOWS_SCAN);
                    self.busy_task = None;
                    self.windows_programs = Some(programs);
                    self.refresh_windows_matches();
                    self.status_msg = format!(
                        "✔ {} {}",
                        self.windows_matches.len(),
                        self.tr("apps do catálogo encontrados no Windows")
                    );
                    self.status_is_error = false;
                }
                BgEvent::WingetResults { query, packages } => {
                    self.busy_task = None;
                    self.winget_query = query;
                    self.winget_results = Some(packages);
                }
                BgEvent::ExternalGames(games) => {
                    self.busy_task = None;
                    self.status_msg = format!("✔ {} {}", games.len(), self.tr("jogos encontrados em outras lojas"));
                    self.status_is_error = false;
                    self.external_games = Some(games);
                }
                BgEvent::GhVerified { repo, result } => {
                    self.busy_task = None;
                    self.gh_verify_status = Some((repo, result));
                }
                BgEvent::AiTranslated(res) => self.apply_ai_result(res),
                BgEvent::DeviceCode { user_code, verification_uri } => {
                    self.login_state = LoginState::WaitingCode { user_code, verification_uri };
                }
                BgEvent::LoggedIn(res) => match res {
                    Ok((token, user)) => {
                        if let Ok(conn) = db::open(&self.db_path) {
                            if let Ok(j) = serde_json::to_string(&user) {
                                let _ = db::set_setting(&conn, crate::github_auth::SETTING_PROFILE_CACHE, &j);
                            }
                        }
                        self.status_msg = format!("✔ {} @{}", self.tr("Conectado ao GitHub como"), user.login);
                        self.status_is_error = false;
                        self.auth_token = Some(token);
                        self.github_user = Some(user);
                        self.login_state = LoginState::Idle;
                        self.section = NavSection::Profile;
                    }
                    Err(e) => self.login_state = LoginState::Failed(e),
                },
                BgEvent::ProfilePublished(res) => {
                    self.busy_task = None;
                    match res {
                        Ok(id) => {
                            if let Ok(conn) = db::open(&self.db_path) {
                                let _ = db::set_setting(&conn, crate::social::SETTING_PROFILE_GIST, &id);
                            }
                            let first = self.profile_gist_id.is_none();
                            self.profile_gist_id = Some(id);
                            self.status_msg = self.tr("✔ Perfil publicado no seu GitHub (Gist público).");
                            self.status_is_error = false;
                            if first {
                                self.record_stat(crate::achievements::STAT_PROFILE_PUBLISHED);
                            }
                        }
                        Err(e) => {
                            self.status_msg = e;
                            self.status_is_error = true;
                        }
                    }
                }
                BgEvent::ProfileUnpublished(res) => {
                    self.busy_task = None;
                    match res {
                        Ok(()) => {
                            if let Ok(conn) = db::open(&self.db_path) {
                                let _ = db::set_setting(&conn, crate::social::SETTING_PROFILE_GIST, "");
                            }
                            self.profile_gist_id = None;
                            self.status_msg = self.tr("✔ Perfil despublicado (o Gist foi apagado da sua conta).");
                            self.status_is_error = false;
                        }
                        Err(e) => {
                            self.status_msg = e;
                            self.status_is_error = true;
                        }
                    }
                }
                BgEvent::FriendsLoaded(res) => {
                    self.busy_task = None;
                    self.friends = Some(res);
                }
                BgEvent::LibraryShared(res) => {
                    self.busy_task = None;
                    match res {
                        Ok(url) => {
                            ctx.copy_text(url.clone());
                            self.status_msg = format!("🔗 {} {url}", self.tr("Link copiado:"));
                            self.status_is_error = false;
                            self.record_stat(crate::achievements::STAT_LIBRARY_EXPORT);
                        }
                        Err(e) => {
                            self.status_msg = e;
                            self.status_is_error = true;
                        }
                    }
                }
                BgEvent::LibraryImported(res) => {
                    self.busy_task = None;
                    self.show_import_result(res.map_err(|e| anyhow::anyhow!(e)));
                }
                BgEvent::CommunityList { repo, result } => {
                    self.busy_task = None;
                    self.community_list = Some((repo, result));
                }
                BgEvent::CommunityThread(res) => {
                    self.busy_task = None;
                    self.community_thread = Some(res);
                }
                BgEvent::LearnRunFinished(res) => {
                    if res.success {
                        self.record_stat(crate::achievements::STAT_LEARN_RUN);
                        self.status_msg = self.tr("✔ Código Rust compilado e executado com sucesso!");
                        self.status_is_error = false;
                    } else {
                        self.status_msg = self.tr("O compilador rustc reportou erros no código.");
                        self.status_is_error = true;
                    }
                    self.busy_task = None;
                    self.learn_is_running = false;
                    self.learn_output = Some(res);
                }
                BgEvent::OptimizerScanFinished {
                    specs,
                    programs,
                    startup,
                    actions,
                } => {
                    self.optimizer_is_scanning = false;
                    self.optimizer_specs = Some(specs);
                    self.optimizer_programs = Some(programs);
                    self.optimizer_startup = Some(startup);
                    self.optimizer_actions = actions;
                    self.busy_task = None;
                    self.status_msg = self.tr("Diagnóstico do PC e especificações atualizadas.");
                    self.status_is_error = false;
                }
                BgEvent::OptimizerActionFinished(res) => {
                    self.busy_task = None;
                    match res {
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
                BgEvent::TerminalCommandFinished(out) => {
                    self.terminal_is_running = false;
                    self.busy_task = None;
                    if !out.stdout.is_empty() {
                        for line in out.stdout.lines() {
                            self.terminal_logs.push((
                                crate::terminal_studio::TerminalLineKind::Stdout,
                                line.to_string(),
                            ));
                        }
                    }
                    if !out.stderr.is_empty() {
                        for line in out.stderr.lines() {
                            self.terminal_logs.push((
                                crate::terminal_studio::TerminalLineKind::Stderr,
                                line.to_string(),
                            ));
                        }
                    }
                    let status_txt = if out.success {
                        format!("✔ {} ({:.2}s)", self.tr("Comando concluído"), (out.elapsed_ms as f64) / 1000.0)
                    } else {
                        format!("❌ {} ({:.2}s)", self.tr("Comando falhou"), (out.elapsed_ms as f64) / 1000.0)
                    };
                    self.terminal_logs.push((
                        crate::terminal_studio::TerminalLineKind::SystemInfo,
                        status_txt.clone(),
                    ));
                    self.status_msg = status_txt;
                    self.status_is_error = !out.success;
                }
                BgEvent::Stat(key) => self.record_stat(key),
                BgEvent::CrateResolved(res) => {
                    self.busy_task = None;
                    match res {
                        Ok(repo) => {
                            self.pending_cli = Some(repo.clone());
                            self.spawn_fetch_or_verify(repo, false);
                        }
                        Err(e) => {
                            self.status_msg = e;
                            self.status_is_error = true;
                        }
                    }
                }
                BgEvent::ReleasesLoaded { repo, releases } => {
                    self.busy_task = None;
                    self.status_msg = format!(
                        "{} versões de {} carregadas do GitHub.",
                        releases.len(),
                        friendly_title(&repo)
                    );
                    self.status_is_error = false;
                    self.releases_cache.insert(repo, releases);
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

        // Pacing de renderização e fluidez de acordo com o perfil de desempenho
        ctx.request_repaint_after(self.performance_mode.frame_interval());
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

        let gh_token = self.api_token();
        let vt_key = self.vt_key_input.clone();
        let events = Arc::clone(&self.bg_events);
        let existing = self
            .apps
            .iter()
            .find(|a| a.full_name.eq_ignore_ascii_case(&clean))
            .cloned();

        self.rt.spawn(async move {
            let res: anyhow::Result<(App, String)> = async {
                let gh = Forge::new(Some(&gh_token))?;
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

                // Atestado de build do GitHub para o instalador recomendado (presença, sem conferir assinatura)
                let attestation = match github::pick_best_asset(&assets).and_then(|i| assets[i].digest.clone()) {
                    Some(digest) => match gh.has_attestation(&repo.full_name, &digest).await {
                        Ok(true) => github::ATTESTATION_PRESENT.to_string(),
                        Ok(false) => github::ATTESTATION_ABSENT.to_string(),
                        Err(_) => String::new(),
                    },
                    None => String::new(),
                };

                let rep = verify::evaluate(&repo, release.as_ref(), contributors, &vt_results);
                let (latest_tag, released_at) = release
                    .map(|r| (r.tag_name, r.published_at.unwrap_or_default()))
                    .unwrap_or_default();
                let app = App {
                    full_name: repo.full_name.clone(),
                    description: repo.description.unwrap_or_default(),
                    url: repo.html_url,
                    stars: repo.stargazers_count,
                    license: repo.license.and_then(|l| l.spdx_id).unwrap_or_default(),
                    topics: repo.topics.join(","),
                    language: repo.language.unwrap_or_default(),
                    created_at: repo.created_at,
                    pushed_at: repo.pushed_at.unwrap_or_default(),
                    fork: repo.fork,
                    archived: repo.archived,
                    has_issues: repo.has_issues,
                    latest_tag,
                    released_at,
                    platforms: plats.join(","),
                    assets,
                    kind: existing.as_ref().and_then(|e| e.kind.clone()),
                    category: existing.as_ref().and_then(|e| e.category.clone()),
                    tags: existing.as_ref().and_then(|e| e.tags.clone()),
                    summary: existing.as_ref().and_then(|e| e.summary.clone()),
                    trust_score: rep.score,
                    trust_level: rep.level.to_string(),
                    trust_notes: rep.notes,
                    attestation,
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
                    events.lock().unwrap().push(BgEvent::AppUpdated { app: Box::new(app), message });
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
        let gh_token = self.api_token();
        let db_path = self.db_path.clone();
        let catalog_path = self.catalog_path.clone();
        let events = Arc::clone(&self.bg_events);

        self.rt.spawn(async move {
            let res: anyhow::Result<usize> = async {
                let gh = Forge::new(Some(&gh_token))?;
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
                            if app.latest_tag != rel.tag_name {
                                app.attestation.clear();
                            }
                            app.latest_tag = rel.tag_name;
                            app.released_at = rel.published_at.unwrap_or_default();
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
        // Cada atualização entra na fila (até 2 ao mesmo tempo) e se registra no banco ao terminar.
        let mut queued = 0;
        for app in outdated {
            if let Some(best_idx) = github::pick_best_asset(&app.assets) {
                let asset = app.assets[best_idx].clone();
                if self.start_download(app.full_name.clone(), app.latest_tag.clone(), asset) {
                    queued += 1;
                }
            }
        }
        self.status_msg = format!("⬇ {queued} atualização(ões) na fila de downloads.");
        self.status_is_error = false;
    }

    /// Abre um app instalado, mede o tempo de jogo e faz backup dos saves quando ele fecha
    /// (se a pasta de saves estiver configurada). Usado pelo botão "Jogar" e pelo Modo TV.
    fn launch_installed(&mut self, app: &App) {
        let Some(inst) = &app.installed else { return };
        let path = self.installed_path(&inst.file_path);
        let repo_name = app.full_name.clone();
        let events = Arc::clone(&self.bg_events);
        let save_dir = db::open(&self.db_path)
            .ok()
            .and_then(|c| crate::saves::save_dir(&c, &repo_name));
        let base_dir = self.base_dir();
        self.status_msg = format!("▶ {}...", friendly_title(&repo_name));
        self.status_is_error = false;
        std::thread::spawn(move || {
            let start = std::time::Instant::now();
            match downloader::launch_installed_app(&path) {
                Ok(Some(mut child)) => {
                    let _ = child.wait();
                    let elapsed = start.elapsed().as_secs();
                    if let Some(dir) = save_dir {
                        let ev = match crate::saves::backup(&base_dir, &repo_name, std::path::Path::new(&dir), None) {
                            Ok(zip) => {
                                events.lock().unwrap().push(BgEvent::Stat(crate::achievements::STAT_SAVES));
                                BgEvent::Status(format!("💾 Backup dos saves: {}", zip.display()))
                            }
                            Err(e) => BgEvent::Error(format!("Backup dos saves não feito: {e:#}")),
                        };
                        events.lock().unwrap().push(ev);
                    }
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

    /// Busca no GitHub as releases do repositório para o bloco "Versões anteriores".
    fn spawn_load_releases(&mut self, repo: String) {
        if self.busy_task.is_some() {
            return;
        }
        self.busy_task = Some(format!("Carregando versões de {}...", friendly_title(&repo)));
        let gh_token = self.api_token();
        let events = Arc::clone(&self.bg_events);
        self.rt.spawn(async move {
            let res = async { Forge::new(Some(&gh_token))?.releases(&repo, 15).await }.await;
            let ev = match res {
                Ok(releases) => BgEvent::ReleasesLoaded { repo, releases },
                Err(e) => BgEvent::Error(format!("Não foi possível carregar as versões: {e:#}")),
            };
            events.lock().unwrap().push(ev);
        });
    }

    /// Coloca um instalador (de qualquer versão) na fila; ele confere o SHA-256 e se registra ao terminar.
    /// Devolve `false` se essa versão já estava na fila.
    fn start_download(&mut self, repo: String, tag: String, asset: github::Asset) -> bool {
        let queued = self
            .downloads
            .enqueue(&self.rt, self.db_path.clone(), self.base_dir(), repo.clone(), tag.clone(), asset)
            .is_some();
        if queued {
            self.status_msg = format!("⬇ {} {tag} na fila de downloads.", friendly_title(&repo));
            self.status_is_error = false;
        }
        queued
    }
}

impl eframe::App for StoreGui {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        self.process_background(ctx);
        if self.tv_mode {
            self.render_tv(ctx);
            return;
        }

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
        let rust_count = self.apps.iter().filter(|a| a.is_rust()).count();
        let cli_count = self.apps.iter().filter(|a| a.kind.as_deref() == Some("cli")).count();

        // ─── BARRA LATERAL ESQUERDA (Estilo Steam / Microsoft Store) ───────
        let p = self.palette();
        egui::SidePanel::left("left_nav")
            .resizable(false)
            .exact_width(210.0)
            .frame(
                egui::Frame::NONE
                    .fill(p.bg_sidebar)
                    .stroke(Stroke::new(1.0_f32, p.border_subtle))
                    .inner_margin(egui::Margin::symmetric(12, 14)),
            )
            .show(ctx, |ui| {
                ui.horizontal(|ui| {
                    crate::brand::logo_badge(ui, 17.0);
                    ui.add_space(2.0);
                    ui.label(
                        RichText::new("Riva Rust Store")
                            .size(15.5)
                            .strong()
                            .color(Color32::from_rgb(95, 180, 255)),
                    );
                });
                ui.add_space(2.0);
                draw_pill(
                    ui,
                    &tr(self.lang, "🛡 CATÁLOGO VERIFICADO"),
                    Color32::from_rgb(24, 105, 65),
                );
                ui.add_space(8.0);
                self.render_profile_chip(ui);

                ui.add_space(16.0);
                ui.label(
                    RichText::new(tr(self.lang, "EXPLORAR LOJA"))
                        .size(11.0)
                        .strong()
                        .color(Color32::from_rgb(120, 130, 150)),
                );
                ui.add_space(4.0);

                nav_button(
                    ui,
                    &mut self.section,
                    NavSection::Discover,
                    &format!("🏠  {} ({})", tr(self.lang, "Vitrine & Destaques"), self.apps.len()),
                );
                nav_button(
                    ui,
                    &mut self.section,
                    NavSection::MadeInRust,
                    &format!("🦀  {} ({rust_count})", tr(self.lang, "Feito em Rust")),
                );
                nav_button(
                    ui,
                    &mut self.section,
                    NavSection::ReverseEngineering,
                    &format!("🧬  {} ({re_count})", tr(self.lang, "Engenharia Reversa")),
                );
                nav_button(
                    ui,
                    &mut self.section,
                    NavSection::Games,
                    &format!("🎮  {} ({games_count})", tr(self.lang, "Jogos & Ports")),
                );
                nav_button(
                    ui,
                    &mut self.section,
                    NavSection::Emulators,
                    &format!("🕹  {} ({emu_count})", tr(self.lang, "Emuladores & Retrô")),
                );
                nav_button(
                    ui,
                    &mut self.section,
                    NavSection::AiAndApps,
                    &format!("🤖  {} ({apps_count})", tr(self.lang, "IA Local & Apps")),
                );
                nav_button(
                    ui,
                    &mut self.section,
                    NavSection::Tools,
                    &format!("🛠  {} ({tools_count})", tr(self.lang, "Ferramentas & RE")),
                );
                nav_button(
                    ui,
                    &mut self.section,
                    NavSection::Terminal,
                    &format!("⌨  {} ({cli_count})", tr(self.lang, "Terminal (Rust)")),
                );

                ui.add_space(14.0);
                ui.label(
                    RichText::new(tr(self.lang, "SEU PC"))
                        .size(11.0)
                        .strong()
                        .color(Color32::from_rgb(120, 130, 150)),
                );
                ui.add_space(4.0);

                let lib_label = if updates_count > 0 {
                    format!("📦  {} ({installed_count})  ⬆{updates_count}", tr(self.lang, "Biblioteca"))
                } else if !self.wishlist.is_empty() {
                    format!("📦  {} ({installed_count})  🔁{}", tr(self.lang, "Biblioteca"), self.wishlist.len())
                } else {
                    format!("📦  {} ({installed_count})", tr(self.lang, "Minha Biblioteca"))
                };
                nav_button(ui, &mut self.section, NavSection::Library, &lib_label);
                nav_button(ui, &mut self.section, NavSection::Optimizer, &tr(self.lang, "⚡  Otimizador do PC"));
                nav_button(ui, &mut self.section, NavSection::Profile, &tr(self.lang, "🏆  Perfil & Conquistas"));
                nav_button(ui, &mut self.section, NavSection::Community, &tr(self.lang, "💬  Comunidade"));

                ui.add_space(12.0);
                ui.label(
                    RichText::new(tr(self.lang, "ESTÚDIO RUST"))
                        .size(11.0)
                        .strong()
                        .color(Color32::from_rgb(245, 145, 80)),
                );
                ui.add_space(4.0);
                nav_button(
                    ui,
                    &mut self.section,
                    NavSection::CppToRust,
                    &tr(self.lang, "🦀  Tradutor C/C++ ➔ Rust"),
                );
                nav_button(
                    ui,
                    &mut self.section,
                    NavSection::LearnRust,
                    &tr(self.lang, "📖  Aprender & Programar"),
                );
                nav_button(
                    ui,
                    &mut self.section,
                    NavSection::RustLib,
                    &tr(self.lang, "📚  Biblioteca Rust (std & Crates)"),
                );
                nav_button(
                    ui,
                    &mut self.section,
                    NavSection::TerminalStudio,
                    &tr(self.lang, "💻  Terminal & IDEs (Espelho)"),
                );

                ui.add_space(14.0);
                ui.separator();
                ui.add_space(10.0);

                ui.checkbox(&mut self.only_trusted, tr(self.lang, "🛡 Só Verificados (70+)"));
                ui.add_space(6.0);
                if ui
                    .add(
                        egui::Button::new(RichText::new(tr(self.lang, "🎮 Modo TV (tela cheia)")).strong())
                            .min_size(Vec2::new(ui.available_width(), 28.0)),
                    )
                    .on_hover_text(tr(self.lang, "Biblioteca em tela cheia para jogar com controle ou teclado (inclui Steam, Epic e GOG)."))
                    .clicked()
                {
                    self.enter_tv_mode(ctx);
                }
                ui.add_space(6.0);

                let btn_text = if let Some(task) = &self.busy_task {
                    task.clone()
                } else {
                    tr(self.lang, "🔄 Checar Atualizações")
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
                nav_button(ui, &mut self.section, NavSection::Settings, &tr(self.lang, "⚙  Configurações"));

                ui.add_space(4.0);
                if ui
                    .add(
                        egui::Button::new(
                            RichText::new(tr(self.lang, "📌 Instalar RRS no PC"))
                                .size(12.0)
                                .color(Color32::from_rgb(155, 225, 180)),
                        )
                        .fill(Color32::from_rgb(24, 52, 38))
                        .min_size(Vec2::new(ui.available_width(), 26.0)),
                    )
                    .on_hover_text(
                        tr(self.lang, "Instala a RRS no Windows e cria atalhos na Área de Trabalho, Menu Iniciar e terminal (comando 'rrs')."),
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
                    .fill(p.bg_topbar)
                    .stroke(Stroke::new(1.0_f32, p.border_subtle))
                    .inner_margin(egui::Margin::symmetric(16, 10)),
            )
            .show(ctx, |ui| {
                // Quebra em duas linhas em janelas estreitas (antes o campo de adicionar repo ficava cortado)
                ui.horizontal_wrapped(|ui| {
                    ui.label("🔎");
                    ui.add(
                        egui::TextEdit::singleline(&mut self.search)
                            .hint_text(tr(self.lang, "Buscar jogos, ports de engenharia reversa, emuladores, IA, apps..."))
                            .desired_width(260.0),
                    );
                    if !self.search.is_empty() && icon_button(ui, "✖", &tr(self.lang, "Limpar busca")).clicked() {
                        self.search.clear();
                    }

                    ui.separator();

                    ui.label(RichText::new(tr(self.lang, "Plataforma:")).size(12.0).color(Color32::GRAY));
                    egui::ComboBox::from_id_salt("plat_combo")
                        .selected_text(&self.platform_filter)
                        .width(88.0)
                        .show_ui(ui, |ui| {
                            for p in ["todas", "windows", "android", "linux", "macos", "java"] {
                                ui.selectable_value(&mut self.platform_filter, p.to_string(), p);
                            }
                        });

                    ui.label(RichText::new(tr(self.lang, "Ordenar:")).size(12.0).color(Color32::GRAY));
                    egui::ComboBox::from_id_salt("sort_combo")
                        .selected_text(match self.sort_by {
                            SortBy::Stars => tr(self.lang, "Mais Populares"),
                            SortBy::Trust => tr(self.lang, "Maior Confiança"),
                            SortBy::Name => tr(self.lang, "Nome (A-Z)"),
                            SortBy::Updated => tr(self.lang, "Mais Recentes"),
                        })
                        .width(120.0)
                        .show_ui(ui, |ui| {
                            ui.selectable_value(&mut self.sort_by, SortBy::Stars, tr(self.lang, "Mais Populares"));
                            ui.selectable_value(&mut self.sort_by, SortBy::Trust, tr(self.lang, "Maior Confiança"));
                            ui.selectable_value(&mut self.sort_by, SortBy::Updated, tr(self.lang, "Mais Recentes"));
                            ui.selectable_value(&mut self.sort_by, SortBy::Name, tr(self.lang, "Nome (A-Z)"));
                        });

                    ui.separator();
                    ui.selectable_value(&mut self.grid_view, true, tr(self.lang, "⊞ Grade"));
                    ui.selectable_value(&mut self.grid_view, false, tr(self.lang, "☰ Lista"));

                    ui.separator();
                    ui.add(
                        egui::TextEdit::singleline(&mut self.add_repo_input)
                            .hint_text(tr(self.lang, "Adicionar dono/repo do GitHub"))
                            .desired_width(185.0),
                    );
                    if ui
                        .add_enabled(
                            self.busy_task.is_none() && !self.add_repo_input.trim().is_empty(),
                            egui::Button::new(tr(self.lang, "+ Verificar Repo")),
                        )
                        .clicked()
                    {
                        let target = self.add_repo_input.trim().to_string();
                        self.add_repo_input.clear();
                        self.spawn_fetch_or_verify(target, false);
                    }

                    ui.separator();
                    ui.label("🎨");
                    egui::ComboBox::from_id_salt("quick_theme_selector")
                        .selected_text(tr(self.lang, self.theme.label()))
                        .width(140.0)
                        .show_ui(ui, |ui| {
                            for theme in crate::theme::AppTheme::ALL {
                                if ui.selectable_label(self.theme == *theme, tr(self.lang, theme.label())).clicked() {
                                    self.set_theme(ctx, *theme);
                                }
                            }
                        });

                    ui.separator();
                    ui.label("⚡");
                    egui::ComboBox::from_id_salt("quick_performance_selector")
                        .selected_text(tr(self.lang, self.performance_mode.short_label()))
                        .width(115.0)
                        .show_ui(ui, |ui| {
                            for mode in crate::performance::PerformanceMode::ALL {
                                if ui.selectable_label(self.performance_mode == *mode, tr(self.lang, mode.label())).clicked() {
                                    self.set_performance_mode(*mode);
                                }
                            }
                        });
                });

            });

        // ─── RODAPÉ: Barra de Status e Progresso de Download ───────────────
        egui::TopBottomPanel::bottom("bottom_status")
            .frame(
                egui::Frame::NONE
                    .fill(p.bg_status)
                    .stroke(Stroke::new(1.0_f32, p.border_subtle))
                    .inner_margin(egui::Margin::symmetric(16, 7)),
            )
            .show(ctx, |ui| {
                ui.horizontal(|ui| {
                    let jobs = self.downloads.snapshot();
                    let queued = jobs.iter().filter(|j| j.state == JobState::Queued).count();
                    let running: Vec<_> = jobs
                        .iter()
                        .filter_map(|j| match j.state {
                            JobState::Downloading { downloaded_bytes, total_bytes } => {
                                Some((j, downloaded_bytes, total_bytes))
                            }
                            _ => None,
                        })
                        .collect();
                    if !running.is_empty() || queued > 0 {
                        for (job, downloaded_bytes, total_bytes) in &running {
                            let frac = if *total_bytes > 0 {
                                (*downloaded_bytes as f32 / *total_bytes as f32).clamp(0.0, 1.0)
                            } else {
                                0.5
                            };
                            let mb_done = *downloaded_bytes as f64 / 1_048_576.0;
                            let mb_tot = *total_bytes as f64 / 1_048_576.0;
                            ui.label(
                                RichText::new(format!("⬇ {} {}:", friendly_title(&job.repo), job.tag))
                                    .color(Color32::from_rgb(100, 200, 255))
                                    .strong(),
                            )
                            .on_hover_text(&job.asset_name);
                            ui.add(
                                egui::ProgressBar::new(frac)
                                    .desired_width(170.0)
                                    .text(format!("{mb_done:.1}/{mb_tot:.1} MB ({:.0}%)", frac * 100.0)),
                            );
                        }
                        if queued > 0 {
                            ui.label(
                                RichText::new(format!("+{queued} {}", tr(self.lang, "na fila")))
                                    .color(Color32::from_rgb(175, 185, 200)),
                            );
                        }
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
                            RichText::new(format!("{} {}", self.apps.len(), tr(self.lang, "projetos verificados no catálogo")))
                                .size(11.5)
                                .color(Color32::from_rgb(115, 125, 140)),
                        );
                    });
                });
            });

        // ─── PAINEL DIREITO: Ficha da Loja, Precedentes e Download ─────────
        if !matches!(
            self.section,
            NavSection::CppToRust
                | NavSection::LearnRust
                | NavSection::RustLib
                | NavSection::TerminalStudio
                | NavSection::Optimizer
                | NavSection::Settings
                | NavSection::Profile
        ) {
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
                            .fill(p.bg_card)
                            .stroke(Stroke::new(1.0_f32, p.border_card))
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
                    .fill(p.bg_app)
                    .inner_margin(egui::Margin::same(16)),
            )
            .show(ctx, |ui| {
                if self.section == NavSection::CppToRust {
                    self.render_translator_studio(ui);
                    return;
                }
                if self.section == NavSection::LearnRust {
                    self.render_learn_rust(ui);
                    return;
                }
                if self.section == NavSection::RustLib {
                    self.render_rust_lib_browser(ui);
                    return;
                }
                if self.section == NavSection::TerminalStudio {
                    self.render_terminal_studio(ui);
                    return;
                }
                if self.section == NavSection::Optimizer {
                    self.render_optimizer(ui);
                    return;
                }
                egui::ScrollArea::vertical().show(ui, |ui| {
                    match self.section {
                        NavSection::Settings => return self.render_settings_section(ui),
                        NavSection::Profile => return self.render_profile_section(ui),
                        NavSection::Community => return self.render_community_section(ui),
                        _ => {}
                    }
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
        self.render_toasts(ctx);
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
                    // Selos quebram linha em janelas estreitas (antes se sobrepunham aos botões de navegação)
                    ui.horizontal_wrapped(|ui| {
                        draw_pill(ui, &tr(self.lang, "★ DESTAQUE"), accent);
                        let (k_label, k_col) = kind_badge(hero.kind.as_deref());
                        draw_pill(ui, &tr(self.lang, k_label), k_col);
                        if let Some(cat) = &hero.category {
                            draw_pill(ui, cat, Color32::from_rgb(48, 58, 78));
                        }
                        if hero.is_rust() {
                            draw_pill(ui, &tr(self.lang, "🦀 Rust"), RUST_ORANGE);
                        }
                        let (t_label, t_col) = trust_badge(hero.trust_score, self.lang);
                        draw_pill(ui, &t_label, t_col);
                    });

                    ui.add_space(8.0);
                    ui.label(
                        RichText::new(friendly_title(&hero.full_name))
                            .size(25.0)
                            .strong()
                            .color(Color32::WHITE),
                    );
                    ui.label(
                        RichText::new(self.summary_of(hero))
                            .size(14.5)
                            .color(Color32::from_rgb(205, 215, 232)),
                    );

                    ui.add_space(6.0);
                    ui.label(
                        RichText::new(format!(
                            "★ {} {}   •   {} {}   •   {} {}",
                            format_stars(hero.stars),
                            tr(self.lang, "estrelas"),
                            tr(self.lang, "Versão"),
                            hero.latest_tag,
                            tr(self.lang, "Plataformas:"),
                            short_platforms(&hero.platforms)
                        ))
                        .size(12.5)
                        .color(Color32::from_rgb(165, 175, 195)),
                    );

                    ui.add_space(10.0);
                    ui.horizontal_wrapped(|ui| {
                        if ui
                            .add(
                                egui::Button::new(
                                    RichText::new(tr(self.lang, "📦 Ver na Loja & Baixar com SHA-256"))
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
                        if ui.button(tr(self.lang, "◀ Anterior")).clicked() {
                            self.featured_idx = (idx + featured_apps.len() - 1) % featured_apps.len();
                        }
                        ui.label(
                            RichText::new(format!("{}/{}", idx + 1, featured_apps.len()))
                                .size(12.0)
                                .color(Color32::GRAY),
                        );
                        if ui.button(tr(self.lang, "Próximo ▶")).clicked() {
                            self.featured_idx = (idx + 1) % featured_apps.len();
                        }
                    });
                });

            ui.add_space(18.0);
        }

        let only_trusted = self.only_trusted;
        let trusted = move |a: &&App| !only_trusted || a.trust_score >= 70;

        // FEITO EM RUST
        let rust_apps: Vec<App> = self.apps.iter().filter(|a| a.is_rust()).filter(trusted).cloned().collect();
        self.render_showcase_section(
            ui,
            "🦀 Feito em Rust",
            "Apps e jogos escritos em Rust, a linguagem que dá nome à loja.",
            &rust_apps,
            Some(NavSection::MadeInRust),
        );

        // LANÇADOS (OU ATUALIZADOS) RECENTEMENTE
        let (recent_title, recent_sub, recent) = recent_apps(&self.apps, self.only_trusted, 8);
        self.render_showcase_section(ui, recent_title, recent_sub, &recent, None);

        // COLEÇÕES CURADAS
        for (title, subtitle, repos) in COLLECTIONS {
            let items: Vec<App> = repos
                .iter()
                .filter_map(|r| self.apps.iter().find(|a| a.full_name.eq_ignore_ascii_case(r)))
                .filter(trusted)
                .cloned()
                .collect();
            self.render_showcase_section(ui, title, subtitle, &items, None);
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
                RichText::new(tr(self.lang, title))
                    .size(17.5)
                    .strong()
                    .color(Color32::from_rgb(235, 240, 252)),
            );
            if let Some(nav) = target_nav {
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if ui.link(tr(self.lang, "Ver todos →")).clicked() {
                        self.section = nav;
                    }
                });
            }
        });
        ui.label(
            RichText::new(tr(self.lang, subtitle))
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
            NavSection::Profile | NavSection::Community | NavSection::Settings => ("", ""),
            NavSection::Terminal => (
                "⌨ Terminal (Rust)",
                "Ferramentas de linha de comando já compiladas, baixadas da release oficial e conferidas pelo SHA-256.",
            ),
            NavSection::MadeInRust => (
                "🦀 Feito em Rust",
                "Apps e jogos cuja linguagem principal no GitHub é Rust (segurança de memória sem coletor de lixo).",
            ),
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
            NavSection::LearnRust => ("📖 Aprender & Programar em Rust", ""),
            NavSection::RustLib => ("📚  Biblioteca Rust (std & Crates)", ""),
            NavSection::TerminalStudio => ("💻  Terminal & IDEs (Espelho)", ""),
            NavSection::Optimizer => ("⚡  Otimizador do PC", ""),
        };

        ui.label(RichText::new(tr(self.lang, title)).size(20.0).strong().color(Color32::WHITE));
        ui.label(
            RichText::new(tr(self.lang, subtitle))
                .size(12.5)
                .color(Color32::from_rgb(145, 155, 175)),
        );

        // Subfiltros exclusivos para a aba de Engenharia Reversa
        if self.section == NavSection::ReverseEngineering {
            ui.add_space(8.0);
            ui.horizontal(|ui| {
                ui.label(RichText::new(tr(self.lang, "Mostrar:")).size(12.5).color(Color32::GRAY));
                ui.selectable_value(&mut self.re_subfilter, "todos".into(), tr(self.lang, "Todos de Eng. Reversa"));
                ui.selectable_value(
                    &mut self.re_subfilter,
                    "re-game".into(),
                    tr(self.lang, "🧬 Jogos (Ports, Decompilações & Recompilações)"),
                );
                ui.selectable_value(
                    &mut self.re_subfilter,
                    "re-app".into(),
                    tr(self.lang, "🔬 Aplicativos & Ferramentas (Ghidra, ImHex, x64dbg)"),
                );
            });
        }

        if self.section == NavSection::Library {
            let outdated_count = self.apps.iter().filter(|a| a.has_update()).count();
            if outdated_count > 0 {
                ui.add_space(8.0);
                let btn = egui::Button::new(
                    RichText::new(format!("{} ({outdated_count})", tr(self.lang, "⬆ Atualizar Todos os Aplicativos")))
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

        if !matches!(self.section, NavSection::ReverseEngineering | NavSection::Library | NavSection::Terminal | NavSection::Optimizer) {
            ui.add_space(8.0);
            ui.horizontal_wrapped(|ui| {
                ui.label(RichText::new(tr(self.lang, "Filtro:")).size(12.0).color(Color32::GRAY));
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
                    if ui.selectable_label(is_active, tr(self.lang, label)).clicked() {
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
        if self.section == NavSection::Terminal {
            self.render_crate_installer(ui);
        }

        let filtered = self.filtered_apps();
        let (reinstall, outside_catalog) = if self.section == NavSection::Library {
            self.wishlist_split()
        } else {
            (Vec::new(), Vec::new())
        };
        if filtered.is_empty() && reinstall.is_empty() && outside_catalog.is_empty() {
            ui.vertical_centered(|ui| {
                ui.add_space(30.0);
                ui.label(
                    RichText::new(self.tr("Nenhum item encontrado nesta seção com os filtros atuais."))
                        .size(15.5)
                        .color(Color32::GRAY),
                );
            });
        }

        self.render_cards_collection(ui, &filtered);
        match self.section {
            NavSection::Library => {
                self.render_wishlist_part(ui, &reinstall, &outside_catalog);
                self.render_windows_section(ui);
                self.render_external_section(ui);
            }
            NavSection::Discover => self.render_winget_section(ui),
            _ => {}
        }
    }

    fn render_wishlist_part(&mut self, ui: &mut egui::Ui, reinstall: &[App], outside_catalog: &[String]) {
        if reinstall.is_empty() && outside_catalog.is_empty() {
            return;
        }
        ui.add_space(14.0);
        ui.label(
            RichText::new(format!(
                "{} ({})",
                self.tr("🔁 Para reinstalar"),
                reinstall.len() + outside_catalog.len()
            ))
            .size(17.0)
            .strong()
            .color(Color32::WHITE),
        );
        ui.label(
            RichText::new(self.tr(
                "Vindos de uma biblioteca importada. Abra cada um e baixe; o tempo de jogo volta junto.",
            ))
            .size(12.0)
            .color(Color32::from_rgb(145, 155, 175)),
        );
        ui.add_space(8.0);
        self.render_cards_collection(ui, reinstall);
        if !outside_catalog.is_empty() {
            ui.label(
                RichText::new(format!(
                    "{} {}",
                    self.tr("Fora do catálogo local (adicione pelo campo \"Adicionar dono/repo\"):"),
                    outside_catalog.join(", ")
                ))
                .size(12.0)
                .color(Color32::from_rgb(240, 195, 85)),
            );
        }
    }

    /// Lista "Para reinstalar": apps do catálogo ainda não instalados e nomes que não estão no catálogo.
    fn wishlist_split(&self) -> (Vec<App>, Vec<String>) {
        let mut in_catalog = Vec::new();
        let mut outside = Vec::new();
        for name in &self.wishlist {
            match self.apps.iter().find(|a| a.full_name.eq_ignore_ascii_case(name)) {
                Some(a) if a.installed.is_none() => in_catalog.push(a.clone()),
                Some(_) => {}
                None => outside.push(name.clone()),
            }
        }
        (in_catalog, outside)
    }

    fn in_wishlist(&self, full_name: &str) -> bool {
        self.wishlist.iter().any(|w| w.eq_ignore_ascii_case(full_name))
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
                    // `vertical`: dentro da linha horizontal, o topo e o corpo do card eram colocados
                    // lado a lado (título cortado e espaço vazio no card).
                    ui.vertical(|ui| self.render_grid_card(ui, app, card_w));
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
                            if let Some(avatar_url) = avatar_url(&app.full_name, 64) {
                                ui.add(
                                    egui::Image::new(&avatar_url)
                                        .fit_to_exact_size(Vec2::new(22.0, 22.0))
                                        .corner_radius(CornerRadius::same(5)),
                                );
                            }

                            draw_pill(ui, &tr(self.lang, kind_label), kind_color);
                            if app.is_rust() {
                                draw_pill(ui, "🦀", RUST_ORANGE);
                            }

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

                        ui.label(
                            RichText::new(format!(
                                "{}  •  {}",
                                owner_label(&app.full_name),
                                app.category.as_deref().unwrap_or("Geral")
                            ))
                            .size(11.0)
                            .color(Color32::from_rgb(135, 150, 175)),
                        );

                        ui.add_space(4.0);
                        let sum = truncate_chars(&self.summary_of(app), 88);
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
                                            RichText::new(tr(self.lang, btn_txt)).size(11.5).strong().color(Color32::WHITE),
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

        if clickable_card(resp, &friendly_title(&app.full_name)).clicked() {
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
                    if let Some(avatar_url) = avatar_url(&app.full_name, 64) {
                        ui.add(
                            egui::Image::new(&avatar_url)
                                .fit_to_exact_size(Vec2::new(26.0, 26.0))
                                .corner_radius(CornerRadius::same(5)),
                        );
                    }

                    let (kind_label, kind_color) = kind_badge(app.kind.as_deref());
                    draw_pill(ui, &tr(self.lang, kind_label), kind_color);

                    if let Some(cat) = &app.category {
                        if !cat.is_empty() {
                            draw_pill(ui, cat, Color32::from_rgb(52, 62, 82));
                        }
                    }
                    if app.is_rust() {
                        draw_pill(ui, &tr(self.lang, "🦀 Rust"), RUST_ORANGE);
                    }
                    if app.attestation == github::ATTESTATION_PRESENT {
                        draw_pill(ui, &tr(self.lang, "🏗 Atestado"), Color32::from_rgb(40, 100, 90));
                    }

                    ui.label(
                        RichText::new(friendly_title(&app.full_name))
                            .size(15.5)
                            .strong()
                            .color(Color32::from_rgb(235, 240, 250)),
                    );

                    if app.has_update() {
                        draw_pill(ui, &tr(self.lang, "⬆ ATUALIZAÇÃO"), Color32::from_rgb(200, 120, 20));
                    } else if let Some(inst) = &app.installed {
                        draw_pill(
                            ui,
                            &format!("{} ({})", tr(self.lang, "✔ Instalado"), inst.installed_tag),
                            Color32::from_rgb(30, 125, 75),
                        );
                    }

                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        let (t_label, t_col) = trust_badge(app.trust_score, self.lang);
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
                    RichText::new(self.summary_of(app))
                        .size(13.0)
                        .color(Color32::from_rgb(190, 198, 212)),
                );
            })
            .response;

        if clickable_card(resp, &friendly_title(&app.full_name)).clicked() {
            self.select_app(&app.full_name);
        }
    }

    fn render_details_panel(&mut self, ui: &mut egui::Ui, app: &App) {
        ui.horizontal(|ui| {
            let (kind_label, kind_color) = kind_badge(app.kind.as_deref());
            draw_pill(ui, &tr(self.lang, kind_label), kind_color);
            if let Some(cat) = &app.category {
                draw_pill(ui, cat, Color32::from_rgb(52, 62, 82));
            }
        });

        ui.add_space(4.0);
        ui.horizontal(|ui| {
            if let Some(avatar_url) = avatar_url(&app.full_name, 128) {
                ui.add(
                    egui::Image::new(&avatar_url)
                        .fit_to_exact_size(Vec2::new(42.0, 42.0))
                        .corner_radius(CornerRadius::same(8)),
                );
            }
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
                RichText::new(format!("★ {} {}", app.stars, tr(self.lang, "estrelas")))
                    .strong()
                    .color(Color32::from_rgb(245, 200, 75)),
            );
            ui.separator();
            ui.label(format!(
                "{} {}",
                tr(self.lang, "Licença:"),
                if app.license.is_empty() || app.license == "NOASSERTION" {
                    tr(self.lang, "não informada")
                } else {
                    app.license.clone()
                }
            ));
            ui.separator();
            if ui.link(tr(self.lang, "GitHub Oficial ↗")).clicked() {
                if let Err(e) = downloader::open_url(&app.url) {
                    self.status_msg = format!("{e:#}");
                    self.status_is_error = true;
                }
            }
        });

        self.render_share_buttons(ui, app);

        // Selos: linguagem e atestado de build do GitHub
        ui.add_space(4.0);
        ui.horizontal_wrapped(|ui| {
            if app.is_rust() {
                draw_pill(ui, &tr(self.lang, "🦀 Feito em Rust"), RUST_ORANGE);
            } else if !app.language.is_empty() {
                draw_pill(ui, &format!("{} {}", tr(self.lang, "Linguagem:"), app.language), Color32::from_rgb(52, 62, 82));
            }
            let attestation_hint = "O GitHub tem um atestado de build (artifact attestation) para o instalador recomendado: \
                ele foi gerado pelo GitHub Actions a partir do código do repositório. A RRS só confere se o atestado existe; \
                a assinatura pode ser conferida com `gh release verify-asset`.";
            match app.attestation.as_str() {
                github::ATTESTATION_PRESENT => {
                    draw_pill(ui, &tr(self.lang, "🏗 Atestado de build: presente"), Color32::from_rgb(40, 100, 90));
                    ui.label(RichText::new("ⓘ").color(Color32::GRAY)).on_hover_text(attestation_hint);
                }
                github::ATTESTATION_ABSENT => {
                    draw_pill(ui, &tr(self.lang, "🏗 Sem atestado de build"), Color32::from_rgb(60, 66, 80));
                    ui.label(RichText::new("ⓘ").color(Color32::GRAY)).on_hover_text(
                        "O instalador recomendado não tem atestado de build do GitHub. Isso é comum e não indica problema; \
                         só significa que a origem do arquivo não pode ser rastreada até o código por esse meio.",
                    );
                }
                _ => {
                    ui.label(
                        RichText::new(tr(self.lang, "🏗 Atestado de build: não consultado (use \"Revalidar no GitHub\")"))
                            .size(11.0)
                            .color(Color32::GRAY),
                    );
                }
            }
        });

        ui.add_space(8.0);
        ui.label(
            RichText::new(self.summary_of(app))
                .size(13.8)
                .color(Color32::from_rgb(215, 222, 235)),
        );

        if self.in_wishlist(&app.full_name) && app.installed.is_none() {
            ui.add_space(8.0);
            let mut remove = false;
            egui::Frame::NONE
                .fill(Color32::from_rgb(24, 40, 52))
                .stroke(Stroke::new(1.0_f32, Color32::from_rgb(60, 130, 170)))
                .corner_radius(CornerRadius::same(6))
                .inner_margin(egui::Margin::same(10))
                .show(ui, |ui| {
                    ui.label(
                        RichText::new(tr(self.lang, "🔁 Na lista para reinstalar (biblioteca importada)"))
                            .size(12.5)
                            .strong()
                            .color(Color32::from_rgb(170, 215, 245)),
                    );
                    ui.label(
                        RichText::new(tr(self.lang, "Ao baixar, o tempo de jogo do outro PC volta junto."))
                            .size(11.5)
                            .color(Color32::from_rgb(170, 185, 205)),
                    );
                    if ui.small_button(tr(self.lang, "Tirar da lista")).clicked() {
                        remove = true;
                    }
                });
            if remove {
                if let Ok(conn) = db::open(&self.db_path) {
                    let _ = db::remove_from_wishlist(&conn, &app.full_name);
                }
                self.reload_from_db();
            }
        }

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
                        RichText::new(tr(self.lang, "🧬 Port / Recompilação por Engenharia Reversa"))
                            .size(12.5)
                            .strong()
                            .color(Color32::from_rgb(210, 175, 255)),
                    );
                    ui.label(
                        RichText::new(tr(self.lang, "Este projeto recria o motor do jogo para PC a partir de engenharia reversa e não inclui os arquivos do jogo. Para jogos originalmente comerciais, o programa pede a sua cópia/ROM original na primeira abertura."))
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
                    RichText::new(format!("{} {}", tr(self.lang, "📦 Obter Aplicativo — Versão"), app.latest_tag))
                        .size(14.5)
                        .strong(),
                );
                ui.add_space(6.0);

                if app.assets.is_empty() {
                    ui.label(tr(self.lang, "Nenhum instalador encontrado nesta Release."));
                    return;
                }

                let recommended_idx = github::pick_best_asset(&app.assets).unwrap_or(0);
                let idx = self.selected_asset_idx.min(app.assets.len() - 1);
                let chosen = &app.assets[idx];

                ui.label(RichText::new(tr(self.lang, "Instalador selecionado:")).size(11.5).color(Color32::GRAY));
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
                                    tr(self.lang, "★ Recomendado p/ seu PC: ")
                                } else {
                                    String::new()
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
                        RichText::new(format!("{}\n{digest}", tr(self.lang, "✔ SHA-256 Oficial do GitHub:")))
                            .size(10.8)
                            .monospace()
                            .color(Color32::from_rgb(105, 215, 145)),
                    );
                } else {
                    ui.label(
                        RichText::new(tr(self.lang, "• O hash SHA-256 será calculado e registrado durante o download."))
                            .size(11.2)
                            .color(Color32::from_rgb(220, 185, 95)),
                    );
                }

                ui.add_space(8.0);

                let is_downloading = self.downloads.is_queued(&app.full_name, &app.latest_tag);

                let dl_btn_label = if is_downloading {
                    tr(self.lang, "⏳ Na fila de downloads...")
                } else if app.has_update() {
                    format!("{} {} ({})", tr(self.lang, "⬆ Atualizar para"), app.latest_tag, tr(self.lang, "Verificar SHA-256"))
                } else {
                    tr(self.lang, "⬇ Baixar & Verificar SHA-256 Automaticamente")
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
                    self.start_download(app.full_name.clone(), app.latest_tag.clone(), chosen.clone());
                }

                if let Some(inst) = &app.installed {
                    ui.add_space(10.0);
                    ui.separator();
                    ui.add_space(6.0);

                    let status_text = if app.has_update() {
                        format!(
                            "{} {} ({} {})",
                            tr(self.lang, "Instalado:"),
                            inst.installed_tag,
                            tr(self.lang, "Nova versão disponível:"),
                            app.latest_tag
                        )
                    } else {
                        format!("{} {}", tr(self.lang, "✔ Pronto no PC — Versão"), inst.installed_tag)
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
                                tr(self.lang, "confirmado c/ GitHub")
                            } else {
                                tr(self.lang, "calculado")
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
                                "{} {}",
                                tr(self.lang, "Baixado em:"),
                                &inst.installed_at[..inst.installed_at.len().min(10)]
                            ))
                            .size(10.2)
                            .color(Color32::GRAY),
                        );
                    }

                    ui.add_space(4.0);
                    ui.horizontal(|ui| {
                        ui.label(
                            RichText::new(format!("{} {}", tr(self.lang, "⏱ Tempo de Jogo:"), inst.formatted_playtime()))
                                .size(11.5)
                                .color(Color32::from_rgb(175, 215, 255)),
                        );
                        ui.add_space(6.0);
                        ui.label(
                            RichText::new(format!("{} {}", tr(self.lang, "📅 Última vez:"), inst.formatted_last_played()))
                                .size(11.5)
                                .color(Color32::from_rgb(150, 165, 185)),
                        );
                    });

                    ui.add_space(8.0);
                    let is_game = matches!(app.kind.as_deref(), Some("game" | "re-game"));
                    let play_label = if is_game { tr(self.lang, "▶ JOGAR AGORA") } else { tr(self.lang, "▶ ABRIR APLICATIVO") };
                    let play_btn = egui::Button::new(
                        RichText::new(play_label).size(13.5).strong().color(Color32::WHITE),
                    )
                    .fill(Color32::from_rgb(32, 145, 78))
                    .min_size(Vec2::new(175.0, 34.0))
                    .corner_radius(CornerRadius::same(6));

                    if ui.add(play_btn).clicked() {
                        self.launch_installed(app);
                    }

                    ui.add_space(6.0);
                    ui.horizontal(|ui| {
                        if ui.button(tr(self.lang, "📂 Abrir Pasta")).clicked() {
                            let _ = downloader::reveal_in_folder(&self.installed_path(&inst.file_path));
                        }

                        if ui
                            .add_enabled(
                                self.busy_task.is_none(),
                                egui::Button::new(tr(self.lang, "🛡️ Windows Defender")),
                            )
                            .on_hover_text(tr(self.lang, "Executa uma varredura nativa do Windows Defender no arquivo baixado sem custos"))
                            .clicked()
                        {
                            let path = self.installed_path(&inst.file_path);
                            let repo = app.full_name.clone();
                            let events = Arc::clone(&self.bg_events);
                            self.busy_task = Some("Escaneando com Windows Defender...".into());
                            std::thread::spawn(move || {
                                let def_res = downloader::scan_with_windows_defender(&path)
                                    .unwrap_or_else(|e| {
                                        downloader::DefenderScanResult::not_verified(&format!(
                                            "não foi possível acionar o Windows Defender ({e})."
                                        ))
                                    });
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
                                    defender_verdict: def_res.verdict,
                                    defender_msg: def_res.message,
                                    authenticode: auth_text,
                                });
                            });
                        }

                        // Dois cliques: a pasta do app em downloads/ pode ter saves dentro
                        let confirming = self.uninstall_confirm.as_deref() == Some(app.full_name.as_str());
                        let label = if confirming { self.tr("⚠ Confirmar remoção") } else { self.tr("🗑 Desinstalar") };
                        if ui
                            .button(label)
                            .on_hover_text(self.tr("Apaga a pasta do app em downloads\\ e tira da biblioteca."))
                            .clicked()
                        {
                            if !confirming {
                                self.uninstall_confirm = Some(app.full_name.clone());
                            } else {
                                self.uninstall_confirm = None;
                                let _ = downloader::uninstall_app(&self.base_dir(), &app.full_name);
                                if let Ok(conn) = db::open(&self.db_path) {
                                    let _ = db::unmark_installed(&conn, &app.full_name);
                                }
                                self.reload_from_db();
                                self.status_msg =
                                    format!("✔ {} {}", friendly_title(&app.full_name), self.tr("removido do computador."));
                                self.status_is_error = false;
                            }
                        }
                    });

                    if let Some((msg, verdict)) = &self.defender_scan_status {
                        ui.add_space(6.0);
                        let col = match verdict {
                            ScanVerdict::Clean => Color32::from_rgb(120, 230, 150),
                            ScanVerdict::Threat => Color32::from_rgb(255, 120, 120),
                            ScanVerdict::NotVerified => Color32::from_rgb(235, 190, 85),
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

                    self.render_security_extras(ui, app, inst);
                    self.render_saves_block(ui, app, inst);
                }
            });

        // ─── VERSÕES ANTERIORES ─────────────────────────────────────────────
        ui.add_space(10.0);
        let arrow = if self.versions_open { "▼" } else { "▶" };
        if ui
            .selectable_label(
                self.versions_open,
                RichText::new(format!("{arrow} 🕘 Versões anteriores")).size(13.5).strong(),
            )
            .clicked()
        {
            self.versions_open = !self.versions_open;
        }
        if self.versions_open {
            ui.indent("versoes_anteriores", |ui| {
                let Some(releases) = self.releases_cache.get(&app.full_name).cloned() else {
                    ui.label(
                        RichText::new(tr(self.lang, "Instale uma versão anterior se a mais nova der problema no seu PC."))
                            .size(11.5)
                            .color(Color32::GRAY),
                    );
                    if ui
                        .add_enabled(self.busy_task.is_none(), egui::Button::new(tr(self.lang, "Carregar versões do GitHub")))
                        .clicked()
                    {
                        self.spawn_load_releases(app.full_name.clone());
                    }
                    return;
                };
                if releases.is_empty() {
                    ui.label(tr(self.lang, "Nenhuma release publicada."));
                    return;
                }
                let installed_tag = app.installed.as_ref().map(|i| i.installed_tag.clone());
                for rel in &releases {
                    let assets = rel.installable_assets();
                    let best = github::pick_best_asset(&assets).map(|i| assets[i].clone());
                    ui.horizontal_wrapped(|ui| {
                        ui.label(RichText::new(&rel.tag_name).strong().monospace());
                        if let Some(date) = rel.published_at.as_deref() {
                            ui.label(
                                RichText::new(&date[..date.len().min(10)])
                                    .size(11.0)
                                    .color(Color32::GRAY),
                            );
                        }
                        if rel.prerelease {
                            draw_pill(ui, &tr(self.lang, "pré-lançamento"), Color32::from_rgb(150, 100, 20));
                        }
                        if rel.tag_name == app.latest_tag {
                            draw_pill(ui, &tr(self.lang, "mais recente"), Color32::from_rgb(35, 95, 160));
                        }
                        if installed_tag.as_deref() == Some(rel.tag_name.as_str()) {
                            draw_pill(ui, &tr(self.lang, "instalada"), Color32::from_rgb(30, 125, 75));
                        }
                        match &best {
                            Some(asset) => {
                                let hint = format!(
                                    "{} ({:.1} MB), {}",
                                    asset.name,
                                    asset.size as f64 / 1_048_576.0,
                                    if asset.digest.is_some() {
                                        tr(self.lang, "SHA-256 conferido com o GitHub")
                                    } else {
                                        tr(self.lang, "release sem SHA-256 do GitHub (só calculado)")
                                    }
                                );
                                if ui
                                    .add_enabled(
                                        !self.downloads.is_queued(&app.full_name, &rel.tag_name),
                                        egui::Button::new(tr(self.lang, "⬇ Baixar")),
                                    )
                                    .on_hover_text(hint)
                                    .clicked()
                                    && self.start_download(app.full_name.clone(), rel.tag_name.clone(), asset.clone())
                                    && rel.tag_name != app.latest_tag
                                {
                                    self.record_stat(crate::achievements::STAT_OLD_VERSION);
                                }
                            }
                            None => {
                                ui.label(RichText::new(tr(self.lang, "sem instalador")).size(11.0).color(Color32::GRAY));
                            }
                        }
                    });
                }
            });
        }

        ui.add_space(12.0);

        // ─── BLOCO 2: VERIFICAÇÃO DE SEGURANÇA E PRECEDENTES ───────────────
        let (t_label, t_col) = trust_badge(app.trust_score, self.lang);
        egui::Frame::NONE
            .fill(Color32::from_rgb(16, 19, 26))
            .stroke(Stroke::new(1.0_f32, t_col))
            .corner_radius(CornerRadius::same(8))
            .inner_margin(egui::Margin::same(12))
            .show(ui, |ui| {
                ui.horizontal(|ui| {
                    ui.label(RichText::new(tr(self.lang, "🛡 Auditoria de Precedentes")).size(14.5).strong());
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        draw_pill(ui, &t_label, t_col);
                    });
                });

                ui.add_space(6.0);
                ui.add(
                    egui::ProgressBar::new(app.trust_score as f32 / 100.0)
                        .fill(t_col)
                        .text(format!("{} {}/100", tr(self.lang, "Índice de Confiança:"), app.trust_score)),
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
                            egui::Button::new(tr(self.lang, "🔄 Revalidar no GitHub")),
                        )
                        .clicked()
                    {
                        self.spawn_fetch_or_verify(app.full_name.clone(), false);
                    }

                    if ui
                        .add_enabled(
                            self.busy_task.is_none(),
                            egui::Button::new(tr(self.lang, "🔍 Checar no VirusTotal")),
                        )
                        .on_hover_text(tr(self.lang, "Consulta os hashes oficiais da Release no VirusTotal sem baixar o arquivo"))
                        .clicked()
                    {
                        self.record_stat(crate::achievements::STAT_VT);
                        self.spawn_fetch_or_verify(app.full_name.clone(), true);
                    }
                });
            });
    }

    /// Estúdio interativo de Tradução de Código C e C++ para Rust dentro do RRS - Riva Rust Store.
    fn render_translator_studio(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            ui.label(
                RichText::new(tr(self.lang, "🦀 Tradutor de C / C++ para Rust — RRS Studio"))
                    .size(21.0)
                    .strong()
                    .color(Color32::from_rgb(245, 140, 75)),
            );
            draw_pill(ui, &tr(self.lang, "OFFLINE & INSTANTÂNEO"), Color32::from_rgb(30, 120, 75));
            draw_pill(ui, &tr(self.lang, "VALIDADOR RUSTC INTEGRADO"), Color32::from_rgb(45, 95, 175));
        });
        ui.label(
            RichText::new(
                tr(self.lang, "Converta estruturas, classes, headers de engenharia reversa, ponteiros, loops e containers STL (std::vector, std::string) de C/C++ para Rust."),
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
                ui.horizontal_wrapped(|ui| {
                    ui.label(RichText::new(tr(self.lang, "Modo de Conversão:")).strong().size(12.5));
                    if ui
                        .selectable_value(
                            &mut self.translate_mode,
                            TranslateMode::SafeRust,
                            tr(self.lang, "🦀 Rust Idiomático & Seguro (Vec, String, &mut)"),
                        )
                        .clicked()
                    {
                        trigger_retranslate = true;
                    }
                    if ui
                        .selectable_value(
                            &mut self.translate_mode,
                            TranslateMode::FfiBindings,
                            tr(self.lang, "🔗 FFI / ABI C (#[repr(C)], *mut T, extern \"C\")"),
                        )
                        .clicked()
                    {
                        trigger_retranslate = true;
                    }

                    ui.separator();

                    ui.label(RichText::new(tr(self.lang, "Exemplos:")).size(12.0).color(Color32::GRAY));
                    for (idx, ex) in translator::EXAMPLES.iter().enumerate() {
                        if ui
                            .button(format!("{} {}", tr(self.lang, "Exemplo"), idx + 1))
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

                // Linha que quebra em janelas estreitas (antes "Traduzir" ficava por cima de "Gerar Projeto Cargo")
                ui.horizontal_wrapped(|ui| {
                    if ui
                        .add(
                            egui::Button::new(RichText::new(tr(self.lang, "⚡ Traduzir")).strong())
                                .fill(Color32::from_rgb(200, 95, 40)),
                        )
                        .clicked()
                    {
                        trigger_retranslate = true;
                        self.record_stat(crate::achievements::STAT_TRANSLATIONS);
                    }

                    if ui
                        .add(
                            egui::Button::new(
                                RichText::new(tr(self.lang, "✔ Validar Sintaxe no rustc"))
                                    .strong()
                                    .color(Color32::WHITE),
                            )
                            .fill(Color32::from_rgb(30, 130, 75)),
                        )
                        .on_hover_text(tr(self.lang, "Compila o código Rust gerado usando o rustc local para garantir que não há erros"))
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
                            egui::Button::new(RichText::new(tr(self.lang, "📋 Copiar Rust")).strong())
                                .fill(Color32::from_rgb(45, 95, 165)),
                        )
                        .clicked()
                    {
                        ui.ctx().copy_text(self.rust_output.clone());
                        self.status_msg = "Código Rust copiado para a área de transferência!".into();
                        self.status_is_error = false;
                    }
                });

                ui.add_space(8.0);

                ui.horizontal_wrapped(|ui| {
                    ui.label(RichText::new(tr(self.lang, "Arquivo (.c/.cpp/.h/.rs):")).size(12.0).color(Color32::GRAY));
                    ui.add(
                        egui::TextEdit::singleline(&mut self.file_io_path)
                            .hint_text(tr(self.lang, "Caminho de arquivo para carregar (.cpp/.h) ou salvar (.rs)"))
                            .desired_width(290.0),
                    );
                    if ui.button(tr(self.lang, "📂 Carregar C/C++")).clicked() && !self.file_io_path.trim().is_empty() {
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
                    if ui.button(tr(self.lang, "💾 Salvar .rs")).clicked() && !self.file_io_path.trim().is_empty() {
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
                                RichText::new(tr(self.lang, "📦 Gerar Projeto Cargo"))
                                    .strong()
                                    .color(Color32::from_rgb(255, 210, 140)),
                            )
                            .fill(Color32::from_rgb(52, 38, 20)),
                        )
                        .on_hover_text(tr(self.lang, "Cria uma pasta completa com Cargo.toml e src/main.rs pronta para 'cargo run'"))
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
                });
            });

        ui.add_space(6.0);
        self.render_ai_translate(ui);
        ui.add_space(6.0);

        // Editor Lado a Lado: C/C++ (Esquerda)  ➔  Rust (Direita)
        let editor_height = (ui.available_height() - 135.0).max(240.0);
        let half_w = ((ui.available_width() - 14.0) / 2.0).max(260.0);

        ui.horizontal(|ui| {
            // Coluna Esquerda: Entrada C / C++
            ui.vertical(|ui| {
                ui.set_width(half_w);
                ui.horizontal(|ui| {
                    draw_pill(ui, &tr(self.lang, "ENTRADA C / C++"), Color32::from_rgb(55, 85, 140));
                    ui.label(
                        RichText::new(tr(self.lang, "Cole seu código .c, .cpp, .h ou .hpp abaixo (tradução em tempo real)"))
                            .size(11.5)
                            .color(Color32::GRAY),
                    );
                });
                ui.add_space(4.0);
                // min_scrolled_height: dentro da linha horizontal a altura disponível é a de uma linha,
                // e o editor ficava com ~4 linhas visíveis.
                egui::ScrollArea::vertical()
                    .id_salt("cpp_scroll")
                    .max_height(editor_height)
                    .min_scrolled_height(editor_height)
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
                    draw_pill(ui, &tr(self.lang, "SAÍDA RUST (.RS)"), Color32::from_rgb(195, 85, 35));
                    ui.label(
                        RichText::new(tr(self.lang, "Você também pode editar o código Rust gerado antes de validar/salvar"))
                            .size(11.5)
                            .color(Color32::GRAY),
                    );
                });
                ui.add_space(4.0);
                egui::ScrollArea::vertical()
                    .id_salt("rust_scroll")
                    .max_height(editor_height)
                    .min_scrolled_height(editor_height)
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
                    RichText::new(tr(self.lang, "📋 Notas de Conversão & Verificação do Compilador"))
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
                                RichText::new(format!("{}\n{err_msg}", tr(self.lang, "Diagnóstico do rustc:")))
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

const RUST_ORANGE: Color32 = Color32::from_rgb(190, 85, 35);

/// Linha "Lançados recentemente" (data da release). Enquanto poucos apps têm essa data
/// (ela chega ao revalidar/checar atualizações), usa a data do último commit e diz isso no título.
fn recent_apps(apps: &[App], only_trusted: bool, max: usize) -> (&'static str, &'static str, Vec<App>) {
    let eligible = apps.iter().filter(|a| !only_trusted || a.trust_score >= 70);
    let mut released: Vec<&App> = eligible.clone().filter(|a| !a.released_at.is_empty()).collect();
    if released.len() >= 4 {
        released.sort_by(|a, b| b.released_at.cmp(&a.released_at));
        return (
            "🆕 Lançados recentemente",
            "Versões publicadas há menos tempo no GitHub.",
            released.into_iter().take(max).cloned().collect(),
        );
    }
    let mut pushed: Vec<&App> = eligible.filter(|a| !a.pushed_at.is_empty()).collect();
    pushed.sort_by(|a, b| b.pushed_at.cmp(&a.pushed_at));
    (
        "🆕 Atualizados recentemente",
        "Projetos com código atualizado há menos tempo (data do último commit no GitHub).",
        pushed.into_iter().take(max).cloned().collect(),
    )
}

/// Fontes de símbolos do Windows como reserva. A fonte padrão do egui não tem vários emojis e setas
/// usados na interface (🦀, 🧬, ➔...), que apareciam como "☐". Sem esses arquivos, fica a fonte padrão.
pub fn configure_fonts(ctx: &egui::Context) {
    ctx.set_fonts(font_definitions());
}

fn font_definitions() -> egui::FontDefinitions {
    let mut fonts = egui::FontDefinitions::default();
    let fonts_dir = std::env::var_os("WINDIR")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("C:\\Windows"))
        .join("Fonts");
    for (name, file) in [("segoe-symbol", "seguisym.ttf"), ("segoe-emoji", "seguiemj.ttf")] {
        if let Ok(bytes) = std::fs::read(fonts_dir.join(file)) {
            fonts
                .font_data
                .insert(name.to_owned(), Arc::new(egui::FontData::from_owned(bytes)));
            for family in [egui::FontFamily::Proportional, egui::FontFamily::Monospace] {
                fonts.families.entry(family).or_default().push(name.to_owned());
            }
        }
    }
    fonts
}

#[allow(dead_code)]
fn configure_theme(ctx: &egui::Context) {
    crate::theme::AppTheme::default().apply(ctx, crate::theme::AccentColor::default());
}

fn nav_button(ui: &mut egui::Ui, current: &mut NavSection, target: NavSection, label: &str) {
    let active = *current == target;
    let fill = if active {
        ui.visuals().selection.bg_fill
    } else {
        Color32::TRANSPARENT
    };
    let text_col = if active {
        Color32::WHITE
    } else {
        ui.visuals().widgets.noninteractive.fg_stroke.color
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
        Some("cli") => ("TERMINAL", Color32::from_rgb(70, 95, 70)),
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
        "alacritty/alacritty" => "Alacritty (Terminal)".into(),
        "wezterm/wezterm" => "WezTerm (Terminal)".into(),
        "lapce/lapce" => "Lapce (Editor de Código)".into(),
        "neovide/neovide" => "Neovide (Neovim com Interface)".into(),
        "espanso/espanso" => "Espanso (Expansor de Texto)".into(),
        "gyroflow/gyroflow" => "Gyroflow (Estabilização de Vídeo)".into(),
        "a-b-street/abstreet" => "A/B Street (Simulador de Trânsito)".into(),
        "BurntSushi/ripgrep" => "ripgrep (rg)".into(),
        "sharkdp/bat" => "bat".into(),
        "sharkdp/fd" => "fd".into(),
        "sharkdp/hyperfine" => "hyperfine".into(),
        "ajeetdsouza/zoxide" => "zoxide".into(),
        "starship/starship" => "Starship (Prompt)".into(),
        other => other.rsplit('/').next().unwrap_or(other).to_string(),
    }
}

/// Avatar do dono no GitHub; outras forjas não têm um endereço público padronizado.
fn avatar_url(full_name: &str, size: u32) -> Option<String> {
    if crate::forge::is_other_forge(full_name) {
        return None;
    }
    let owner = full_name.split('/').next()?;
    Some(format!("https://github.com/{owner}.png?size={size}"))
}

/// "dono" no GitHub; "dono (codeberg.org)" nas outras forjas.
fn owner_label(full_name: &str) -> String {
    let mut parts = full_name.split('/');
    let first = parts.next().unwrap_or(full_name);
    if crate::forge::is_other_forge(full_name) {
        format!("{} ({first})", parts.next().unwrap_or(""))
    } else {
        first.to_string()
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

fn trust_badge(score: i32, lang: Lang) -> (String, Color32) {
    if score >= 70 {
        (format!("✔ {score}/100 {}", tr(lang, "Verificado")), Color32::from_rgb(30, 135, 75))
    } else if score >= 40 {
        (format!("⚠ {score}/100 {}", tr(lang, "Atenção")), Color32::from_rgb(185, 130, 20))
    } else {
        (format!("✘ {score}/100 {}", tr(lang, "Alto Risco")), Color32::from_rgb(185, 45, 45))
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

/// Card clicável com nome acessível: o leitor de tela anuncia o título do app como botão
/// (sem isso o card era só uma área clicável sem nome).
fn clickable_card(resp: egui::Response, title: &str) -> egui::Response {
    let resp = resp.interact(egui::Sense::click());
    let label = title.to_string();
    resp.widget_info(move || egui::WidgetInfo::labeled(egui::WidgetType::Button, true, &label));
    resp
}

/// Botão só com ícone, com nome acessível e dica ao passar o mouse.
fn icon_button(ui: &mut egui::Ui, icon: &str, name: &str) -> egui::Response {
    let resp = ui.small_button(icon).on_hover_text(name);
    let label = name.to_string();
    resp.widget_info(move || egui::WidgetInfo::labeled(egui::WidgetType::Button, true, &label));
    resp
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

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::BTreeSet;

    fn embedded_names() -> Vec<String> {
        let entries: Vec<db::CatalogEntry> =
            serde_json::from_str(db::EMBEDDED_CATALOG.trim_start_matches('\u{feff}')).unwrap();
        entries.into_iter().map(|e| e.full_name).collect()
    }

    /// O leitor de tela precisa receber o nome do app no card clicável e no botão de ícone.
    #[test]
    fn cards_e_botoes_de_icone_tem_nome_para_leitor_de_tela() {
        let ctx = egui::Context::default();
        ctx.enable_accesskit();
        let out = ctx.run(egui::RawInput::default(), |ctx| {
            egui::CentralPanel::default().show(ctx, |ui| {
                let resp = egui::Frame::NONE.show(ui, |ui| ui.label("conteúdo do card")).response;
                clickable_card(resp, "Ship of Harkinian");
                icon_button(ui, "✖", "Limpar busca");
            });
        });
        let update = out.platform_output.accesskit_update.expect("AccessKit ligado");
        let has = |name: &str| {
            update
                .nodes
                .iter()
                .any(|(_, n)| n.label() == Some(name) && n.role() == egui::accesskit::Role::Button)
        };
        assert!(has("Ship of Harkinian"), "card sem nome acessível");
        assert!(has("Limpar busca"), "botão de ícone sem nome acessível");
    }

    #[test]
    fn colecoes_e_destaques_usam_apps_do_catalogo() {
        let names = embedded_names();
        let has = |r: &str| names.iter().any(|n| n.eq_ignore_ascii_case(r));
        for (title, _, repos) in COLLECTIONS {
            assert!(repos.len() >= 4, "coleção '{title}' com poucos itens");
            for r in *repos {
                assert!(has(r), "coleção '{title}': {r} não está no catálogo");
            }
        }
        for r in FEATURED_REPOS {
            assert!(has(r), "destaque {r} não está no catálogo");
        }
    }

    fn apps_with(released: &[(&str, &str)], pushed: &[(&str, &str)]) -> Vec<App> {
        let mut items = Vec::new();
        for (name, date) in released {
            items.push(format!(
                r#"{{"full_name":"{name}","description":"","url":"","stars":1,"license":"","topics":"",
                    "pushed_at":"2020-01-01T00:00:00Z","latest_tag":"v1","released_at":"{date}","platforms":"",
                    "assets":[],"trust_score":90,"trust_level":"x","trust_notes":["x"]}}"#
            ));
        }
        for (name, date) in pushed {
            items.push(format!(
                r#"{{"full_name":"{name}","description":"","url":"","stars":1,"license":"","topics":"",
                    "pushed_at":"{date}","latest_tag":"v1","platforms":"","assets":[],
                    "trust_score":90,"trust_level":"x","trust_notes":["x"]}}"#
            ));
        }
        let c = db::open(":memory:").unwrap();
        db::import_catalog_str(&c, &format!("[{}]", items.join(",")), "t").unwrap();
        db::list_all(&c).unwrap()
    }

    #[test]
    fn recentes_usa_data_da_release_quando_ha_dados_suficientes() {
        let apps = apps_with(
            &[("a/1", "2026-01-01T00:00:00Z"), ("a/2", "2026-03-01T00:00:00Z"), ("a/3", "2026-02-01T00:00:00Z"), ("a/4", "2025-12-01T00:00:00Z")],
            &[("b/novo", "2026-10-01T00:00:00Z")],
        );
        let (title, _, list) = recent_apps(&apps, true, 3);
        assert!(title.contains("Lançados"));
        let names: Vec<_> = list.iter().map(|a| a.full_name.as_str()).collect();
        assert_eq!(names, vec!["a/2", "a/3", "a/1"]);
    }

    #[test]
    fn recentes_cai_para_data_do_commit_e_avisa_no_titulo() {
        let apps = apps_with(
            &[("a/1", "2026-01-01T00:00:00Z")],
            &[("b/velho", "2024-01-01T00:00:00Z"), ("b/novo", "2026-10-01T00:00:00Z")],
        );
        let (title, sub, list) = recent_apps(&apps, true, 2);
        assert!(title.contains("Atualizados"));
        assert!(sub.contains("commit"));
        assert_eq!(list[0].full_name, "b/novo");
    }

    /// Caracteres não ASCII dentro de literais de string (o que pode aparecer na tela).
    fn string_literal_chars(src: &str) -> BTreeSet<char> {
        let chars: Vec<char> = src.chars().collect();
        let mut out = BTreeSet::new();
        let mut i = 0;
        while i < chars.len() {
            let c = chars[i];
            if c == '/' && chars.get(i + 1) == Some(&'/') {
                while i < chars.len() && chars[i] != '\n' {
                    i += 1;
                }
                continue;
            }
            // literais de caractere: 'x' e '\n'
            if c == '\'' && chars.get(i + 2) == Some(&'\'') {
                i += 3;
                continue;
            }
            if c == '\'' && chars.get(i + 1) == Some(&'\\') && chars.get(i + 3) == Some(&'\'') {
                i += 4;
                continue;
            }
            if c == '"' {
                i += 1;
                while i < chars.len() && chars[i] != '"' {
                    if chars[i] == '\\' {
                        i += 2;
                        continue;
                    }
                    if !chars[i].is_ascii() && chars[i] != '\u{feff}' {
                        out.insert(chars[i]);
                    }
                    i += 1;
                }
            }
            i += 1;
        }
        out
    }

    /// Regressão do "☐": todo caractere usado nos textos da interface precisa ter glifo nas fontes configuradas.
    #[cfg(target_os = "windows")]
    #[test]
    fn todos_os_caracteres_da_interface_tem_glifo() {
        let ctx = egui::Context::default();
        configure_fonts(&ctx);
        let _ = ctx.run(egui::RawInput::default(), |_| {});
        let mut used = BTreeSet::new();
        for src in [
            include_str!("gui.rs"),
            include_str!("installer.rs"),
            include_str!("downloader.rs"),
            include_str!("translator.rs"),
            include_str!("gui/learn.rs"),
        ] {
            used.extend(string_literal_chars(src));
        }
        assert!(used.contains(&'🦀'), "o leitor de literais não encontrou o 🦀");
        let missing: Vec<char> = ctx.fonts(|f| {
            used.iter()
                .copied()
                .filter(|c| {
                    !f.has_glyph(&egui::FontId::proportional(13.0), *c)
                        || !f.has_glyph(&egui::FontId::monospace(13.0), *c)
                })
                .collect()
        });
        assert!(missing.is_empty(), "sem glifo nas fontes: {missing:?}");
    }
}
