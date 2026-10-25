//! Banco de dados SQLite local + sincronização com `catalog.json` e controle de instalados.

use crate::github::Asset;
use crate::verify;
use anyhow::{Context, Result};
use rusqlite::{params, Connection, Row};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CatalogEntry {
    pub full_name: String,
    pub description: String,
    pub url: String,
    pub stars: u64,
    pub license: String,
    pub topics: String,
    #[serde(default)]
    pub language: String,
    #[serde(default)]
    pub created_at: String,
    pub pushed_at: String,
    #[serde(default)]
    pub fork: bool,
    #[serde(default)]
    pub archived: bool,
    #[serde(default = "default_true")]
    pub has_issues: bool,
    pub latest_tag: String,
    /// Data de publicação da última release (RFC 3339), quando conhecida.
    #[serde(default)]
    pub released_at: String,
    pub platforms: String,
    pub assets: Vec<Asset>,
    pub kind: Option<String>,
    pub category: Option<String>,
    pub tags: Option<String>,
    pub summary: Option<String>,
    #[serde(default)]
    pub trust_score: i32,
    #[serde(default)]
    pub trust_level: String,
    #[serde(default)]
    pub trust_notes: Vec<String>,
    /// Atestado de build do GitHub para o instalador recomendado: "presente", "ausente" ou "" (não consultado).
    #[serde(default)]
    pub attestation: String,
}

fn default_true() -> bool {
    true
}

#[derive(Debug, Clone)]
pub struct InstalledInfo {
    pub installed_tag: String,
    pub file_path: String,
    pub sha256: String,
    pub verified_github_digest: bool,
    pub installed_at: String,
    pub last_played: Option<String>,
    pub playtime_seconds: u64,
}

impl InstalledInfo {
    pub fn formatted_playtime(&self) -> String {
        if self.playtime_seconds == 0 {
            "Nunca jogado".to_string()
        } else if self.playtime_seconds < 60 {
            format!("{} segundos", self.playtime_seconds)
        } else if self.playtime_seconds < 3600 {
            format!("{} min", self.playtime_seconds / 60)
        } else {
            let hours = self.playtime_seconds as f64 / 3600.0;
            format!("{:.1} horas", hours)
        }
    }

