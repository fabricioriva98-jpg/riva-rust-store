//! Conquistas, XP e níveis. Tudo é calculado localmente a partir de contadores (`stats`) e do que está
//! instalado; nada sai do PC, a não ser que o usuário publique o perfil (ver `social.rs`).

use crate::db;
use anyhow::Result;
use rusqlite::Connection;
use std::collections::{HashMap, HashSet};

// Contadores (tabela `stats`)
pub const STAT_DOWNLOADS: &str = "downloads";
pub const STAT_VERIFIES: &str = "verificacoes";
pub const STAT_VT: &str = "virustotal";
pub const STAT_DEFENDER: &str = "defender";
pub const STAT_TRANSLATIONS: &str = "traducoes";
pub const STAT_AI_OK: &str = "traducoes_ia_ok";
pub const STAT_LIBRARY_EXPORT: &str = "biblioteca_exportada";
pub const STAT_WINDOWS_SCAN: &str = "busca_windows";
pub const STAT_TV: &str = "modo_tv";
pub const STAT_CLI: &str = "terminal";
pub const STAT_SAVES: &str = "backups_saves";
pub const STAT_OLD_VERSION: &str = "versao_anterior";
pub const STAT_PROFILE_PUBLISHED: &str = "perfil_publicado";
pub const STAT_SHARES: &str = "compartilhamentos";
pub const STAT_COMMUNITY: &str = "comunidade";
pub const STAT_LEARN_RUN: &str = "aprendeu_rust";

/// XP por evento.
const STAT_XP: &[(&str, u64)] = &[
    (STAT_DOWNLOADS, 50),
    (STAT_VERIFIES, 10),
    (STAT_VT, 20),
    (STAT_DEFENDER, 15),
    (STAT_TRANSLATIONS, 10),
    (STAT_AI_OK, 40),
    (STAT_LIBRARY_EXPORT, 30),
    (STAT_WINDOWS_SCAN, 20),
    (STAT_TV, 20),
    (STAT_CLI, 40),
    (STAT_SAVES, 25),
    (STAT_OLD_VERSION, 30),
    (STAT_PROFILE_PUBLISHED, 50),
    (STAT_SHARES, 15),
    (STAT_COMMUNITY, 10),
    (STAT_LEARN_RUN, 25),
];
const XP_PER_HOUR_PLAYED: u64 = 30;

pub struct Achievement {
    pub id: &'static str,
    pub icon: &'static str,
    pub name: &'static str,
    pub desc: &'static str,
    pub xp: u64,
}

pub const ACHIEVEMENTS: &[Achievement] = &[
    Achievement { id: "primeiro_download", icon: "⬇", name: "Primeiro download", desc: "Baixe e instale o primeiro app pela RRS.", xp: 50 },
    Achievement { id: "colecionador", icon: "📦", name: "Colecionador", desc: "Tenha 10 apps instalados.", xp: 150 },
    Achievement { id: "rustaceo", icon: "🦀", name: "Rustáceo", desc: "Instale 3 apps feitos em Rust.", xp: 120 },
    Achievement { id: "desconfiado", icon: "🔍", name: "Desconfiado", desc: "Revalide 10 repositórios no GitHub.", xp: 80 },
    Achievement { id: "segunda_opiniao", icon: "🧪", name: "Segunda opinião", desc: "Consulte um arquivo no VirusTotal.", xp: 40 },
    Achievement { id: "vigilante", icon: "🛡", name: "Vigilante", desc: "Faça 5 varreduras com o Windows Defender.", xp: 60 },
    Achievement { id: "atestado_em_dia", icon: "🏗", name: "Atestado em dia", desc: "Instale um app com atestado de build do GitHub.", xp: 60 },
    Achievement { id: "arqueologo", icon: "🕘", name: "Arqueólogo", desc: "Instale uma versão anterior de um app.", xp: 40 },
    Achievement { id: "maratonista", icon: "⏱", name: "Maratonista", desc: "Jogue 10 horas somando todos os jogos.", xp: 150 },
    Achievement { id: "retro", icon: "🕹", name: "Retrô", desc: "Instale um emulador.", xp: 40 },
    Achievement { id: "engenheiro_reverso", icon: "🧬", name: "Engenheiro reverso", desc: "Instale um port feito por engenharia reversa.", xp: 60 },
    Achievement { id: "tradutor", icon: "⚡", name: "Tradutor", desc: "Traduza 5 códigos de C/C++ para Rust.", xp: 50 },
    Achievement { id: "parceria_com_ia", icon: "🤖", name: "Parceria com IA", desc: "Faça uma tradução com IA que compile no rustc.", xp: 80 },
    Achievement { id: "terminal_raiz", icon: "⌨", name: "Terminal raiz", desc: "Instale uma ferramenta de terminal em Rust.", xp: 60 },
    Achievement { id: "precavido", icon: "💾", name: "Precavido", desc: "Faça um backup de saves.", xp: 40 },
    Achievement { id: "mudanca", icon: "📤", name: "Mudança de casa", desc: "Exporte ou compartilhe a sua biblioteca.", xp: 40 },
    Achievement { id: "modo_sofa", icon: "🎮", name: "Modo sofá", desc: "Abra o Modo TV.", xp: 30 },
    Achievement { id: "social", icon: "🌐", name: "Social", desc: "Publique o seu perfil para os amigos.", xp: 60 },
    Achievement { id: "boa_conversa", icon: "💬", name: "Boa conversa", desc: "Abra a central de bate-papo da comunidade.", xp: 30 },
    Achievement { id: "indicacao", icon: "🔗", name: "Indicação", desc: "Compartilhe um app com alguém.", xp: 30 },
    Achievement { id: "aluno_rustaceo", icon: "📖", name: "Primeiros passos em Rust", desc: "Compile ou execute um código na área Aprender Rust.", xp: 50 },
];

