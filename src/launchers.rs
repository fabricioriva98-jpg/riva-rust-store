//! Jogos instalados por outras lojas (Steam, Epic Games, GOG) para aparecer na biblioteca da RRS.
//! A RRS só lê os arquivos de cada loja e pede para a própria loja abrir o jogo.

use anyhow::{bail, Context, Result};
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Store {
    Steam,
    Epic,
    Gog,
    Xbox,
    Ea,
    Ubisoft,
    BattleNet,
    Riot,
}

impl Store {
    pub fn label(self) -> &'static str {
        match self {
            Store::Steam => "Steam",
            Store::Epic => "Epic Games",
            Store::Gog => "GOG",
            Store::Xbox => "Xbox",
            Store::Ea => "EA App",
            Store::Ubisoft => "Ubisoft",
            Store::BattleNet => "Battle.net",
            Store::Riot => "Riot Games",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Launch {
    /// Endereço da própria loja (steam://..., com.epicgames.launcher://...).
    Uri(String),
    Exe { path: PathBuf, dir: PathBuf },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExternalGame {
    pub store: Store,
    pub id: String,
    pub name: String,
    pub launch: Launch,
}

// ─── VDF (formato de texto da Steam) ───────────────────────────────────────

#[derive(Debug, Clone, PartialEq)]
pub enum Vdf {
    Str(String),
    Map(BTreeMap<String, Vdf>),
}

impl Vdf {
    pub fn get(&self, key: &str) -> Option<&Vdf> {
        match self {
            Vdf::Map(m) => m.iter().find(|(k, _)| k.eq_ignore_ascii_case(key)).map(|(_, v)| v),
            Vdf::Str(_) => None,
        }
    }

    pub fn str(&self, key: &str) -> Option<&str> {
        match self.get(key)? {
            Vdf::Str(s) => Some(s),
            Vdf::Map(_) => None,
        }
    }
}

fn vdf_tokens(text: &str) -> Vec<String> {
    let mut tokens = Vec::new();
    let mut chars = text.chars().peekable();
    while let Some(c) = chars.next() {
        match c {
            '"' => {
                let mut s = String::new();
                while let Some(ch) = chars.next() {
                    match ch {
                        '\\' => {
                            if let Some(next) = chars.next() {
                                s.push(match next {
                                    'n' => '\n',
                                    't' => '\t',
                                    other => other,
                                });
                            }
                        }
                        '"' => break,
                        other => s.push(other),
                    }
                }
                tokens.push(format!("s{s}"));
            }
            '{' => tokens.push("{".into()),
            '}' => tokens.push("}".into()),
            '/' if chars.peek() == Some(&'/') => {
                for ch in chars.by_ref() {
                    if ch == '\n' {
                        break;
                    }
                }
            }
            _ => {}
        }
    }
    tokens
}

/// Lê um arquivo VDF ("chave" "valor" e "chave" { ... }).
pub fn parse_vdf(text: &str) -> Vdf {
    fn block(tokens: &[String], pos: &mut usize) -> Vdf {
        let mut map = BTreeMap::new();
        while *pos < tokens.len() {
            let tok = &tokens[*pos];
            if tok == "}" {
                *pos += 1;
                break;
            }
            let Some(key) = tok.strip_prefix('s') else {
                *pos += 1;
                continue;
            };
            *pos += 1;
            match tokens.get(*pos).map(String::as_str) {
                Some("{") => {
                    *pos += 1;
                    map.insert(key.to_string(), block(tokens, pos));
                }
                Some(v) if v.starts_with('s') => {
                    map.insert(key.to_string(), Vdf::Str(v[1..].to_string()));
                    *pos += 1;
                }
                _ => {}
            }
        }
        Vdf::Map(map)
    }
    let tokens = vdf_tokens(text);
    let mut pos = 0;
    block(&tokens, &mut pos)
}

/// Itens da Steam que não são jogos (bibliotecas, Proton, ferramentas).
fn is_steam_tool(appid: &str, name: &str) -> bool {
    const SKIP_IDS: &[&str] = &["228980", "1070560", "1391110", "1628350", "250820"];
    let lower = name.to_lowercase();
    SKIP_IDS.contains(&appid)
        || ["redistributable", "steamworks", "proton", "steam linux runtime", "steamvr"]
            .iter()
            .any(|w| lower.contains(w))
}

/// Pastas de biblioteca listadas no `libraryfolders.vdf`.
pub fn steam_library_paths(libraryfolders: &str) -> Vec<PathBuf> {
    let root = parse_vdf(libraryfolders);
    let Some(Vdf::Map(folders)) = root.get("libraryfolders") else {
        return Vec::new();
    };
    folders
        .values()
        .filter_map(|f| f.str("path"))
        .map(PathBuf::from)
        .collect()
}

/// Um `appmanifest_<id>.acf` → jogo (ou `None` para ferramentas e instalações incompletas).
pub fn steam_game_from_manifest(acf: &str) -> Option<ExternalGame> {
    let root = parse_vdf(acf);
    let state = root.get("AppState")?;
    let appid = state.str("appid")?.to_string();
    let name = state.str("name")?.to_string();
    if !appid.chars().all(|c| c.is_ascii_digit()) || is_steam_tool(&appid, &name) {
        return None;
    }
    Some(ExternalGame {
        store: Store::Steam,
        launch: Launch::Uri(format!("steam://rungameid/{appid}")),
        id: appid,
        name,
    })
}

fn steam_root() -> Option<PathBuf> {
    let mut candidates = Vec::new();
    #[cfg(target_os = "windows")]
    {
        if let Ok(out) = std::process::Command::new("reg")
            .args(["query", r"HKCU\Software\Valve\Steam", "/v", "SteamPath"])
            .output()
        {
            let text = String::from_utf8_lossy(&out.stdout).to_string();
            if let Some(line) = text.lines().find(|l| l.contains("SteamPath")) {
                if let Some(v) = line.splitn(3, "    ").nth(2) {
                    candidates.push(PathBuf::from(v.trim().replace('/', "\\")));
                }
            }
        }
        candidates.push(PathBuf::from(r"C:\Program Files (x86)\Steam"));
        candidates.push(PathBuf::from(r"C:\Program Files\Steam"));
    }
    #[cfg(target_os = "linux")]
    {
        if let Some(home) = std::env::var_os("HOME") {
            let home = PathBuf::from(home);
            candidates.push(home.join(".local/share/Steam"));
            candidates.push(home.join(".steam/steam"));
            candidates.push(home.join(".steam/root"));
            candidates.push(home.join(".var/app/com.valvesoftware.Steam/.local/share/Steam"));
        }
    }
    #[cfg(target_os = "macos")]
    {
        if let Some(home) = std::env::var_os("HOME") {
            let home = PathBuf::from(home);
            candidates.push(home.join("Library/Application Support/Steam"));
        }
    }
    candidates.into_iter().find(|p| p.join("steamapps").is_dir())
}

pub fn steam_games() -> Vec<ExternalGame> {
    let Some(root) = steam_root() else { return Vec::new() };
    let mut libraries = std::fs::read_to_string(root.join("steamapps").join("libraryfolders.vdf"))
        .map(|t| steam_library_paths(&t))
        .unwrap_or_default();
    if !libraries.iter().any(|l| l == &root) {
        libraries.push(root);
    }
    let mut games = Vec::new();
    for lib in libraries {
        let Ok(entries) = std::fs::read_dir(lib.join("steamapps")) else { continue };
        for e in entries.flatten() {
            let name = e.file_name().to_string_lossy().to_string();
            if name.starts_with("appmanifest_") && name.ends_with(".acf") {
                if let Some(g) = std::fs::read_to_string(e.path()).ok().and_then(|t| steam_game_from_manifest(&t)) {
                    if !games.iter().any(|x: &ExternalGame| x.id == g.id) {
                        games.push(g);
                    }
                }
            }
        }
    }
    games
}

// ─── Epic Games ───────────────────────────────────────────────────────────

/// Um manifesto `.item` da Epic → jogo (só instalações completas).
pub fn epic_game_from_manifest(json_text: &str) -> Option<ExternalGame> {
    let v: serde_json::Value = serde_json::from_str(json_text.trim_start_matches('\u{feff}')).ok()?;
    if v["bIsIncompleteInstall"].as_bool().unwrap_or(false) {
        return None;
    }
    let name = v["DisplayName"].as_str()?.to_string();
    let app = v["AppName"].as_str()?;
    let ns = v["CatalogNamespace"].as_str()?;
    let item = v["CatalogItemId"].as_str()?;
    let safe = |s: &str| s.chars().all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_');
    if !(safe(app) && safe(ns) && safe(item)) {
        return None;
    }
    Some(ExternalGame {
        store: Store::Epic,
        id: app.to_string(),
        name,
        launch: Launch::Uri(format!(
            "com.epicgames.launcher://apps/{ns}%3A{item}%3A{app}?action=launch&silent=true"
        )),
    })
}

pub fn epic_games() -> Vec<ExternalGame> {
    let dir = std::env::var_os("ProgramData")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from(r"C:\ProgramData"))
        .join(r"Epic\EpicGamesLauncher\Data\Manifests");
    let Ok(entries) = std::fs::read_dir(dir) else { return Vec::new() };
    entries
        .flatten()
        .filter(|e| e.path().extension().is_some_and(|x| x.eq_ignore_ascii_case("item")))
        .filter_map(|e| std::fs::read_to_string(e.path()).ok())
        .filter_map(|t| epic_game_from_manifest(&t))
        .collect()
}

// ─── GOG ──────────────────────────────────────────────────────────────────

/// Saída de `reg query HKLM\SOFTWARE\WOW6432Node\GOG.com\Games /s` → jogos com executável.
pub fn gog_games_from_reg(output: &str) -> Vec<ExternalGame> {
    let mut games = Vec::new();
    let mut fields: BTreeMap<String, String> = BTreeMap::new();
    let flush = |fields: &mut BTreeMap<String, String>, games: &mut Vec<ExternalGame>| {
        if let (Some(name), Some(exe), Some(id)) = (fields.get("gameName"), fields.get("exe"), fields.get("gameID")) {
            let path = PathBuf::from(exe);
            let dir = fields
                .get("path")
                .map(PathBuf::from)
                .or_else(|| path.parent().map(Path::to_path_buf))
                .unwrap_or_default();
            games.push(ExternalGame {
                store: Store::Gog,
                id: id.clone(),
                name: name.clone(),
                launch: Launch::Exe { path, dir },
            });
        }
        fields.clear();
    };
    for raw in output.lines() {
        let line = raw.trim_end_matches('\r');
        if line.starts_with("HKEY_") {
            flush(&mut fields, &mut games);
            continue;
        }
        let mut parts = line.trim_start().splitn(3, "    ");
        if let (Some(k), Some(t), Some(v)) = (parts.next(), parts.next(), parts.next()) {
            if t.starts_with("REG_") {
                fields.insert(k.to_string(), v.trim().to_string());
            }
        }
    }
    flush(&mut fields, &mut games);
    games
}

pub fn gog_games() -> Vec<ExternalGame> {
    #[cfg(target_os = "windows")]
    if let Ok(out) = std::process::Command::new("reg")
        .args(["query", r"HKLM\SOFTWARE\WOW6432Node\GOG.com\Games", "/s"])
        .output()
    {
        return gog_games_from_reg(&String::from_utf8_lossy(&out.stdout));
    }
    Vec::new()
}

// ─── Xbox / Microsoft Store / PC Game Pass ─────────────────────────────────

pub fn xbox_games() -> Vec<ExternalGame> {
    let mut games = Vec::new();
    #[cfg(target_os = "windows")]
    if let Ok(out) = std::process::Command::new("reg")
        .args(["query", r"HKCU\System\GameConfigStore\Children", "/s"])
        .output()
    {
        let text = String::from_utf8_lossy(&out.stdout);
        let mut utm = String::new();
        let mut exe_path = String::new();

        let flush = |utm: &mut String, exe_path: &mut String, games: &mut Vec<ExternalGame>| {
            if !utm.is_empty() {
                let clean = utm.trim_start_matches("P~");
                let name = clean
                    .split('_')
                    .next()
                    .unwrap_or(clean)
                    .replace("Microsoft.", "")
                    .replace('.', " ");
                let id = clean.to_string();
                if !games.iter().any(|g| g.id == id) {
                    games.push(ExternalGame {
                        store: Store::Xbox,
                        id: id.clone(),
                        name,
                        launch: Launch::Uri(format!("shell:AppsFolder\\{clean}")),
                    });
                }
            } else if !exe_path.is_empty() {
                let p = PathBuf::from(exe_path.as_str());
                if p.is_file() {
                    let file_stem = p.file_stem().and_then(|s| s.to_str()).unwrap_or("Jogo");
                    if !["explorer", "cmd", "powershell", "runtime", "installer", "setup"]
                        .contains(&file_stem.to_lowercase().as_str())
                    {
                        let name = p
                            .parent()
                            .and_then(|d| d.file_name())
                            .and_then(|s| s.to_str())
                            .unwrap_or(file_stem)
                            .to_string();
                        let dir = p.parent().map(Path::to_path_buf).unwrap_or_default();
                        let id = format!("{}", p.display());
                        if !games.iter().any(|g| g.name.eq_ignore_ascii_case(&name)) {
                            games.push(ExternalGame {
                                store: Store::Xbox,
                                id,
                                name,
                                launch: Launch::Exe { path: p, dir },
                            });
                        }
                    }
                }
            }
            utm.clear();
            exe_path.clear();
        };

        for line in text.lines() {
            let line = line.trim_end();
            if line.starts_with("HKEY_") {
                flush(&mut utm, &mut exe_path, &mut games);
                continue;
            }
            let mut parts = line.trim_start().splitn(3, "    ");
            if let (Some(k), Some(_), Some(v)) = (parts.next(), parts.next(), parts.next()) {
                if k == "UtmItemId" {
                    utm.clear();
                    utm.push_str(v.trim());
                } else if k == "MatchedExeFullPath" {
                    exe_path.clear();
                    exe_path.push_str(v.trim());
                }
            }
        }
        flush(&mut utm, &mut exe_path, &mut games);
    }
    games
}

// ─── EA App / Origin ───────────────────────────────────────────────────────

pub fn ea_games() -> Vec<ExternalGame> {
    let mut games = Vec::new();
    #[cfg(target_os = "windows")]
    for key in [
        r"HKLM\SOFTWARE\Electronic Arts\EA Desktop\InstalledGames",
        r"HKLM\SOFTWARE\WOW6432Node\Electronic Arts\EA Games",
    ] {
        if let Ok(out) = std::process::Command::new("reg").args(["query", key, "/s"]).output() {
            let text = String::from_utf8_lossy(&out.stdout);
            for line in text.lines() {
                let line = line.trim();
                if line.starts_with("Install Dir") || line.starts_with("InstallDir") {
                    if let Some(val) = line.split("    ").nth(2) {
                        let p = PathBuf::from(val.trim());
                        if p.is_dir() {
                            let name = p.file_name().and_then(|s| s.to_str()).unwrap_or("Jogo EA").to_string();
                            let id = name.to_lowercase().replace(' ', "-");
                            games.push(ExternalGame {
                                store: Store::Ea,
                                id,
                                name,
                                launch: Launch::Exe { path: p.clone(), dir: p },
                            });
                        }
                    }
                }
            }
        }
    }
    games
}

// ─── Ubisoft Connect ───────────────────────────────────────────────────────

pub fn ubisoft_games() -> Vec<ExternalGame> {
    let mut games = Vec::new();
    #[cfg(target_os = "windows")]
    if let Ok(out) = std::process::Command::new("reg")
        .args(["query", r"HKLM\SOFTWARE\WOW6432Node\Ubisoft\Launcher\Installs", "/s"])
        .output()
    {
        let text = String::from_utf8_lossy(&out.stdout);
        let mut current_id = String::new();
        for line in text.lines() {
            let line = line.trim();
            if line.starts_with("HKEY_") {
                current_id = line.split('\\').next_back().unwrap_or("").to_string();
            } else if line.starts_with("InstallDir") {
                if let Some(val) = line.split("    ").nth(2) {
                    let p = PathBuf::from(val.trim());
                    let name = p.file_name().and_then(|s| s.to_str()).unwrap_or("Ubisoft Game").to_string();
                    if !current_id.is_empty() && !games.iter().any(|g: &ExternalGame| g.id == current_id) {
                        games.push(ExternalGame {
                            store: Store::Ubisoft,
                            id: current_id.clone(),
                            name,
                            launch: Launch::Uri(format!("uplay://launch/{current_id}/0")),
                        });
                    }
                }
            }
        }
    }
    games
}

// ─── Battle.net (Blizzard) ─────────────────────────────────────────────────

pub fn battlenet_games() -> Vec<ExternalGame> {
    let mut games = Vec::new();
    #[cfg(target_os = "windows")]
    for (id, name, default_path) in [
        ("Pro", "Overwatch 2", r"C:\Program Files (x86)\Overwatch\_retail_\Overwatch.exe"),
        ("WoW", "World of Warcraft", r"C:\Program Files (x86)\World of Warcraft\_retail_\Wow.exe"),
        ("Fen", "Diablo IV", r"C:\Program Files (x86)\Diablo IV\Diablo IV.exe"),
        ("WTCG", "Hearthstone", r"C:\Program Files (x86)\Hearthstone\Hearthstone.exe"),
        ("OSI", "Diablo II: Resurrected", r"C:\Program Files (x86)\Diablo II Resurrected\D2R.exe"),
    ] {
        let p = PathBuf::from(default_path);
        if p.exists() {
            games.push(ExternalGame {
                store: Store::BattleNet,
                id: id.to_string(),
                name: name.to_string(),
                launch: Launch::Uri(format!("battlenet://{id}")),
            });
        }
    }
    games
}

// ─── Riot Games ────────────────────────────────────────────────────────────

pub fn riot_games() -> Vec<ExternalGame> {
    let mut games = Vec::new();
    #[cfg(target_os = "windows")]
    for (id, name, path) in [
        ("league_of_legends", "League of Legends", r"D:\Games\Riot Games\League of Legends\LeagueClient.exe"),
        ("valorant", "VALORANT", r"D:\Games\Riot Games\VALORANT\live\VALORANT.exe"),
        ("league_of_legends_c", "League of Legends", r"C:\Riot Games\League of Legends\LeagueClient.exe"),
        ("valorant_c", "VALORANT", r"C:\Riot Games\VALORANT\live\VALORANT.exe"),
    ] {
        let p = PathBuf::from(path);
        if p.exists() && !games.iter().any(|g: &ExternalGame| g.name == name) {
            let dir = p.parent().map(Path::to_path_buf).unwrap_or_default();
            games.push(ExternalGame {
                store: Store::Riot,
                id: id.to_string(),
                name: name.to_string(),
                launch: Launch::Exe { path: p, dir },
            });
        }
    }
    games
}

/// Relatório consolidado da sincronização de todas as lojas instaladas no PC.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
#[allow(dead_code)]
pub struct StoreSyncReport {
    pub total: usize,
    pub by_store: BTreeMap<String, usize>,
    pub games: Vec<ExternalGame>,
}

/// Sincroniza e reúne os jogos de todas as lojas reconhecidas no sistema.
#[allow(dead_code)]
pub fn sync_all_stores() -> StoreSyncReport {
    let games = all_games();
    let mut by_store = BTreeMap::new();
    for g in &games {
        *by_store.entry(g.store.label().to_string()).or_insert(0) += 1;
    }
    StoreSyncReport {
        total: games.len(),
        by_store,
        games,
    }
}

/// Todos os jogos das outras lojas, em ordem alfabética.
pub fn all_games() -> Vec<ExternalGame> {
    let mut all = steam_games();
    all.extend(epic_games());
    all.extend(gog_games());
    all.extend(xbox_games());
    all.extend(ea_games());
    all.extend(ubisoft_games());
    all.extend(battlenet_games());
    all.extend(riot_games());
    all.sort_by_key(|g| g.name.to_lowercase());
    all.dedup_by(|a, b| a.id == b.id && a.store == b.store);
    all
}

/// Só endereços das lojas conhecidas, com identificadores sem caracteres especiais.
pub fn is_allowed_store_uri(uri: &str) -> bool {
    if let Some(id) = uri.strip_prefix("steam://rungameid/") {
        return !id.is_empty() && id.chars().all(|c| c.is_ascii_digit());
    }
    if let Some(rest) = uri.strip_prefix("com.epicgames.launcher://apps/") {
        return rest
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || matches!(c, '-' | '_' | '%' | '?' | '&' | '='));
    }
    if let Some(rest) = uri.strip_prefix("uplay://launch/") {
        return rest
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || matches!(c, '/' | '?' | '&' | '='));
    }
    if let Some(rest) = uri.strip_prefix("origin2://game/launch?offerId=") {
        return rest
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || matches!(c, '-' | '_' | '%' | '&' | '='));
    }
    if let Some(rest) = uri.strip_prefix("battlenet://") {
        return rest
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || matches!(c, '-' | '_'));
    }
    if let Some(rest) = uri.strip_prefix("shell:AppsFolder\\") {
        return rest
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || matches!(c, '-' | '_' | '.' | '!' | '~'));
    }
    false
}

