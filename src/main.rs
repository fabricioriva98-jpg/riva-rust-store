mod db;
mod downloader;
mod github;
mod gui;
mod installer;
mod translator;
mod verify;

use anyhow::{bail, Context, Result};
use clap::{Parser, Subcommand};
use std::path::PathBuf;
use std::time::Duration;

#[derive(Parser)]
#[command(
    name = "rrs",
    about = "RRS - Riva Rust Store: Loja verificada de Apps/Jogos do GitHub + Tradutor C/C++ para Rust"
)]
struct Cli {
    /// Caminho do banco de dados SQLite
    #[arg(long, default_value = "gitstore.db", global = true)]
    db: String,

    /// Caminho do arquivo de catálogo JSON (atualizado semanalmente)
    #[arg(long, default_value = "catalog.json", global = true)]
    catalog: PathBuf,

    #[command(subcommand)]
    cmd: Option<Cmd>,
}

#[derive(Subcommand)]
enum Cmd {
    /// Abre a interface gráfica da loja (padrão quando executado sem argumentos)
    Gui,
    /// Abre o Assistente de Instalação no Windows (cria atalhos na Área de Trabalho e Menu Iniciar)
    Install {
        /// Instala silenciosamente sem abrir a janela do assistente
        #[arg(long)]
        silent: bool,
    },
    /// Descobre repositórios no GitHub que têm instaladores em Releases e salva no catálogo
    Sync {
        /// Páginas de 50 resultados por consulta
        #[arg(long, default_value_t = 1)]
        pages: u32,
        #[arg(long, default_value_t = 100)]
        min_stars: u64,
        /// Consulta de busca (ex.: "topic:roguelike"). Pode repetir.
        #[arg(long)]
        query: Vec<String>,
        /// Adicionar repositórios específicos (ex.: --repo obsproject/obs-studio)
        #[arg(long)]
        repo: Vec<String>,
    },
    /// Importa o arquivo catalog.json para o banco SQLite local
    Import,
    /// Exporta o banco SQLite local para o arquivo catalog.json
    Export,
    /// Busca no catálogo local
    Search {
        text: Option<String>,
        /// app | game | tool
        #[arg(long)]
        kind: Option<String>,
        /// windows | linux | macos | android | java
        #[arg(long)]
        platform: Option<String>,
    },
    /// Mostra os detalhes completos de um item do catálogo
    Show { repo: String },
    /// Verifica a confiabilidade e precedentes de um repositório ANTES de baixar (dono/repo)
    Verify {
        repo: String,
        /// Consulta os hashes no VirusTotal (requer VIRUSTOTAL_API_KEY)
        #[arg(long)]
        vt: bool,
        /// Máximo de arquivos consultados no VirusTotal (limite grátis: 4/min)
        #[arg(long, default_value_t = 3)]
        max_vt: usize,
    },
    /// Baixa o melhor instalador para o seu PC, confere o SHA-256 oficial e registra como instalado
    Download {
        repo: String,
        /// Versão específica (tag da release), ex.: --tag v1.2.0. Sem ela, baixa a mais recente.
        #[arg(long)]
        tag: Option<String>,
    },
    /// Exporta a lista de apps instalados (e tempo de jogo) para reinstalar em outro PC
    LibraryExport {
        /// Arquivo de saída (padrão: rrs-biblioteca.json na pasta da RRS)
        #[arg(long)]
        out: Option<PathBuf>,
    },
    /// Importa uma biblioteca exportada: os apps entram na lista "Para reinstalar"
    LibraryImport { file: PathBuf },
    /// Verifica se há atualizações no GitHub para os aplicativos instalados no seu PC
    CheckUpdates,
    /// Traduz um arquivo de código C ou C++ (.c, .cpp, .h, .hpp) para Rust (.rs)
    Translate {
        /// Arquivo C/C++ de entrada
        input: PathBuf,
        /// Arquivo Rust (.rs) de saída opcional (se omitido, imprime na tela)
        #[arg(short, long)]
        output: Option<PathBuf>,
        /// Usa o modo FFI (#[repr(C)], extern "C") em vez de Rust idiomático
        #[arg(long)]
        ffi: bool,
        /// Valida o código Rust gerado usando o compilador `rustc`
        #[arg(long)]
        check: bool,
    },
}

