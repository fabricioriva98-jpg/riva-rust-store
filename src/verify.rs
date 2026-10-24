//! Verificação de confiança: análise de precedentes do repositório + consulta de hash no VirusTotal.
//!
//! Importante: nota alta NÃO garante que o software é 100% seguro; é um indicador de precedentes.

use crate::github::{platform_of, Asset, Release, Repo};
use anyhow::Result;
use chrono::{DateTime, Utc};

#[derive(Debug, Clone, Copy)]
pub struct VtStats {
    pub malicious: u64,
    pub suspicious: u64,
    pub harmless: u64,
    pub undetected: u64,
}

pub struct VtResult {
    pub asset: String,
    /// `None` = hash desconhecido pelo VirusTotal.
    pub stats: Option<VtStats>,
}

#[derive(Debug, Clone)]
pub struct Report {
    pub score: i32,
    pub level: &'static str,
    pub notes: Vec<String>,
}

/// Consulta um SHA256 no VirusTotal sem baixar o arquivo.
pub async fn vt_lookup(client: &reqwest::Client, key: &str, sha256: &str) -> Result<Option<VtStats>> {
    let clean = sha256.trim().trim_start_matches("sha256:");
    let resp = client
        .get(format!("https://www.virustotal.com/api/v3/files/{clean}"))
        .header("x-apikey", key.trim())
        .send()
        .await?;
    if resp.status() == reqwest::StatusCode::NOT_FOUND {
        return Ok(None);
    }
    if !resp.status().is_success() {
        anyhow::bail!("VirusTotal retornou HTTP {}", resp.status());
    }
    let v: serde_json::Value = resp.json().await?;
    let s = &v["data"]["attributes"]["last_analysis_stats"];
    let n = |k: &str| s[k].as_u64().unwrap_or(0);
    Ok(Some(VtStats {
        malicious: n("malicious"),
        suspicious: n("suspicious"),
        harmless: n("harmless"),
        undetected: n("undetected"),
    }))
}

fn days_since(ts: &str) -> Option<i64> {
    if ts.is_empty() {
        return None;
    }
    let t = DateTime::parse_from_rfc3339(ts).ok()?;
    Some((Utc::now() - t.with_timezone(&Utc)).num_days())
}

pub fn evaluate(
    repo: &Repo,
    release: Option<&Release>,
    contributors: Option<u64>,
    vt: &[VtResult],
) -> Report {
    let assets: &[Asset] = release.map(|r| r.assets.as_slice()).unwrap_or(&[]);
    let license = repo
        .license
        .as_ref()
        .and_then(|l| l.spdx_id.as_deref())
        .unwrap_or("");
    evaluate_metadata(
        &repo.created_at,
        repo.pushed_at.as_deref().unwrap_or(""),
        repo.stargazers_count,
        contributors,
        license,
        repo.has_issues,
        repo.fork,
        repo.archived,
        release.is_some(),
        assets,
        vt,
    )
}

