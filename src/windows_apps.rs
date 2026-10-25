//! Programas instalados no Windows (fora da RRS), lidos das chaves "Uninstall" do registro, e a
//! comparação com o catálogo para avisar quando existe versão nova.

use crate::db::App;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InstalledProgram {
    pub name: String,
    pub version: String,
    pub publisher: String,
}

/// Um programa do Windows reconhecido como um app do catálogo.
#[derive(Debug, Clone)]
pub struct CatalogMatch {
    pub full_name: String,
    pub program: InstalledProgram,
    /// `Some(true)` = a versão instalada é mais antiga que a do catálogo.
    pub outdated: Option<bool>,
}

const UNINSTALL_KEYS: &[&str] = &[
    r"HKLM\SOFTWARE\Microsoft\Windows\CurrentVersion\Uninstall",
    r"HKLM\SOFTWARE\WOW6432Node\Microsoft\Windows\CurrentVersion\Uninstall",
    r"HKCU\Software\Microsoft\Windows\CurrentVersion\Uninstall",
];

/// Lê os programas instalados (vazio fora do Windows ou se o `reg` falhar).
pub fn list_installed_programs() -> Vec<InstalledProgram> {
    let mut all = Vec::new();
    #[cfg(target_os = "windows")]
    for key in UNINSTALL_KEYS {
        if let Ok(out) = std::process::Command::new("reg").args(["query", key, "/s"]).output() {
            all.extend(parse_reg_query(&String::from_utf8_lossy(&out.stdout)));
        }
    }
    all.sort_by_key(|p| p.name.to_lowercase());
    all.dedup_by(|a, b| a.name.eq_ignore_ascii_case(&b.name) && a.version == b.version);
    all
}

/// Interpreta a saída de `reg query <chave> /s`: blocos que começam com o nome da chave e linhas
/// `    Nome    REG_SZ    Valor`.
pub fn parse_reg_query(output: &str) -> Vec<InstalledProgram> {
    let mut programs = Vec::new();
    let mut current: Option<InstalledProgram> = None;
    let flush = |cur: &mut Option<InstalledProgram>, out: &mut Vec<InstalledProgram>| {
        if let Some(p) = cur.take() {
            if !p.name.trim().is_empty() {
                out.push(p);
            }
        }
    };
    for raw in output.lines() {
        let line = raw.trim_end_matches('\r');
        if line.starts_with("HKEY_") {
            flush(&mut current, &mut programs);
            current = Some(InstalledProgram {
                name: String::new(),
                version: String::new(),
                publisher: String::new(),
            });
            continue;
        }
        let Some(p) = current.as_mut() else { continue };
        let trimmed = line.trim_start();
        let mut parts = trimmed.splitn(3, "    ");
        let (Some(name), Some(kind), value) = (parts.next(), parts.next(), parts.next()) else {
            continue;
        };
        if !kind.starts_with("REG_") {
            continue;
        }
        let value = value.unwrap_or("").trim().to_string();
        match name {
            "DisplayName" => p.name = value,
            "DisplayVersion" => p.version = value,
            "Publisher" => p.publisher = value,
            _ => {}
        }
    }
    flush(&mut current, &mut programs);
    programs
}

fn alnum_lower(s: &str) -> String {
    s.chars().filter(|c| c.is_alphanumeric()).collect::<String>().to_lowercase()
}

/// Números de uma versão: "v1.2.3-beta" → [1, 2, 3]. Vazio se não houver números.
pub fn version_numbers(v: &str) -> Vec<u64> {
    let start = v.find(|c: char| c.is_ascii_digit()).unwrap_or(v.len());
    v[start..]
        .split(|c: char| !c.is_ascii_digit())
        .take_while(|s| !s.is_empty())
        .filter_map(|s| s.parse().ok())
        .collect()
}

/// `Some(true)` se `installed` < `latest`; `None` quando não dá para comparar (ex.: tags por data).
pub fn is_older(installed: &str, latest: &str) -> Option<bool> {
    let (a, b) = (version_numbers(installed), version_numbers(latest));
    if a.is_empty() || b.is_empty() || b[0] > 1000 {
        return None;
    }
    let n = a.len().max(b.len());
    let pad = |v: &Vec<u64>| (0..n).map(|i| v.get(i).copied().unwrap_or(0)).collect::<Vec<_>>();
    Some(pad(&a) < pad(&b))
}