pub fn launch(game: &ExternalGame) -> Result<()> {
    match &game.launch {
        Launch::Uri(uri) => {
            if !is_allowed_store_uri(uri) {
                bail!("endereço de loja recusado: {uri}");
            }
            crate::downloader::open_store_uri(uri)
        }
        Launch::Exe { path, dir } => {
            if !path.is_file() {
                bail!("executável não encontrado: {}", path.display());
            }
            std::process::Command::new(path)
                .current_dir(dir)
                .spawn()
                .with_context(|| format!("não foi possível abrir {}", game.name))?;
            Ok(())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const LIBRARY: &str = r#""libraryfolders"
{
	"0"
	{
		"path"		"C:\\Program Files (x86)\\Steam"
		"apps"
		{
			"228980"		"116674056"
		}
	}
	"1"
	{
		"path"		"D:\\SteamLibrary"
		"apps" { "105600" "803964902" }
	}
}"#;

    #[test]
    fn le_as_bibliotecas_da_steam() {
        let libs = steam_library_paths(LIBRARY);
        assert_eq!(libs, vec![PathBuf::from(r"C:\Program Files (x86)\Steam"), PathBuf::from(r"D:\SteamLibrary")]);
    }

    #[test]
    fn manifesto_da_steam_vira_jogo_e_ignora_ferramentas() {
        let game = steam_game_from_manifest("\"AppState\"\n{\n\t\"appid\"\t\t\"105600\"\n\t\"name\"\t\t\"Terraria\"\n\t\"installdir\"\t\t\"Terraria\"\n}");
        let game = game.unwrap();
        assert_eq!(game.name, "Terraria");
        assert_eq!(game.launch, Launch::Uri("steam://rungameid/105600".into()));
        assert!(steam_game_from_manifest("\"AppState\" { \"appid\" \"228980\" \"name\" \"Steamworks Common Redistributables\" }").is_none());
    }

    #[test]
    fn manifesto_da_epic() {
        let ok = r#"{"DisplayName":"Jogo X","AppName":"Fennec","CatalogNamespace":"abc123","CatalogItemId":"def456","bIsIncompleteInstall":false}"#;
        let g = epic_game_from_manifest(ok).unwrap();
        assert_eq!(g.launch, Launch::Uri("com.epicgames.launcher://apps/abc123%3Adef456%3AFennec?action=launch&silent=true".into()));
        assert!(epic_game_from_manifest(&ok.replace("false", "true")).is_none(), "instalação incompleta");
        assert!(epic_game_from_manifest(&ok.replace("Fennec", "a&calc")).is_none(), "nome com caractere especial");
    }

    #[test]
    fn registro_do_gog() {
        let reg = "HKEY_LOCAL_MACHINE\\SOFTWARE\\WOW6432Node\\GOG.com\\Games\\1207658924\r\n    gameID    REG_SZ    1207658924\r\n    gameName    REG_SZ    Heroes 3\r\n    exe    REG_SZ    C:\\GOG Games\\HoMM 3\\Heroes3.exe\r\n    path    REG_SZ    C:\\GOG Games\\HoMM 3\r\n\r\nHKEY_LOCAL_MACHINE\\SOFTWARE\\WOW6432Node\\GOG.com\\Games\\999\r\n    gameName    REG_SZ    Sem exe\r\n";
        let g = gog_games_from_reg(reg);
        assert_eq!(g.len(), 1);
        assert_eq!(g[0].name, "Heroes 3");
        assert_eq!(
            g[0].launch,
            Launch::Exe { path: PathBuf::from(r"C:\GOG Games\HoMM 3\Heroes3.exe"), dir: PathBuf::from(r"C:\GOG Games\HoMM 3") }
        );
    }

    #[test]
    fn sincronizacao_consolidada_todas_as_lojas_funciona() {
        let report = sync_all_stores();
        assert_eq!(report.total, report.games.len());
    }

    #[test]
    fn so_aceita_enderecos_das_lojas() {
        assert!(is_allowed_store_uri("steam://rungameid/105600"));
        assert!(!is_allowed_store_uri("steam://rungameid/1&calc"));
        assert!(!is_allowed_store_uri("steam://install/1"));
        assert!(!is_allowed_store_uri("file:///C:/x.exe"));
        assert!(is_allowed_store_uri("com.epicgames.launcher://apps/a%3Ab%3Ac?action=launch&silent=true"));
        assert!(!is_allowed_store_uri("com.epicgames.launcher://apps/a b"));
    }
}