    pub fn formatted_last_played(&self) -> String {
        match &self.last_played {
            Some(ts) if !ts.is_empty() => {
                if let Ok(dt) = chrono::DateTime::parse_from_rfc3339(ts) {
                    dt.format("%d/%m/%Y às %H:%M").to_string()
                } else {
                    ts.clone()
                }
            }
            _ => "Nunca".to_string(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct App {
    pub full_name: String,
    pub description: String,
    pub url: String,
    pub stars: u64,
    pub license: String,
    pub topics: String,
    pub language: String,
    pub created_at: String,
    pub pushed_at: String,
    pub fork: bool,
    pub archived: bool,
    pub has_issues: bool,
    pub latest_tag: String,
    pub released_at: String,
    pub platforms: String,
    pub assets: Vec<Asset>,
    pub kind: Option<String>,
    pub category: Option<String>,
    pub tags: Option<String>,
    pub summary: Option<String>,
    pub trust_score: i32,
    pub trust_level: String,
    pub trust_notes: Vec<String>,
    pub attestation: String,
    pub installed: Option<InstalledInfo>,
}

impl App {
    pub fn is_rust(&self) -> bool {
        crate::github::is_rust_language(&self.language)
    }

    pub fn has_update(&self) -> bool {
        match &self.installed {
            Some(inst) => !self.latest_tag.is_empty() && inst.installed_tag != self.latest_tag,
            None => false,
        }
    }

    pub fn display_summary(&self) -> &str {
        self.summary
            .as_deref()
            .filter(|s| !s.is_empty())
            .unwrap_or(&self.description)
    }

    pub fn recompute_trust(&mut self, contributors: Option<u64>, vt: &[verify::VtResult]) {
        let rep = verify::evaluate_metadata(
            &self.created_at,
            &self.pushed_at,
            self.stars,
            contributors,
            &self.license,
            self.has_issues,
            self.fork,
            self.archived,
            !self.latest_tag.is_empty(),
            &self.assets,
            vt,
        );
        self.trust_score = rep.score;
        self.trust_level = rep.level.to_string();
        self.trust_notes = rep.notes;
    }

    pub fn to_catalog_entry(&self) -> CatalogEntry {
        CatalogEntry {
            full_name: self.full_name.clone(),
            description: self.description.clone(),
            url: self.url.clone(),
            stars: self.stars,
            license: self.license.clone(),
            topics: self.topics.clone(),
            language: self.language.clone(),
            created_at: self.created_at.clone(),
            pushed_at: self.pushed_at.clone(),
            fork: self.fork,
            archived: self.archived,
            has_issues: self.has_issues,
            latest_tag: self.latest_tag.clone(),
            released_at: self.released_at.clone(),
            platforms: self.platforms.clone(),
            assets: self.assets.clone(),
            kind: self.kind.clone(),
            category: self.category.clone(),
            tags: self.tags.clone(),
            summary: self.summary.clone(),
            trust_score: self.trust_score,
            trust_level: self.trust_level.clone(),
            trust_notes: self.trust_notes.clone(),
            attestation: self.attestation.clone(),
        }
    }
}

pub fn open(path: &str) -> Result<Connection> {
    let conn = Connection::open(path)?;
    conn.execute_batch(
        "CREATE TABLE IF NOT EXISTS apps (
            full_name        TEXT PRIMARY KEY,
            description      TEXT NOT NULL DEFAULT '',
            url              TEXT NOT NULL,
            stars            INTEGER NOT NULL DEFAULT 0,
            license          TEXT NOT NULL DEFAULT '',
            topics           TEXT NOT NULL DEFAULT '',
            created_at       TEXT NOT NULL DEFAULT '',
            pushed_at        TEXT NOT NULL DEFAULT '',
            fork             INTEGER NOT NULL DEFAULT 0,
            archived         INTEGER NOT NULL DEFAULT 0,
            has_issues       INTEGER NOT NULL DEFAULT 1,
            latest_tag       TEXT NOT NULL DEFAULT '',
            platforms        TEXT NOT NULL DEFAULT '',
            assets_json      TEXT NOT NULL DEFAULT '[]',
            kind             TEXT,
            category         TEXT,
            tags             TEXT,
            summary          TEXT,
            trust_score      INTEGER NOT NULL DEFAULT 0,
            trust_level      TEXT NOT NULL DEFAULT '',
            trust_notes_json TEXT NOT NULL DEFAULT '[]'
        );
        CREATE TABLE IF NOT EXISTS installed (
            full_name        TEXT PRIMARY KEY,
            installed_tag    TEXT NOT NULL,
            file_path        TEXT NOT NULL,
            sha256           TEXT NOT NULL,
            verified         INTEGER NOT NULL DEFAULT 0,
            installed_at     TEXT NOT NULL,
            last_played      TEXT,
            playtime_seconds INTEGER NOT NULL DEFAULT 0
        );
        CREATE TABLE IF NOT EXISTS settings (
            key   TEXT PRIMARY KEY,
            value TEXT NOT NULL
        );
        CREATE TABLE IF NOT EXISTS library_wishlist (
            full_name        TEXT PRIMARY KEY,
            playtime_seconds INTEGER NOT NULL DEFAULT 0,
            last_played      TEXT,
            added_at         TEXT NOT NULL
        );
        CREATE TABLE IF NOT EXISTS stats (
            key   TEXT PRIMARY KEY,
            value INTEGER NOT NULL DEFAULT 0
        );
        CREATE TABLE IF NOT EXISTS achievements (
            id          TEXT PRIMARY KEY,
            unlocked_at TEXT NOT NULL
        );",
    )?;

    // Migração suave para bancos criados na v0.1
    let alter_cols = [
        "ALTER TABLE apps ADD COLUMN created_at TEXT NOT NULL DEFAULT ''",
        "ALTER TABLE apps ADD COLUMN fork INTEGER NOT NULL DEFAULT 0",
        "ALTER TABLE apps ADD COLUMN archived INTEGER NOT NULL DEFAULT 0",
        "ALTER TABLE apps ADD COLUMN has_issues INTEGER NOT NULL DEFAULT 1",
        "ALTER TABLE apps ADD COLUMN trust_score INTEGER NOT NULL DEFAULT 0",
        "ALTER TABLE apps ADD COLUMN trust_level TEXT NOT NULL DEFAULT ''",
        "ALTER TABLE apps ADD COLUMN trust_notes_json TEXT NOT NULL DEFAULT '[]'",
        "ALTER TABLE installed ADD COLUMN last_played TEXT",
        "ALTER TABLE installed ADD COLUMN playtime_seconds INTEGER NOT NULL DEFAULT 0",
        "ALTER TABLE apps ADD COLUMN language TEXT NOT NULL DEFAULT ''",
        "ALTER TABLE apps ADD COLUMN released_at TEXT NOT NULL DEFAULT ''",
        "ALTER TABLE apps ADD COLUMN attestation TEXT NOT NULL DEFAULT ''",
    ];
    for sql in alter_cols {
        let _ = conn.execute(sql, []);
    }

    Ok(conn)
}

const SELECT_SQL: &str = "
    SELECT a.full_name, a.description, a.url, a.stars, a.license, a.topics,
           a.created_at, a.pushed_at, a.fork, a.archived, a.has_issues,
           a.latest_tag, a.platforms, a.assets_json,
           a.kind, a.category, a.tags, a.summary,
           a.trust_score, a.trust_level, a.trust_notes_json,
           i.installed_tag, i.file_path, i.sha256, i.verified, i.installed_at,
           i.last_played, i.playtime_seconds,
           a.language, a.released_at, a.attestation
    FROM apps a
    LEFT JOIN installed i ON lower(a.full_name) = lower(i.full_name)
";

fn from_row(r: &Row) -> rusqlite::Result<App> {
    let assets_json: String = r.get(13)?;
    let notes_json: String = r.get(20)?;
    let assets: Vec<Asset> = serde_json::from_str(&assets_json).unwrap_or_default();
    let trust_notes: Vec<String> = serde_json::from_str(&notes_json).unwrap_or_default();

    let installed_tag: Option<String> = r.get(21)?;
    let installed = installed_tag.map(|tag| InstalledInfo {
        installed_tag: tag,
        file_path: r.get::<_, Option<String>>(22).ok().flatten().unwrap_or_default(),
        sha256: r.get::<_, Option<String>>(23).ok().flatten().unwrap_or_default(),
        verified_github_digest: r.get::<_, Option<i64>>(24).ok().flatten().unwrap_or(0) != 0,
        installed_at: r.get::<_, Option<String>>(25).ok().flatten().unwrap_or_default(),
        last_played: r.get::<_, Option<String>>(26).ok().flatten(),
        playtime_seconds: r.get::<_, Option<i64>>(27).ok().flatten().unwrap_or(0) as u64,
    });

    let mut app = App {
        full_name: r.get(0)?,
        description: r.get(1)?,
        url: r.get(2)?,
        stars: r.get::<_, i64>(3)? as u64,
        license: r.get(4)?,
        topics: r.get(5)?,
        language: r.get(28)?,
        created_at: r.get(6)?,
        pushed_at: r.get(7)?,
        fork: r.get::<_, i64>(8)? != 0,
        archived: r.get::<_, i64>(9)? != 0,
        has_issues: r.get::<_, i64>(10)? != 0,
        latest_tag: r.get(11)?,
        released_at: r.get(29)?,
        platforms: r.get(12)?,
        assets,
        kind: r.get(14)?,
        category: r.get(15)?,
        tags: r.get(16)?,
        summary: r.get(17)?,
        trust_score: r.get(18)?,
        trust_level: r.get(19)?,
        trust_notes,
        attestation: r.get(30)?,
        installed,
    };

    if app.trust_level.is_empty() || app.trust_notes.is_empty() {
        app.recompute_trust(None, &[]);
    }
    Ok(app)
}

/// Insere ou atualiza um app no banco SQLite, preservando a curadoria da IA caso o novo registro não a traga.
pub fn upsert(conn: &Connection, a: &App) -> Result<()> {
    let assets_json = serde_json::to_string(&a.assets)?;
    let notes_json = serde_json::to_string(&a.trust_notes)?;
    conn.execute(
        "INSERT INTO apps (
            full_name, description, url, stars, license, topics,
            created_at, pushed_at, fork, archived, has_issues,
            latest_tag, platforms, assets_json,
            kind, category, tags, summary,
            trust_score, trust_level, trust_notes_json,
            language, released_at, attestation
         )
         VALUES (?1,?2,?3,?4,?5,?6,?7,?8,?9,?10,?11,?12,?13,?14,?15,?16,?17,?18,?19,?20,?21,?22,?23,?24)
         ON CONFLICT(full_name) DO UPDATE SET
            language=CASE WHEN excluded.language != '' THEN excluded.language ELSE apps.language END,
            released_at=CASE WHEN excluded.released_at != '' THEN excluded.released_at
                             WHEN excluded.latest_tag != apps.latest_tag THEN ''
                             ELSE apps.released_at END,
            attestation=CASE WHEN excluded.attestation != '' THEN excluded.attestation
                             WHEN excluded.latest_tag != apps.latest_tag THEN ''
                             ELSE apps.attestation END,
            description=excluded.description,
            url=excluded.url,
            stars=excluded.stars,
            license=excluded.license,
            topics=excluded.topics,
            created_at=CASE WHEN excluded.created_at != '' THEN excluded.created_at ELSE apps.created_at END,
            pushed_at=excluded.pushed_at,
            fork=excluded.fork,
            archived=excluded.archived,
            has_issues=excluded.has_issues,
            latest_tag=excluded.latest_tag,
            platforms=excluded.platforms,
            assets_json=excluded.assets_json,
            kind=COALESCE(excluded.kind, apps.kind),
            category=COALESCE(excluded.category, apps.category),
            tags=COALESCE(excluded.tags, apps.tags),
            summary=COALESCE(excluded.summary, apps.summary),
            trust_score=excluded.trust_score,
            trust_level=excluded.trust_level,
            trust_notes_json=excluded.trust_notes_json",
        params![
            a.full_name,
            a.description,
            a.url,
            a.stars as i64,
            a.license,
            a.topics,
            a.created_at,
            a.pushed_at,
            a.fork as i64,
            a.archived as i64,
            a.has_issues as i64,
            a.latest_tag,
            a.platforms,
            assets_json,
            a.kind,
            a.category,
            a.tags,
            a.summary,
            a.trust_score,
            a.trust_level,
            notes_json,
            a.language,
            a.released_at,
            a.attestation,
        ],
    )?;
    Ok(())
}

pub fn mark_installed(
    conn: &Connection,
    full_name: &str,
    tag: &str,
    file_path: &str,
    sha256: &str,
    verified: bool,
) -> Result<()> {
    let now = chrono::Utc::now().to_rfc3339();
    conn.execute(
        "INSERT INTO installed (full_name, installed_tag, file_path, sha256, verified, installed_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6)
         ON CONFLICT(full_name) DO UPDATE SET
            installed_tag=excluded.installed_tag,
            file_path=excluded.file_path,
            sha256=excluded.sha256,
            verified=excluded.verified,
            installed_at=excluded.installed_at",
        params![full_name, tag, file_path, sha256, verified as i64, now],
    )?;
    // Reinstalação de um app vindo de biblioteca importada: traz o tempo de jogo e sai da lista.
    conn.execute(
        "UPDATE installed SET
            playtime_seconds = MAX(playtime_seconds,
                (SELECT w.playtime_seconds FROM library_wishlist w WHERE lower(w.full_name) = lower(?1))),
            last_played = COALESCE(last_played,
                (SELECT w.last_played FROM library_wishlist w WHERE lower(w.full_name) = lower(?1)))
         WHERE lower(full_name) = lower(?1)
           AND EXISTS (SELECT 1 FROM library_wishlist w WHERE lower(w.full_name) = lower(?1))",
        [full_name],
    )?;
    conn.execute(
        "DELETE FROM library_wishlist WHERE lower(full_name) = lower(?1)",
        [full_name],
    )?;
    Ok(())
}

pub fn unmark_installed(conn: &Connection, full_name: &str) -> Result<()> {
    conn.execute(
        "DELETE FROM installed WHERE lower(full_name) = lower(?1)",
        [full_name],
    )?;
    Ok(())
}

pub fn record_playtime(conn: &Connection, full_name: &str, elapsed_seconds: u64) -> Result<()> {
    let now = chrono::Utc::now().to_rfc3339();
    conn.execute(
        "UPDATE installed
         SET playtime_seconds = COALESCE(playtime_seconds, 0) + ?1,
             last_played = ?2
         WHERE lower(full_name) = lower(?3)",
        params![elapsed_seconds as i64, now, full_name],
    )?;
    Ok(())
}

pub fn get_setting(conn: &Connection, key: &str) -> Option<String> {
    conn.query_row("SELECT value FROM settings WHERE key = ?1", [key], |r| r.get(0))
        .ok()
        .filter(|s: &String| !s.trim().is_empty())
}

pub fn set_setting(conn: &Connection, key: &str, value: &str) -> Result<()> {
    conn.execute(
        "INSERT INTO settings (key, value) VALUES (?1, ?2)
         ON CONFLICT(key) DO UPDATE SET value = excluded.value",
        params![key, value.trim()],
    )?;
    Ok(())
}

pub fn list_all(conn: &Connection) -> Result<Vec<App>> {
    let sql = format!("{SELECT_SQL} ORDER BY a.stars DESC");
    let mut st = conn.prepare(&sql)?;
    let rows = st.query_map([], from_row)?;
    Ok(rows.collect::<rusqlite::Result<_>>()?)
}

pub fn get(conn: &Connection, full_name: &str) -> Result<Option<App>> {
    let sql = format!("{SELECT_SQL} WHERE lower(a.full_name) = lower(?1)");
    let mut st = conn.prepare(&sql)?;
    let mut rows = st.query_map([full_name], from_row)?;
    Ok(rows.next().transpose()?)
}

pub fn search(
    conn: &Connection,
    text: Option<&str>,
    kind: Option<&str>,
    platform: Option<&str>,
) -> Result<Vec<App>> {
    let sql = format!(
        "{SELECT_SQL}
         WHERE (?1 IS NULL OR a.full_name LIKE ?1 OR a.description LIKE ?1 OR a.topics LIKE ?1
                OR a.tags LIKE ?1 OR a.summary LIKE ?1 OR a.category LIKE ?1)
           AND (?2 IS NULL OR a.kind = ?2)
           AND (?3 IS NULL OR a.platforms LIKE ?3)
         ORDER BY a.stars DESC LIMIT 100"
    );
    let like = text.map(|t| format!("%{t}%"));
    let plat = platform.map(|p| format!("%{p}%"));
    let mut st = conn.prepare(&sql)?;
    let rows = st.query_map(params![like, kind, plat], from_row)?;
    Ok(rows.collect::<rusqlite::Result<_>>()?)
}

// ─── Caminhos portáteis ─────────────────────────────────────────────────────

/// Caminho gravado no banco: relativo à pasta da RRS quando o arquivo está dentro dela,
/// para a instalação continuar funcionando se o pendrive ou a pasta mudarem de letra/lugar.
pub fn to_stored_path(base_dir: &Path, path: &Path) -> String {
    match path.strip_prefix(base_dir) {
        Ok(rel) if !rel.as_os_str().is_empty() => rel.to_string_lossy().to_string(),
        _ => path.to_string_lossy().to_string(),
    }
}

/// Caminho real a partir do que está gravado no banco.
pub fn resolve_stored_path(base_dir: &Path, stored: &str) -> PathBuf {
    let p = Path::new(stored);
    if p.is_absolute() {
        p.to_path_buf()
    } else {
        base_dir.join(p)
    }
}

/// Trecho `downloads\...` de um caminho absoluto (onde a RRS guarda os apps).
fn downloads_suffix(path: &Path) -> Option<PathBuf> {
    let comps: Vec<_> = path.components().collect();
    let idx = comps
        .iter()
        .rposition(|c| c.as_os_str().eq_ignore_ascii_case("downloads"))?;
    Some(comps[idx..].iter().collect())
}

/// Converte para relativo os caminhos absolutos de apps instalados (bancos anteriores ao modo portátil).
/// Se o caminho absoluto não existe mais (outra letra de unidade), procura o mesmo `downloads\...` na pasta atual.
pub fn relativize_installed_paths(conn: &Connection, base_dir: &Path) -> Result<usize> {
    let rows: Vec<(String, String)> = {
        let mut st = conn.prepare("SELECT full_name, file_path FROM installed")?;
        let mapped = st.query_map([], |r| Ok((r.get(0)?, r.get(1)?)))?;
        mapped.collect::<rusqlite::Result<_>>()?
    };
    let mut changed = 0;
    for (full_name, stored) in rows {
        let p = Path::new(&stored);
        if !p.is_absolute() {
            continue;
        }
        let new_value = if p.strip_prefix(base_dir).is_ok() {
            Some(to_stored_path(base_dir, p))
        } else if !p.exists() {
            downloads_suffix(p)
                .filter(|suffix| base_dir.join(suffix).exists())
                .map(|suffix| suffix.to_string_lossy().to_string())
        } else {
            None
        };
        if let Some(v) = new_value {
            conn.execute(
                "UPDATE installed SET file_path = ?1 WHERE full_name = ?2",
                params![v, full_name],
            )?;
            changed += 1;
        }
    }
    Ok(changed)
}

// ─── Estatísticas e conquistas ────────────────────────────────────────────

/// Soma `by` ao contador `key` e devolve o novo valor.
pub fn bump_stat(conn: &Connection, key: &str, by: i64) -> Result<i64> {
    conn.execute(
        "INSERT INTO stats (key, value) VALUES (?1, ?2)
         ON CONFLICT(key) DO UPDATE SET value = value + excluded.value",
        params![key, by],
    )?;
    Ok(conn.query_row("SELECT value FROM stats WHERE key = ?1", [key], |r| r.get(0))?)
}

pub fn get_stats(conn: &Connection) -> Result<std::collections::HashMap<String, i64>> {
    let mut st = conn.prepare("SELECT key, value FROM stats")?;
    let rows = st.query_map([], |r| Ok((r.get::<_, String>(0)?, r.get::<_, i64>(1)?)))?;
    Ok(rows.collect::<rusqlite::Result<_>>()?)
}

/// Conquistas já desbloqueadas: (id, data).
pub fn unlocked_achievements(conn: &Connection) -> Result<Vec<(String, String)>> {
    let mut st = conn.prepare("SELECT id, unlocked_at FROM achievements ORDER BY unlocked_at")?;
    let rows = st.query_map([], |r| Ok((r.get(0)?, r.get(1)?)))?;
    Ok(rows.collect::<rusqlite::Result<_>>()?)
}

/// Marca a conquista; `true` se ela foi desbloqueada agora.
pub fn unlock_achievement(conn: &Connection, id: &str) -> Result<bool> {
    let n = conn.execute(
        "INSERT OR IGNORE INTO achievements (id, unlocked_at) VALUES (?1, ?2)",
        params![id, chrono::Utc::now().to_rfc3339()],
    )?;
    Ok(n > 0)
}

// ─── Exportar / importar a biblioteca ──────────────────────────────────────

pub const LIBRARY_FORMAT: &str = "rrs-biblioteca";
pub const LIBRARY_FILE_NAME: &str = "rrs-biblioteca.json";

#[derive(Debug, Serialize, Deserialize)]
pub struct LibraryFile {
    pub formato: String,
    pub versao: u32,
    #[serde(default)]
    pub exportado_em: String,
    pub apps: Vec<LibraryItem>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LibraryItem {
    pub full_name: String,
    /// Versão instalada no PC de origem ("" = estava na lista para reinstalar).
    #[serde(default)]
    pub installed_tag: String,
    #[serde(default)]
    pub playtime_seconds: u64,
    #[serde(default)]
    pub last_played: Option<String>,
}

#[derive(Debug, Default)]
pub struct LibraryImport {
    /// Apps que entraram na lista "Para reinstalar".
    pub to_reinstall: usize,
    /// Apps que já estavam instalados aqui (tempo de jogo mesclado).
    pub merged: usize,
    /// Repositórios que não estão no catálogo local (use "Verificar Repo" para adicioná-los).
    pub not_in_catalog: Vec<String>,
}

/// Apps instalados mais os que aguardam reinstalação, no formato do arquivo de biblioteca.
pub fn library_items(conn: &Connection) -> Result<Vec<LibraryItem>> {
    let mut st = conn.prepare(
        "SELECT full_name, installed_tag, playtime_seconds, last_played FROM installed
         UNION ALL
         SELECT w.full_name, '', w.playtime_seconds, w.last_played FROM library_wishlist w
          WHERE NOT EXISTS (SELECT 1 FROM installed i WHERE lower(i.full_name) = lower(w.full_name))
         ORDER BY 1",
    )?;
    let rows = st.query_map([], |r| {
        Ok(LibraryItem {
            full_name: r.get(0)?,
            installed_tag: r.get(1)?,
            playtime_seconds: r.get::<_, i64>(2)?.max(0) as u64,
            last_played: r.get(3)?,
        })
    })?;
    Ok(rows.collect::<rusqlite::Result<_>>()?)
}

pub fn export_library(conn: &Connection, path: &Path) -> Result<usize> {
    let file = LibraryFile {
        formato: LIBRARY_FORMAT.into(),
        versao: 1,
        exportado_em: chrono::Utc::now().to_rfc3339(),
        apps: library_items(conn)?,
    };
    std::fs::write(path, serde_json::to_string_pretty(&file)?)
        .with_context(|| format!("falha ao salvar a biblioteca em {}", path.display()))?;
    Ok(file.apps.len())
}

pub fn import_library(conn: &Connection, path: &Path) -> Result<LibraryImport> {
    let raw = std::fs::read_to_string(path)
        .with_context(|| format!("falha ao ler {}", path.display()))?;
    import_library_str(conn, &raw)
}

pub fn import_library_str(conn: &Connection, raw: &str) -> Result<LibraryImport> {
    let file: LibraryFile = serde_json::from_str(raw.trim_start_matches('\u{feff}'))
        .context("arquivo de biblioteca inválido")?;
    if file.formato != LIBRARY_FORMAT {
        anyhow::bail!(
            "não é um arquivo de biblioteca da RRS (formato '{}', esperado '{LIBRARY_FORMAT}')",
            file.formato
        );
    }
    let mut report = LibraryImport::default();
    let now = chrono::Utc::now().to_rfc3339();
    for item in &file.apps {
        let name = item.full_name.trim();
        if name.is_empty() {
            continue;
        }
        let playtime = item.playtime_seconds.min(i64::MAX as u64) as i64;
        let installed_here: bool = conn.query_row(
            "SELECT EXISTS (SELECT 1 FROM installed WHERE lower(full_name) = lower(?1))",
            [name],
            |r| r.get(0),
        )?;
        if installed_here {
            conn.execute(
                "UPDATE installed SET
                    playtime_seconds = MAX(playtime_seconds, ?2),
                    last_played = CASE WHEN ?3 IS NULL THEN last_played
                                       WHEN last_played IS NULL OR ?3 > last_played THEN ?3
                                       ELSE last_played END
                 WHERE lower(full_name) = lower(?1)",
                params![name, playtime, item.last_played],
            )?;
            report.merged += 1;
            continue;
        }
        let catalog_name: Option<String> = conn
            .query_row(
                "SELECT full_name FROM apps WHERE lower(full_name) = lower(?1)",
                [name],
                |r| r.get(0),
            )
            .ok();
        if catalog_name.is_none() {
            report.not_in_catalog.push(name.to_string());
        }
        conn.execute(
            "INSERT INTO library_wishlist (full_name, playtime_seconds, last_played, added_at)
             VALUES (?1, ?2, ?3, ?4)
             ON CONFLICT(full_name) DO UPDATE SET
                playtime_seconds = MAX(library_wishlist.playtime_seconds, excluded.playtime_seconds),
                last_played = CASE WHEN excluded.last_played IS NULL THEN library_wishlist.last_played
                                   WHEN library_wishlist.last_played IS NULL
                                        OR excluded.last_played > library_wishlist.last_played
                                   THEN excluded.last_played
                                   ELSE library_wishlist.last_played END",
            params![catalog_name.as_deref().unwrap_or(name), playtime, item.last_played, now],
        )?;
        report.to_reinstall += 1;
    }
    Ok(report)
}

/// Nomes na lista "Para reinstalar" (vindos de uma biblioteca importada).
pub fn wishlist(conn: &Connection) -> Result<Vec<String>> {
    let mut st = conn.prepare("SELECT full_name FROM library_wishlist ORDER BY full_name")?;
    let rows = st.query_map([], |r| r.get(0))?;
    Ok(rows.collect::<rusqlite::Result<_>>()?)
}

pub fn remove_from_wishlist(conn: &Connection, full_name: &str) -> Result<()> {
    conn.execute(
        "DELETE FROM library_wishlist WHERE lower(full_name) = lower(?1)",
        [full_name],
    )?;
    Ok(())
}

/// Exporta todo o catálogo para `catalog.json` (formatado para versionamento e curadoria semanal).
pub fn export_catalog(conn: &Connection, path: &Path) -> Result<usize> {
    let apps = list_all(conn)?;
    let entries: Vec<CatalogEntry> = apps.iter().map(|a| a.to_catalog_entry()).collect();
    let json = serde_json::to_string_pretty(&entries)?;
    std::fs::write(path, json)
        .with_context(|| format!("falha ao salvar catálogo em {}", path.display()))?;
    Ok(entries.len())
}

/// Catálogo embutido diretamente dentro do executável (.exe) para funcionar 100% standalone.
pub const EMBEDDED_CATALOG: &str = include_str!("../catalog.json");

/// Importa itens de `catalog.json` (ou do catálogo embutido no .exe) para o SQLite, preservando o registro de apps instalados.
pub fn import_catalog(conn: &Connection, path: &Path) -> Result<usize> {
    if !path.exists() {
        let _ = std::fs::write(path, EMBEDDED_CATALOG);
        return import_catalog_str(conn, EMBEDDED_CATALOG, "catálogo embutido");
    }
    let raw = std::fs::read_to_string(path)
        .with_context(|| format!("falha ao ler {}", path.display()))?;
    // Se o catálogo embutido tiver itens novos, garante que eles também entrem no banco
    let _ = import_catalog_str(conn, EMBEDDED_CATALOG, "catálogo embutido");
    import_catalog_str(conn, &raw, &path.display().to_string())
}

pub fn import_catalog_str(conn: &Connection, raw: &str, source_label: &str) -> Result<usize> {
    let clean = raw.trim_start_matches('\u{feff}');
    let entries: Vec<CatalogEntry> = serde_json::from_str(clean)
        .with_context(|| format!("JSON inválido em {source_label}"))?;
    for e in &entries {
        let filtered_assets: Vec<Asset> = e
            .assets
            .iter()
            .filter(|a| crate::github::platform_of(&a.name).is_some())
            .cloned()
            .collect();
        let assets = if filtered_assets.is_empty() {
            e.assets.clone()
        } else {
            filtered_assets
        };
        let mut plats: Vec<&str> = assets
            .iter()
            .filter_map(|a| crate::github::platform_of(&a.name))
            .collect();
        plats.sort();
        plats.dedup();
        let platforms = if plats.is_empty() {
            e.platforms.clone()
        } else {
            plats.join(",")
        };

        let mut app = App {
            full_name: e.full_name.clone(),
            description: e.description.clone(),
            url: e.url.clone(),
            stars: e.stars,
            license: e.license.clone(),
            topics: e.topics.clone(),
            language: e.language.clone(),
            created_at: e.created_at.clone(),
            pushed_at: e.pushed_at.clone(),
            fork: e.fork,
            archived: e.archived,
            has_issues: e.has_issues,
            latest_tag: e.latest_tag.clone(),
            released_at: e.released_at.clone(),
            platforms,
            assets,
            kind: e.kind.clone(),
            category: e.category.clone(),
            tags: e.tags.clone(),
            summary: e.summary.clone(),
            trust_score: e.trust_score,
            trust_level: e.trust_level.clone(),
            trust_notes: e.trust_notes.clone(),
            attestation: e.attestation.clone(),
            installed: None,
        };
        if app.trust_level.is_empty() || app.trust_notes.is_empty() {
            app.recompute_trust(None, &[]);
        }
        upsert(conn, &app)?;
    }
    Ok(entries.len())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn mem() -> Connection {
        open(":memory:").unwrap()
    }

    fn temp_dir(tag: &str) -> PathBuf {
        let nanos = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let dir = std::env::temp_dir().join(format!("rrs-teste-db-{tag}-{}-{nanos}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        dir
    }

    /// Item mínimo de catálogo em JSON.
    fn entry(name: &str, tag: &str, language: &str, attestation: &str) -> String {
        format!(
            r#"{{"full_name":"{name}","description":"d","url":"https://github.com/{name}","stars":10,
                "license":"MIT","topics":"","language":"{language}","pushed_at":"2026-10-01T00:00:00Z",
                "latest_tag":"{tag}","platforms":"windows","assets":[],"attestation":"{attestation}",
                "trust_level":"x","trust_notes":["x"]}}"#
        )
    }

    fn catalog(entries: &[String]) -> String {
        format!("[{}]", entries.join(","))
    }

    #[test]
    fn importar_catalogo_sem_linguagem_nao_apaga_a_linguagem() {
        let c = mem();
        import_catalog_str(&c, &catalog(&[entry("a/rusty", "v1", "Rust", "")]), "t").unwrap();
        import_catalog_str(&c, &catalog(&[entry("a/rusty", "v1", "", "")]), "t").unwrap();
        let app = get(&c, "a/rusty").unwrap().unwrap();
        assert_eq!(app.language, "Rust");
        assert!(app.is_rust());
    }

    #[test]
    fn atestado_e_zerado_quando_a_versao_muda() {
        let c = mem();
        import_catalog_str(&c, &catalog(&[entry("a/b", "v1", "C", "presente")]), "t").unwrap();
        // Mesma versão, catálogo sem atestado: mantém
        import_catalog_str(&c, &catalog(&[entry("a/b", "v1", "C", "")]), "t").unwrap();
        assert_eq!(get(&c, "a/b").unwrap().unwrap().attestation, "presente");
        // Versão nova: o atestado era do arquivo antigo
        import_catalog_str(&c, &catalog(&[entry("a/b", "v2", "C", "")]), "t").unwrap();
        assert_eq!(get(&c, "a/b").unwrap().unwrap().attestation, "");
    }

    #[test]
    fn biblioteca_exporta_importa_e_devolve_tempo_de_jogo_na_reinstalacao() {
        let dir = temp_dir("biblioteca");
        let file = dir.join(LIBRARY_FILE_NAME);

        let pc1 = mem();
        import_catalog_str(&pc1, &catalog(&[entry("dono/jogo", "v2", "C++", "")]), "t").unwrap();
        mark_installed(&pc1, "dono/jogo", "v2", "downloads/dono_jogo/jogo.exe", "abc", true).unwrap();
        record_playtime(&pc1, "dono/jogo", 3600).unwrap();
        assert_eq!(export_library(&pc1, &file).unwrap(), 1);

        let pc2 = mem();
        import_catalog_str(
            &pc2,
            &catalog(&[entry("dono/jogo", "v2", "C++", ""), entry("outro/app", "v1", "Go", "")]),
            "t",
        )
        .unwrap();
        let rep = import_library(&pc2, &file).unwrap();
        assert_eq!(rep.to_reinstall, 1);
        assert_eq!(rep.merged, 0);
        assert!(rep.not_in_catalog.is_empty());
        assert_eq!(wishlist(&pc2).unwrap(), vec!["dono/jogo".to_string()]);
        // Exportar de novo inclui o que ainda está na lista para reinstalar
        assert_eq!(library_items(&pc2).unwrap().len(), 1);

        mark_installed(&pc2, "dono/jogo", "v2", "downloads/dono_jogo/jogo.exe", "abc", true).unwrap();
        let inst = get(&pc2, "dono/jogo").unwrap().unwrap().installed.unwrap();
        assert_eq!(inst.playtime_seconds, 3600);
        assert!(inst.last_played.is_some());
        assert!(wishlist(&pc2).unwrap().is_empty());

        std::fs::remove_dir_all(&dir).unwrap();
    }

    #[test]
    fn biblioteca_mescla_tempo_de_jogo_de_app_ja_instalado() {
        let c = mem();
        import_catalog_str(&c, &catalog(&[entry("dono/jogo", "v1", "C", "")]), "t").unwrap();
        mark_installed(&c, "dono/jogo", "v1", "x.exe", "abc", false).unwrap();
        record_playtime(&c, "dono/jogo", 100).unwrap();
        let raw = r#"{"formato":"rrs-biblioteca","versao":1,"apps":[
            {"full_name":"DONO/JOGO","installed_tag":"v1","playtime_seconds":500,"last_played":"2030-01-01T00:00:00Z"},
            {"full_name":"ninguem/sumiu","playtime_seconds":0}]}"#;
        let rep = import_library_str(&c, raw).unwrap();
        assert_eq!(rep.merged, 1);
        assert_eq!(rep.to_reinstall, 1);
        assert_eq!(rep.not_in_catalog, vec!["ninguem/sumiu".to_string()]);
        let inst = get(&c, "dono/jogo").unwrap().unwrap().installed.unwrap();
        assert_eq!(inst.playtime_seconds, 500);
        assert_eq!(inst.last_played.as_deref(), Some("2030-01-01T00:00:00Z"));
    }

    #[test]
    fn biblioteca_recusa_arquivo_de_outro_formato() {
        let c = mem();
        assert!(import_library_str(&c, r#"{"formato":"outra-coisa","versao":1,"apps":[]}"#).is_err());
        assert!(import_library_str(&c, "[1,2,3]").is_err());
    }

    #[test]
    fn caminho_dentro_da_pasta_da_rrs_vira_relativo() {
        let base = Path::new("F:\\RRS");
        let stored = to_stored_path(base, Path::new("F:\\RRS\\downloads\\a_b\\app.exe"));
        assert_eq!(Path::new(&stored), Path::new("downloads\\a_b\\app.exe"));
        assert_eq!(
            resolve_stored_path(Path::new("E:\\Pendrive\\RRS"), &stored),
            Path::new("E:\\Pendrive\\RRS\\downloads\\a_b\\app.exe")
        );
        // Fora da pasta da RRS continua absoluto
        assert_eq!(to_stored_path(base, Path::new("C:\\Jogos\\x.exe")), "C:\\Jogos\\x.exe");
        assert_eq!(resolve_stored_path(base, "C:\\Jogos\\x.exe"), Path::new("C:\\Jogos\\x.exe"));
    }

    #[test]
    fn caminhos_antigos_sao_convertidos_e_corrigidos_ao_abrir() {
        let base = temp_dir("portatil");
        let exe = base.join("downloads").join("dono_jogo").join("jogo.exe");
        std::fs::create_dir_all(exe.parent().unwrap()).unwrap();
        std::fs::write(&exe, b"x").unwrap();

        let c = mem();
        import_catalog_str(
            &c,
            &catalog(&[entry("dono/jogo", "v1", "C", ""), entry("dono/movido", "v1", "C", ""), entry("dono/fora", "v1", "C", "")]),
            "t",
        )
        .unwrap();
        // 1) absoluto dentro da pasta atual; 2) mesma pasta em outra letra de unidade (não existe mais);
        // 3) absoluto fora da RRS que não existe e não tem downloads\ correspondente
        mark_installed(&c, "dono/jogo", "v1", &exe.to_string_lossy(), "a", true).unwrap();
        mark_installed(&c, "dono/movido", "v1", "Q:\\RRS-antiga\\downloads\\dono_jogo\\jogo.exe", "a", true).unwrap();
        mark_installed(&c, "dono/fora", "v1", "Q:\\Outro\\programa.exe", "a", true).unwrap();

        assert_eq!(relativize_installed_paths(&c, &base).unwrap(), 2);
        let path_of = |n: &str| get(&c, n).unwrap().unwrap().installed.unwrap().file_path;
        let rel = Path::new("downloads").join("dono_jogo").join("jogo.exe");
        assert_eq!(Path::new(&path_of("dono/jogo")), rel.as_path());
        assert_eq!(Path::new(&path_of("dono/movido")), rel.as_path());
        assert_eq!(path_of("dono/fora"), "Q:\\Outro\\programa.exe");
        assert_eq!(resolve_stored_path(&base, &path_of("dono/movido")), exe);

        std::fs::remove_dir_all(&base).unwrap();
    }

    #[test]
    fn catalogo_embutido_tem_linguagem_em_todos_os_itens() {
        let entries: Vec<CatalogEntry> =
            serde_json::from_str(EMBEDDED_CATALOG.trim_start_matches('\u{feff}')).unwrap();
        let sem: Vec<_> = entries.iter().filter(|e| e.language.is_empty()).map(|e| &e.full_name).collect();
        assert!(sem.is_empty(), "sem linguagem: {sem:?}");
        assert!(entries.iter().any(|e| crate::github::is_rust_language(&e.language)));
    }
}