pub fn find(id: &str) -> Option<&'static Achievement> {
    ACHIEVEMENTS.iter().find(|a| a.id == id)
}

/// Fatos usados para desbloquear conquistas e calcular XP.
#[derive(Debug, Default, Clone)]
pub struct Facts {
    pub stats: HashMap<String, i64>,
    pub installed: usize,
    pub rust_installed: usize,
    pub attested_installed: usize,
    pub kinds: HashSet<String>,
    pub playtime_seconds: u64,
}

impl Facts {
    pub fn stat(&self, key: &str) -> i64 {
        self.stats.get(key).copied().unwrap_or(0)
    }
}

pub fn facts(conn: &Connection) -> Result<Facts> {
    let apps = db::list_all(conn)?;
    let installed: Vec<_> = apps.iter().filter(|a| a.installed.is_some()).collect();
    Ok(Facts {
        stats: db::get_stats(conn)?,
        installed: installed.len(),
        rust_installed: installed.iter().filter(|a| a.is_rust()).count(),
        attested_installed: installed
            .iter()
            .filter(|a| a.attestation == crate::github::ATTESTATION_PRESENT)
            .count(),
        kinds: installed.iter().filter_map(|a| a.kind.clone()).collect(),
        playtime_seconds: installed
            .iter()
            .filter_map(|a| a.installed.as_ref())
            .map(|i| i.playtime_seconds)
            .sum(),
    })
}

pub fn is_unlocked(id: &str, f: &Facts) -> bool {
    match id {
        "primeiro_download" => f.installed >= 1 || f.stat(STAT_DOWNLOADS) >= 1,
        "colecionador" => f.installed >= 10,
        "rustaceo" => f.rust_installed >= 3,
        "desconfiado" => f.stat(STAT_VERIFIES) >= 10,
        "segunda_opiniao" => f.stat(STAT_VT) >= 1,
        "vigilante" => f.stat(STAT_DEFENDER) >= 5,
        "atestado_em_dia" => f.attested_installed >= 1,
        "arqueologo" => f.stat(STAT_OLD_VERSION) >= 1,
        "maratonista" => f.playtime_seconds >= 10 * 3600,
        "retro" => f.kinds.contains("emulator"),
        "engenheiro_reverso" => f.kinds.contains("re-game"),
        "tradutor" => f.stat(STAT_TRANSLATIONS) >= 5,
        "parceria_com_ia" => f.stat(STAT_AI_OK) >= 1,
        "terminal_raiz" => f.kinds.contains("cli") || f.stat(STAT_CLI) >= 1,
        "precavido" => f.stat(STAT_SAVES) >= 1,
        "mudanca" => f.stat(STAT_LIBRARY_EXPORT) >= 1,
        "modo_sofa" => f.stat(STAT_TV) >= 1,
        "social" => f.stat(STAT_PROFILE_PUBLISHED) >= 1,
        "boa_conversa" => f.stat(STAT_COMMUNITY) >= 1,
        "indicacao" => f.stat(STAT_SHARES) >= 1,
        "aluno_rustaceo" => f.stat(STAT_LEARN_RUN) >= 1,
        _ => false,
    }
}

/// XP total: eventos + horas jogadas + conquistas desbloqueadas.
pub fn total_xp(f: &Facts, unlocked: &[String]) -> u64 {
    let events: u64 = STAT_XP
        .iter()
        .map(|(k, xp)| f.stat(k).max(0) as u64 * xp)
        .sum();
    let play = f.playtime_seconds / 3600 * XP_PER_HOUR_PLAYED;
    let ach: u64 = unlocked.iter().filter_map(|id| find(id)).map(|a| a.xp).sum();
    events + play + ach
}