const DEFAULT_QUERIES: &[&str] = &[
    "topic:game",
    "topic:open-source-game",
    "topic:desktop-app",
    "topic:tauri-app",
];

/// Pasta da RRS (onde ficam `catalog.json`, o banco e `downloads/`).
fn base_dir_of(catalog_path: &std::path::Path) -> PathBuf {
    catalog_path
        .parent()
        .filter(|p| !p.as_os_str().is_empty())
        .map(|p| p.to_path_buf())
        .unwrap_or_else(|| PathBuf::from("."))
}

/// Resolve caminhos relativos na pasta do próprio executável caso não existam no diretório atual,
/// permitindo abrir o aplicativo com clique duplo de qualquer lugar.
fn resolve_app_path(p: &std::path::Path) -> PathBuf {
    if p.is_absolute() || p.exists() {
        return p.to_path_buf();
    }
    if let Ok(exe) = std::env::current_exe() {
        if let Some(dir) = exe.parent() {
            // Se o executável estiver dentro de target/debug ou target/release, prefere a raiz do projeto se existir catalog.json lá
            if let Some(grand) = dir.parent().and_then(|d| d.parent()) {
                if grand.join("catalog.json").exists() {
                    return grand.join(p);
                }
            }
            return dir.join(p);
        }
    }
    p.to_path_buf()
}