#[allow(clippy::too_many_arguments)]
pub fn evaluate_metadata(
    created_at: &str,
    pushed_at: &str,
    stars: u64,
    contributors: Option<u64>,
    license: &str,
    has_issues: bool,
    fork: bool,
    archived: bool,
    has_release: bool,
    assets: &[Asset],
    vt: &[VtResult],
) -> Report {
    let mut score: i32 = 0;
    let mut notes = Vec::new();

    // 1. Idade do repositório (até 30 pts)
    match days_since(created_at) {
        Some(d) if d > 365 => {
            score += 30;
            notes.push(format!("🟢 Repositório consolidado ({} anos de histórico)", d / 365));
        }
        Some(d) if d > 90 => {
            score += 18;
            notes.push(format!("🟡 Repositório com {d} dias de existência"));
        }
        Some(d) if d > 30 => {
            score += 6;
            notes.push(format!("🟡 Repositório recente ({d} dias)"));
        }
        Some(d) => notes.push(format!("🔴 Repositório muito novo ({d} dias)")),
        None => {
            // Quando calculado de catálogo sem created_at mas com estrelas altas
            if stars >= 1000 {
                score += 22;
            }
        }
    }

    // 2. Atividade recente (até 15 pts)
    match days_since(pushed_at) {
        Some(d) if d <= 180 => {
            score += 15;
            notes.push(format!("🟢 Projeto ativo (atualizado há {d} dias)"));
        }
        Some(d) if d <= 730 => {
            score += 8;
            notes.push(format!("🟡 Última atualização há {d} dias"));
        }
        Some(d) => notes.push(format!("🔴 Sem atualizações há {d} dias")),
        None => {}
    }

    // 3. Popularidade e revisão comunitária (até 20 pts)
    if stars >= 5000 {
        score += 20;
    } else if stars >= 1000 {
        score += 17;
    } else if stars >= 200 {
        score += 12;
    } else if stars >= 50 {
        score += 6;
    }
    notes.push(format!(
        "{} {} estrelas no GitHub",
        if stars >= 200 { "🟢" } else { "🟡" },
        stars
    ));

    // 4. Contribuidores (até 10 pts)
    if let Some(c) = contributors {
        if c >= 5 {
            score += 10;
            notes.push(format!("🟢 {c} contribuidores públicos"));
        } else if c >= 2 {
            score += 6;
            notes.push(format!("🟡 {c} contribuidores"));
        } else {
            notes.push("🟡 Apenas 1 autor/contribuidor".into());
        }
    } else if stars >= 1000 {
        score += 8;
    }

    // 5. Licença aberta (10 pts)
    if !license.is_empty() && license != "NOASSERTION" {
        score += 10;
        notes.push(format!("🟢 Código aberto com licença {license}"));
    } else {
        notes.push("🟡 Sem licença open-source padronizada".into());
    }

    // 6. Canal de Issues ativo (5 pts)
    if has_issues {
        score += 5;
    } else {
        notes.push("🟡 Aba de Issues desativada no repositório".into());
    }

    // 7. Hash SHA-256 oficial do GitHub (10 pts)
    if has_release {
        let inst: Vec<_> = assets
            .iter()
            .filter(|a| platform_of(&a.name).is_some())
            .collect();
        if !inst.is_empty() && inst.iter().all(|a| a.digest.is_some()) {
            score += 10;
            notes.push("🟢 Todos os instaladores possuem hash SHA-256 oficial do GitHub".into());
        } else if inst.iter().any(|a| a.digest.is_some()) {
            score += 5;
            notes.push("🟡 Parte dos instaladores possui hash SHA-256 oficial".into());
        } else if !inst.is_empty() {
            notes.push("🟡 Release antiga sem hash SHA-256 pré-calculado pelo GitHub".into());
        } else {
            notes.push("🟡 Release sem instaladores reconhecidos".into());
        }
    } else {
        notes.push("🟡 Sem Release publicada".into());
    }

    if fork {
        score -= 25;
        notes.push("🔴 É um FORK: verifique se não é uma cópia de outro projeto".into());
    }
    if archived {
        score -= 10;
        notes.push("🟡 Repositório arquivado (somente leitura)".into());
    }

    // 8. Resultados do VirusTotal (se consultados)
    let mut forced_red = false;
    for r in vt {
        match &r.stats {
            None => notes.push(format!(
                "⚪ VirusTotal: {} ainda não analisado na base (desconhecido)",
                r.asset
            )),
            Some(s) if s.malicious >= 3 => {
                forced_red = true;
                notes.push(format!(
                    "🔴 VirusTotal: {} → {} antivírus detectaram ameaça!",
                    r.asset, s.malicious
                ));
            }
            Some(s) if s.malicious + s.suspicious > 0 => {
                score -= 20;
                notes.push(format!(
                    "🟡 VirusTotal: {} → {} alerta(s), {} suspeito(s) (possível falso positivo)",
                    r.asset, s.malicious, s.suspicious
                ));
            }
            Some(s) => notes.push(format!(
                "🟢 VirusTotal: {} → 0 ameaças ({} motores limpos)",
                r.asset,
                s.harmless + s.undetected
            )),
        }
    }

    let score = score.clamp(0, 100);
    let level = if forced_red || score < 40 {
        "🔴 ALTO RISCO"
    } else if score < 70 {
        "🟡 ATENÇÃO"
    } else {
        "🟢 CONFIÁVEL"
    };
    Report { score, level, notes }
}
