//! Banco de dados SQLite local + sincronização com `catalog.json` e controle de instalados.

use crate::github::Asset;
use crate::verify;
use anyhow::{Context, Result};
use rusqlite::{params, Connection, Row};
use serde::{Deserialize, Serialize};
use std::path::Path;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CatalogEntry {
    pub full_name: String,
    pub description: String,
    pub url: String,
    pub stars: u64,
    pub license: String,
    pub topics: String,
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
    pub created_at: String,
    pub pushed_at: String,
    pub fork: bool,
    pub archived: bool,
    pub has_issues: bool,
    pub latest_tag: String,
    pub platforms: String,
    pub assets: Vec<Asset>,
    pub kind: Option<String>,
    pub category: Option<String>,
    pub tags: Option<String>,
    pub summary: Option<String>,
    pub trust_score: i32,
    pub trust_level: String,
    pub trust_notes: Vec<String>,
    pub installed: Option<InstalledInfo>,
}

impl App {
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
            created_at: self.created_at.clone(),
            pushed_at: self.pushed_at.clone(),
            fork: self.fork,
            archived: self.archived,
            has_issues: self.has_issues,
            latest_tag: self.latest_tag.clone(),
            platforms: self.platforms.clone(),
            assets: self.assets.clone(),
            kind: self.kind.clone(),
            category: self.category.clone(),
            tags: self.tags.clone(),
            summary: self.summary.clone(),
            trust_score: self.trust_score,
            trust_level: self.trust_level.clone(),
            trust_notes: self.trust_notes.clone(),
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
           i.last_played, i.playtime_seconds
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
        created_at: r.get(6)?,
        pushed_at: r.get(7)?,
        fork: r.get::<_, i64>(8)? != 0,
        archived: r.get::<_, i64>(9)? != 0,
        has_issues: r.get::<_, i64>(10)? != 0,
        latest_tag: r.get(11)?,
        platforms: r.get(12)?,
        assets,
        kind: r.get(14)?,
        category: r.get(15)?,
        tags: r.get(16)?,
        summary: r.get(17)?,
        trust_score: r.get(18)?,
        trust_level: r.get(19)?,
        trust_notes,
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
            trust_score, trust_level, trust_notes_json
         )
         VALUES (?1,?2,?3,?4,?5,?6,?7,?8,?9,?10,?11,?12,?13,?14,?15,?16,?17,?18,?19,?20,?21)
         ON CONFLICT(full_name) DO UPDATE SET
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
            created_at: e.created_at.clone(),
            pushed_at: e.pushed_at.clone(),
            fork: e.fork,
            archived: e.archived,
            has_issues: e.has_issues,
            latest_tag: e.latest_tag.clone(),
            platforms,
            assets,
            kind: e.kind.clone(),
            category: e.category.clone(),
            tags: e.tags.clone(),
            summary: e.summary.clone(),
            trust_score: e.trust_score,
            trust_level: e.trust_level.clone(),
            trust_notes: e.trust_notes.clone(),
            installed: None,
        };
        if app.trust_level.is_empty() || app.trust_notes.is_empty() {
            app.recompute_trust(None, &[]);
        }
        upsert(conn, &app)?;
    }
    Ok(entries.len())
}