#[cfg(target_os = "windows")]
fn hide_console_if_double_clicked() {
    #[link(name = "kernel32")]
    extern "system" {
        fn GetConsoleProcessList(lpdwProcessList: *mut u32, dwProcessCount: u32) -> u32;
        fn FreeConsole() -> i32;
    }
    unsafe {
        let mut pids = [0u32; 2];
        let count = GetConsoleProcessList(pids.as_mut_ptr(), 2);
        // Se count == 1, o processo foi aberto por clique duplo no Explorer (não dentro de um terminal existente)
        if count == 1 {
            let _ = FreeConsole();
        }
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    if cli.cmd.is_none() {
        #[cfg(target_os = "windows")]
        hide_console_if_double_clicked();
    }

    // Se o usuário deu clique duplo no arquivo "Instalador - RRS Riva Rust Store.exe" sem subcomando,
    // abre o Assistente Gráfico de Instalação!
    if cli.cmd.is_none() && installer::is_running_as_installer_exe() {
        return installer::run_installer_gui();
    }

    let db_path_buf = resolve_app_path(std::path::Path::new(&cli.db));
    let db_path_str = db_path_buf.to_string_lossy().to_string();
    let catalog_path = resolve_app_path(&cli.catalog);

    let conn = db::open(&db_path_str)?;

    // Sincroniza automaticamente catalog.json (ou catálogo embutido no .exe) -> SQLite na abertura
    let _ = db::import_catalog(&conn, &catalog_path);
    // Modo portátil: caminhos de apps instalados passam a ser relativos à pasta da RRS
    let _ = db::relativize_installed_paths(&conn, &base_dir_of(&catalog_path));

    match cli.cmd.unwrap_or(Cmd::Gui) {
        Cmd::Gui => {
            drop(conn);
            run_gui(db_path_str, catalog_path)
        }
        Cmd::Install { silent } => {
            drop(conn);
            if silent {
                let opts = installer::InstallOptions {
                    launch_after_install: false,
                    ..Default::default()
                };
                let dest = installer::perform_install(&opts)?;
                println!("✔ RRS - Riva Rust Store instalada com sucesso em: {}", dest.display());
                println!("✔ Atalhos criados na Área de Trabalho e no Menu Iniciar!");
                Ok(())
            } else {
                installer::run_installer_gui()
            }
        }
        Cmd::Sync {
            pages,
            min_stars,
            query,
            repo,
        } => {
            sync(&conn, &catalog_path, pages, min_stars, query, repo).await
        }
        Cmd::Import => {
            let n = db::import_catalog(&conn, &catalog_path)?;
            println!("✔ {n} itens importados de {} para {}.", catalog_path.display(), db_path_str);
            Ok(())
        }
        Cmd::Export => {
            let n = db::export_catalog(&conn, &catalog_path)?;
            println!("✔ {n} itens exportados de {} para {}.", db_path_str, catalog_path.display());
            Ok(())
        }
        Cmd::Search {
            text,
            kind,
            platform,
        } => {
            let apps = db::search(&conn, text.as_deref(), kind.as_deref(), platform.as_deref())?;
            if apps.is_empty() {
                println!("Nada encontrado com esses filtros.");
            }
            for a in apps {
                print_line(&a);
            }
            Ok(())
        }
        Cmd::Show { repo } => match db::get(&conn, &github::normalize_repo_name(&repo))? {
            Some(a) => {
                show(&a);
                Ok(())
            }
            None => bail!(
                "'{repo}' não está no catálogo. Use `rrs verify {repo}` ou adicione na interface."
            ),
        },
        Cmd::Verify { repo, vt, max_vt } => verify_cmd(&conn, &catalog_path, &repo, vt, max_vt).await,
        Cmd::Download { repo, tag } => download_cmd(&conn, &catalog_path, &repo, tag.as_deref()).await,
        Cmd::LibraryExport { out } => {
            let path = out.unwrap_or_else(|| base_dir_of(&catalog_path).join(db::LIBRARY_FILE_NAME));
            let n = db::export_library(&conn, &path)?;
            println!("✔ Biblioteca com {n} app(s) exportada para {}", path.display());
            Ok(())
        }
        Cmd::LibraryImport { file } => {
            let rep = db::import_library(&conn, &file)?;
            println!(
                "✔ {} app(s) na lista 'Para reinstalar'; {} já instalado(s) aqui (tempo de jogo mesclado).",
                rep.to_reinstall, rep.merged
            );
            if !rep.not_in_catalog.is_empty() {
                println!(
                    "⚠ Fora do catálogo local (use `rrs verify dono/repo`): {}",
                    rep.not_in_catalog.join(", ")
                );
            }
            Ok(())
        }
        Cmd::CheckUpdates => check_updates_cmd(&conn, &catalog_path).await,
        Cmd::Translate {
            input,
            output,
            ffi,
            check,
        } => translate_cmd(&input, output.as_ref(), ffi, check),
    }
}

fn run_gui(db_path: String, catalog_path: PathBuf) -> Result<()> {
    let rt = tokio::runtime::Handle::current();
    let options = eframe::NativeOptions {
        viewport: eframe::egui::ViewportBuilder::default()
            .with_title("RRS - Riva Rust Store — Catálogo Verificado & Tradutor C/C++ ➔ Rust")
            .with_inner_size([1260.0, 780.0])
            .with_min_inner_size([960.0, 580.0]),
        ..Default::default()
    };
    eframe::run_native(
        "RRS - Riva Rust Store",
        options,
        Box::new(move |cc| Ok(Box::new(gui::StoreGui::new(cc, db_path, catalog_path, rt)))),
    )
    .map_err(|e| anyhow::anyhow!("Erro ao iniciar interface gráfica: {e}"))
}

async fn sync(
    conn: &rusqlite::Connection,
    catalog_path: &PathBuf,
    pages: u32,
    min_stars: u64,
    extra_queries: Vec<String>,
    specific_repos: Vec<String>,
) -> Result<()> {
    let saved_token = db::get_setting(conn, "github_token");
    let gh = github::Github::new(saved_token.as_deref())?;
    if !gh.has_token {
        eprintln!("⚠ Sem GITHUB_TOKEN: limite de 60 requisições/hora.");
    }

    let mut saved = 0;

    // 1. Se o usuário passou repositórios específicos (--repo dono/repo)
    for r_name in specific_repos {
        let clean = github::normalize_repo_name(&r_name);
        print!("🔎 Buscando {clean}... ");
        match gh.repo(&clean).await {
            Ok(repo) => {
                if save_repo_if_installable(conn, &gh, repo).await? {
                    saved += 1;
                }
            }
            Err(e) => println!("erro: {e}"),
        }
    }

    // 2. Se não passou apenas repos específicos ou pediu query
    if saved == 0 || !extra_queries.is_empty() {
        let queries: Vec<String> = if extra_queries.is_empty() {
            DEFAULT_QUERIES.iter().map(|s| s.to_string()).collect()
        } else {
            extra_queries
        };

        for q in queries {
            for page in 1..=pages {
                let full_q = format!("{q} stars:>={min_stars} fork:false archived:false");
                println!("🔎 {full_q} (página {page})");
                let repos = gh.search_repos(&full_q, page).await?;
                if repos.is_empty() {
                    break;
                }
                for repo in repos {
                    if save_repo_if_installable(conn, &gh, repo).await? {
                        saved += 1;
                    }
                }
                tokio::time::sleep(Duration::from_secs(2)).await;
            }
        }
    }

    let total = db::export_catalog(conn, catalog_path)?;
    println!(
        "\n✔ Concluído: {saved} itens novos/atualizados. Catálogo salvo em {} ({total} itens no total).",
        catalog_path.display()
    );
    Ok(())
}

async fn save_repo_if_installable(
    conn: &rusqlite::Connection,
    gh: &github::Github,
    repo: github::Repo,
) -> Result<bool> {
    let Some(release) = gh.latest_release(&repo.full_name).await? else {
        println!("  · {} ignorado (sem Release)", repo.full_name);
        return Ok(false);
    };
    let assets: Vec<_> = release
        .assets
        .iter()
        .filter(|a| github::platform_of(&a.name).is_some())
        .cloned()
        .collect();
    if assets.is_empty() {
        println!("  · {} ignorado (Release sem instalador)", repo.full_name);
        return Ok(false);
    }
    let mut plats: Vec<&str> = assets
        .iter()
        .filter_map(|a| github::platform_of(&a.name))
        .collect();
    plats.sort();
    plats.dedup();

    let rep = verify::evaluate(&repo, Some(&release), None, &[]);
    let existing = db::get(conn, &repo.full_name)?;

    let app = db::App {
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
        latest_tag: release.tag_name,
        released_at: release.published_at.unwrap_or_default(),
        platforms: plats.join(","),
        assets,
        kind: existing.as_ref().and_then(|e| e.kind.clone()),
        category: existing.as_ref().and_then(|e| e.category.clone()),
        tags: existing.as_ref().and_then(|e| e.tags.clone()),
        summary: existing.as_ref().and_then(|e| e.summary.clone()),
        trust_score: rep.score,
        trust_level: rep.level.to_string(),
        trust_notes: rep.notes,
        attestation: String::new(),
        installed: None,
    };
    db::upsert(conn, &app)?;
    println!(
        "  ✔ {} ({}) [{}] — Confiança: {}/100",
        app.full_name, app.latest_tag, app.platforms, app.trust_score
    );
    Ok(true)
}

async fn verify_cmd(
    conn: &rusqlite::Connection,
    catalog_path: &PathBuf,
    repo_name: &str,
    use_vt: bool,
    max_vt: usize,
) -> Result<()> {
    let saved_token = db::get_setting(conn, "github_token");
    let gh = github::Github::new(saved_token.as_deref())?;
    let clean = github::normalize_repo_name(repo_name);
    let repo = gh.repo(&clean).await?;
    let release = gh.latest_release(&repo.full_name).await?;
    let contributors = gh.contributors(&repo.full_name).await?;

    let mut vt_results = Vec::new();
    if use_vt {
        let key = std::env::var("VIRUSTOTAL_API_KEY")
            .ok()
            .or_else(|| db::get_setting(conn, "virustotal_api_key"))
            .ok_or_else(|| anyhow::anyhow!("Defina VIRUSTOTAL_API_KEY para usar --vt"))?;
        let client = reqwest::Client::new();
        if let Some(r) = &release {
            let targets: Vec<_> = r
                .assets
                .iter()
                .filter(|a| github::platform_of(&a.name).is_some())
                .filter_map(|a| {
                    a.digest
                        .as_ref()
                        .map(|d| (a, d.trim_start_matches("sha256:").to_string()))
                })
                .take(max_vt)
                .collect();
            for (i, (asset, sha)) in targets.iter().enumerate() {
                if i > 0 {
                    println!("⏳ Aguardando 15s (limite grátis do VirusTotal)...");
                    tokio::time::sleep(Duration::from_secs(15)).await;
                }
                let stats = verify::vt_lookup(&client, &key, sha).await?;
                vt_results.push(verify::VtResult {
                    asset: asset.name.clone(),
                    stats,
                });
            }
        }
    }

    let report = verify::evaluate(&repo, release.as_ref(), contributors, &vt_results);
    println!("\n{}  —  {}", repo.full_name, repo.html_url);
    if let Some(r) = &release {
        println!("Última release: {}", r.tag_name);
        let inst_assets: Vec<_> = r
            .assets
            .iter()
            .filter(|a| github::platform_of(&a.name).is_some())
            .cloned()
            .collect();
        let best_idx = github::pick_best_asset(&inst_assets);
        for (i, a) in inst_assets.iter().enumerate() {
            let star = if Some(i) == best_idx { "★ " } else { "  " };
            println!(
                "  {star}{} ({:.1} MB)  {}",
                a.name,
                a.size as f64 / 1_048_576.0,
                a.digest.as_deref().unwrap_or("sem digest")
            );
        }
        let attestation = attestation_of_best(&gh, &repo.full_name, &inst_assets).await;
        match attestation.as_str() {
            github::ATTESTATION_PRESENT => println!(
                "🏗 Atestado de build do GitHub: presente para o arquivo ★ (a RRS não confere a assinatura; use `gh release verify-asset`)."
            ),
            github::ATTESTATION_ABSENT => println!("🏗 Atestado de build do GitHub: ausente para o arquivo ★."),
            _ => {}
        }

        // Salva/atualiza no banco caso tenha instaladores
        if !inst_assets.is_empty() {
            let mut plats: Vec<&str> = inst_assets
                .iter()
                .filter_map(|a| github::platform_of(&a.name))
                .collect();
            plats.sort();
            plats.dedup();
            let existing = db::get(conn, &repo.full_name)?;
            let app = db::App {
                full_name: repo.full_name.clone(),
                description: repo.description.unwrap_or_default(),
                url: repo.html_url,
                stars: repo.stargazers_count,
                license: repo.license.and_then(|l| l.spdx_id).unwrap_or_default(),
                topics: repo.topics.join(","),
                language: repo.language.clone().unwrap_or_default(),
                created_at: repo.created_at,
                pushed_at: repo.pushed_at.unwrap_or_default(),
                fork: repo.fork,
                archived: repo.archived,
                has_issues: repo.has_issues,
                latest_tag: r.tag_name.clone(),
                released_at: r.published_at.clone().unwrap_or_default(),
                platforms: plats.join(","),
                assets: inst_assets,
                kind: existing.as_ref().and_then(|e| e.kind.clone()),
                category: existing.as_ref().and_then(|e| e.category.clone()),
                tags: existing.as_ref().and_then(|e| e.tags.clone()),
                summary: existing.as_ref().and_then(|e| e.summary.clone()),
                trust_score: report.score,
                trust_level: report.level.to_string(),
                trust_notes: report.notes.clone(),
                attestation,
                installed: None,
            };
            let _ = db::upsert(conn, &app);
            let _ = db::export_catalog(conn, catalog_path);
        }
    }
    println!("\nNota de confiança: {}/100  {}\n", report.score, report.level);
    for n in &report.notes {
        println!("  {n}");
    }
    if !use_vt {
        println!("\n💡 Use --vt para checar os arquivos no VirusTotal sem baixá-los.");
    }
    println!("\n⚠ Esta nota é um indicador de precedentes, não uma garantia absoluta de segurança.");
    Ok(())
}

/// "presente"/"ausente" para o instalador recomendado com digest; "" quando não deu para consultar.
async fn attestation_of_best(gh: &github::Github, full_name: &str, assets: &[github::Asset]) -> String {
    let Some(digest) = github::pick_best_asset(assets).and_then(|i| assets[i].digest.clone()) else {
        return String::new();
    };
    match gh.has_attestation(full_name, &digest).await {
        Ok(true) => github::ATTESTATION_PRESENT.into(),
        Ok(false) => github::ATTESTATION_ABSENT.into(),
        Err(_) => String::new(),
    }
}

async fn download_cmd(
    conn: &rusqlite::Connection,
    catalog_path: &PathBuf,
    repo_name: &str,
    tag: Option<&str>,
) -> Result<()> {
    let clean = github::normalize_repo_name(repo_name);
    let app = match db::get(conn, &clean)? {
        Some(a) => a,
        None => {
            println!("'{clean}' ainda não está no catálogo local. Consultando GitHub...");
            verify_cmd(conn, catalog_path, &clean, false, 1).await?;
            db::get(conn, &clean)?
                .ok_or_else(|| anyhow::anyhow!("Repositório '{clean}' não possui instaladores."))?
        }
    };

    // Versão pedida com --tag (versão anterior) ou a mais recente do catálogo
    let (tag_name, assets) = match tag {
        Some(t) if t != app.latest_tag => {
            let gh = github::Github::new(db::get_setting(conn, "github_token").as_deref())?;
            let rel = gh.release_by_tag(&app.full_name, t).await?;
            (rel.tag_name.clone(), rel.installable_assets())
        }
        _ => (app.latest_tag.clone(), app.assets.clone()),
    };

    let best_idx = github::pick_best_asset(&assets)
        .ok_or_else(|| anyhow::anyhow!("Nenhum instalador disponível para {} {tag_name}.", app.full_name))?;
    let asset = &assets[best_idx];

    println!(
        "⬇ Baixando {} ({:.1} MB) de {} ({})...",
        asset.name,
        asset.size as f64 / 1_048_576.0,
        app.full_name,
        tag_name
    );
    let base_dir = base_dir_of(catalog_path);
    let (final_path, sha256, verified) =
        downloader::download_and_verify(&app.full_name, &tag_name, asset, &base_dir, None).await?;

    db::mark_installed(
        conn,
        &app.full_name,
        &tag_name,
        &db::to_stored_path(&base_dir, &final_path),
        &sha256,
        verified,
    )?;

    if verified {
        println!("✅ SHA-256 verificado com o digest oficial do GitHub: {sha256}");
    } else {
        println!("✅ Arquivo salvo. SHA-256 calculado: {sha256}");
    }
    println!("📂 Salvo em: {}", final_path.display());
    Ok(())
}

async fn check_updates_cmd(conn: &rusqlite::Connection, catalog_path: &PathBuf) -> Result<()> {
    let all = db::list_all(conn)?;
    let installed: Vec<_> = all.into_iter().filter(|a| a.installed.is_some()).collect();
    if installed.is_empty() {
        println!("Nenhum aplicativo registrado como instalado ainda.");
        return Ok(());
    }
    let saved_token = db::get_setting(conn, "github_token");
    let gh = github::Github::new(saved_token.as_deref())?;
    let mut updates = 0;
    for mut app in installed {
        let inst_tag = app
            .installed
            .as_ref()
            .map(|i| i.installed_tag.clone())
            .unwrap_or_default();
        if let Some(rel) = gh.latest_release(&app.full_name).await? {
            if rel.tag_name != inst_tag {
                println!(
                    "⬆ {} tem atualização: {} → {}",
                    app.full_name, inst_tag, rel.tag_name
                );
                updates += 1;
                app.latest_tag = rel.tag_name;
                app.released_at = rel.published_at.unwrap_or_default();
                app.attestation.clear();
                app.assets = rel.assets.into_iter().filter(|a| github::platform_of(&a.name).is_some()).collect();
                db::upsert(conn, &app)?;
            } else {
                println!("✔ {} está atualizado ({})", app.full_name, inst_tag);
            }
        }
    }
    let _ = db::export_catalog(conn, catalog_path);
    println!("\nTotal de atualizações disponíveis: {updates}");
    Ok(())
}

fn translate_cmd(
    input: &PathBuf,
    output: Option<&PathBuf>,
    ffi: bool,
    check: bool,
) -> Result<()> {
    let src = std::fs::read_to_string(input)
        .with_context(|| format!("falha ao ler arquivo C/C++: {}", input.display()))?;
    let mode = if ffi {
        translator::TranslateMode::FfiBindings
    } else {
        translator::TranslateMode::SafeRust
    };
    let res = translator::translate_c_cpp_to_rust(&src, mode);

    if let Some(out_path) = output {
        std::fs::write(out_path, &res.rust_code)
            .with_context(|| format!("falha ao salvar {}", out_path.display()))?;
        println!("✔ Código Rust salvo em: {}", out_path.display());
    } else {
        println!("{}", res.rust_code);
    }

    for n in &res.notes {
        eprintln!("• {n}");
    }

    if check {
        match translator::verify_rust_syntax(&res.rust_code) {
            Ok(msg) => println!("{msg}"),
            Err(err) => bail!("Falha na validação com rustc:\n{err}"),
        }
    }
    Ok(())
}

fn print_line(a: &db::App) {
    let kind = a.kind.as_deref().unwrap_or("?");
    let inst = if a.has_update() {
        " [⬆ ATUALIZAÇÃO]"
    } else if a.installed.is_some() {
        " [✔ INSTALADO]"
    } else {
        ""
    };
    println!(
        "⭐{:<6} 🛡️{:>3}/100 [{:<4}] {:<36} ({}) — {}{}",
        a.stars,
        a.trust_score,
        kind,
        a.full_name,
        a.platforms,
        a.display_summary(),
        inst
    );
}

fn show(a: &db::App) {
    println!("{}  ({})", a.full_name, a.url);
    println!(
        "⭐ {}   Licença: {}   Última release: {}   Confiança: {}/100 ({})",
        a.stars, a.license, a.latest_tag, a.trust_score, a.trust_level
    );
    println!(
        "Tipo: {}   Categoria: {}",
        a.kind.as_deref().unwrap_or("-"),
        a.category.as_deref().unwrap_or("-")
    );
    println!("Plataformas: {}", a.platforms);
    println!("Tags: {}", a.tags.as_deref().unwrap_or(&a.topics));
    println!("Resumo: {}", a.display_summary());
    if !a.trust_notes.is_empty() {
        println!("\nPrecedentes de Segurança:");
        for n in &a.trust_notes {
            println!("  {n}");
        }
    }
    let best_idx = github::pick_best_asset(&a.assets);
    println!("\nArquivos da Release:");
    for (i, x) in a.assets.iter().enumerate() {
        let rec = if Some(i) == best_idx { "★ [Recomendado] " } else { "" };
        println!(
            "  • {}{} ({:.1} MB) → {}",
            rec,
            x.name,
            x.size as f64 / 1_048_576.0,
            x.digest.as_deref().unwrap_or("sem digest")
        );
    }
    println!("\n⬇ Para baixar com verificação SHA-256: gitstore download {}", a.full_name);
}