/// Reconhece programas do Windows como apps do catálogo pelo nome do repositório
/// (ex.: "OBS Studio" ↔ obsproject/obs-studio). Nomes curtos demais são ignorados para evitar engano.
pub fn match_catalog(apps: &[App], programs: &[InstalledProgram]) -> Vec<CatalogMatch> {
    let mut found = Vec::new();
    for app in apps {
        if app.installed.is_some() {
            continue;
        }
        let repo = app.full_name.rsplit('/').next().unwrap_or(&app.full_name);
        let key = alnum_lower(repo);
        if key.len() < 4 {
            continue;
        }
        let hit = programs.iter().find(|p| {
            let name = alnum_lower(&p.name);
            name.starts_with(&key) || (key.len() >= 6 && name.contains(&key))
        });
        if let Some(p) = hit {
            found.push(CatalogMatch {
                full_name: app.full_name.clone(),
                program: p.clone(),
                outdated: is_older(&p.version, &app.latest_tag),
            });
        }
    }
    found
}

#[cfg(test)]
mod tests {
    use super::*;

    const REG: &str = "\r\nHKEY_LOCAL_MACHINE\\SOFTWARE\\Microsoft\\Windows\\CurrentVersion\\Uninstall\\OBS Studio\r\n    DisplayName    REG_SZ    OBS Studio\r\n    DisplayVersion    REG_SZ    31.0.2\r\n    Publisher    REG_SZ    OBS Project\r\n    NoRepair    REG_DWORD    0x1\r\n\r\nHKEY_LOCAL_MACHINE\\SOFTWARE\\Microsoft\\Windows\\CurrentVersion\\Uninstall\\Vazio\r\n\r\nHKEY_LOCAL_MACHINE\\SOFTWARE\\Microsoft\\Windows\\CurrentVersion\\Uninstall\\Steam App 105600\r\n    DisplayName    REG_SZ    Terraria\r\n    InstallLocation    REG_SZ    \r\n";

    #[test]
    fn le_a_saida_do_reg_query() {
        let p = parse_reg_query(REG);
        assert_eq!(p.len(), 2, "{p:?}");
        assert_eq!(p[0].name, "OBS Studio");
        assert_eq!(p[0].version, "31.0.2");
        assert_eq!(p[0].publisher, "OBS Project");
        assert_eq!(p[1].name, "Terraria");
        assert_eq!(p[1].version, "");
    }

    #[test]
    fn compara_versoes() {
        assert_eq!(version_numbers("v1.17.0-rc1"), vec![1, 17, 0]);
        assert_eq!(is_older("31.0.2", "32.0.1"), Some(true));
        assert_eq!(is_older("v2.4.1", "2.4.1"), Some(false));
        assert_eq!(is_older("2.4", "2.4.0"), Some(false));
        assert_eq!(is_older("1.0", "20240203-110809-5046fc22"), None, "tag por data não é comparável");
        assert_eq!(is_older("", "1.0"), None);
    }

    #[test]
    fn reconhece_programa_do_catalogo() {
        let conn = crate::db::open(":memory:").unwrap();
        crate::db::import_catalog_str(
            &conn,
            r#"[{"full_name":"obsproject/obs-studio","description":"","url":"","stars":1,"license":"","topics":"",
                 "pushed_at":"","latest_tag":"32.0.1","platforms":"windows","assets":[],"trust_level":"x","trust_notes":["x"]},
                {"full_name":"x/go","description":"","url":"","stars":1,"license":"","topics":"",
                 "pushed_at":"","latest_tag":"1.0","platforms":"windows","assets":[],"trust_level":"x","trust_notes":["x"]}]"#,
            "t",
        )
        .unwrap();
        let apps = crate::db::list_all(&conn).unwrap();
        let progs = vec![
            InstalledProgram { name: "OBS Studio".into(), version: "31.0.2".into(), publisher: String::new() },
            InstalledProgram { name: "Google Chrome".into(), version: "1".into(), publisher: String::new() },
        ];
        let m = match_catalog(&apps, &progs);
        assert_eq!(m.len(), 1, "\"go\" é curto demais para casar com \"Google Chrome\"");
        assert_eq!(m[0].full_name, "obsproject/obs-studio");
        assert_eq!(m[0].outdated, Some(true));
    }
}