/// Nível pelo XP: o nível N começa em 100·(N−1)² XP. Devolve (nível, XP dentro do nível, XP do nível inteiro).
pub fn level_for(xp: u64) -> (u32, u64, u64) {
    let mut level: u64 = 1;
    while 100 * level * level <= xp {
        level += 1;
    }
    let start = 100 * (level - 1) * (level - 1);
    let next = 100 * level * level;
    (level as u32, xp - start, next - start)
}

pub fn title_for(level: u32) -> &'static str {
    match level {
        0..=1 => "Novato",
        2..=3 => "Explorador",
        4..=6 => "Colecionador",
        7..=9 => "Veterano",
        _ => "Lenda da RRS",
    }
}

/// Desbloqueia o que for novo e devolve as conquistas desbloqueadas agora.
pub fn check_new(conn: &Connection) -> Result<Vec<&'static Achievement>> {
    let f = facts(conn)?;
    let mut new = Vec::new();
    for a in ACHIEVEMENTS {
        if is_unlocked(a.id, &f) && db::unlock_achievement(conn, a.id)? {
            new.push(a);
        }
    }
    Ok(new)
}

/// Soma um evento e desbloqueia o que for novo.
pub fn record(conn: &Connection, stat: &str, by: i64) -> Result<Vec<&'static Achievement>> {
    db::bump_stat(conn, stat, by)?;
    check_new(conn)
}

/// Resumo para a tela e para o perfil público.
#[derive(Debug, Clone, Default)]
pub struct Progress {
    pub xp: u64,
    pub level: u32,
    pub xp_in_level: u64,
    pub xp_level_size: u64,
    pub unlocked: Vec<(String, String)>,
    pub facts: Facts,
}

pub fn progress(conn: &Connection) -> Result<Progress> {
    let f = facts(conn)?;
    let unlocked = db::unlocked_achievements(conn)?;
    let ids: Vec<String> = unlocked.iter().map(|(id, _)| id.clone()).collect();
    let xp = total_xp(&f, &ids);
    let (level, xp_in_level, xp_level_size) = level_for(xp);
    Ok(Progress {
        xp,
        level,
        xp_in_level,
        xp_level_size,
        unlocked,
        facts: f,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn niveis_crescem_com_o_xp() {
        assert_eq!(level_for(0), (1, 0, 100));
        assert_eq!(level_for(99), (1, 99, 100));
        assert_eq!(level_for(100), (2, 0, 300));
        assert_eq!(level_for(400), (3, 0, 500));
        assert_eq!(level_for(1000).0, 4);
        assert_eq!(title_for(1), "Novato");
        assert_eq!(title_for(5), "Colecionador");
        assert_eq!(title_for(12), "Lenda da RRS");
    }

    #[test]
    fn todas_as_conquistas_tem_regra_e_ids_unicos() {
        let mut ids = HashSet::new();
        let tudo = Facts {
            stats: STAT_XP.iter().map(|(k, _)| (k.to_string(), 100)).collect(),
            installed: 20,
            rust_installed: 5,
            attested_installed: 1,
            kinds: ["emulator", "re-game", "cli"].iter().map(|s| s.to_string()).collect(),
            playtime_seconds: 20 * 3600,
        };
        for a in ACHIEVEMENTS {
            assert!(ids.insert(a.id), "id repetido: {}", a.id);
            assert!(is_unlocked(a.id, &tudo), "{} não tem regra que desbloqueie", a.id);
            assert!(!is_unlocked(a.id, &Facts::default()), "{} desbloqueia sem fazer nada", a.id);
        }
    }

    #[test]
    fn eventos_geram_xp_e_desbloqueiam_uma_vez() {
        let conn = db::open(":memory:").unwrap();
        let novas = record(&conn, STAT_TV, 1).unwrap();
        assert_eq!(novas.iter().map(|a| a.id).collect::<Vec<_>>(), vec!["modo_sofa"]);
        assert!(record(&conn, STAT_TV, 1).unwrap().is_empty(), "não desbloqueia de novo");
        let p = progress(&conn).unwrap();
        // 2 aberturas do Modo TV (20 XP cada) + conquista Modo sofá (30 XP)
        assert_eq!(p.xp, 70);
        assert_eq!(p.level, 1);
        assert_eq!(p.unlocked.len(), 1);
    }

    #[test]
    fn horas_jogadas_contam_xp() {
        let f = Facts { playtime_seconds: 3 * 3600 + 1800, ..Default::default() };
        assert_eq!(total_xp(&f, &[]), 90);
        assert_eq!(total_xp(&f, &["maratonista".into()]), 240);
    }
}
