//! Jogos instalados por outras lojas (Steam, Epic Games, GOG) para aparecer na biblioteca da RRS.
//! A RRS só lê os arquivos de cada loja e pede para a própria loja abrir o jogo.

use anyhow::{bail, Context, Result};
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Store {
    Steam,
    Epic,
    Gog,
}

impl Store {
    pub fn label(self) -> &'static str {
        match self {
            Store::Steam => "Steam",
            Store::Epic => "Epic Games",
            Store::Gog => "GOG",
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

/// Todos os jogos das outras lojas, em ordem alfabética.
pub fn all_games() -> Vec<ExternalGame> {
    let mut all = steam_games();
    all.extend(epic_games());
    all.extend(gog_games());
    all.sort_by_key(|g| g.name.to_lowercase());
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
    fn so_aceita_enderecos_das_lojas() {
        assert!(is_allowed_store_uri("steam://rungameid/105600"));
        assert!(!is_allowed_store_uri("steam://rungameid/1&calc"));
        assert!(!is_allowed_store_uri("steam://install/1"));
        assert!(!is_allowed_store_uri("file:///C:/x.exe"));
        assert!(is_allowed_store_uri("com.epicgames.launcher://apps/a%3Ab%3Ac?action=launch&silent=true"));
        assert!(!is_allowed_store_uri("com.epicgames.launcher://apps/a b"));
    }
}
