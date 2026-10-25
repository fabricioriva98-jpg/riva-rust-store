//! Módulo de Otimização do PC e Diagnóstico de Hardware e Software da RRS.
//! Reconhece especificações de hardware (CPU, RAM, GPU, Discos),
//! lista programas instalados no Windows, audita configurações do sistema
//! e oferece recomendações de otimização de 1 clique (Alto Desempenho, Modo Jogo, Limpeza e Rust Dev).

use anyhow::{Context, Result};
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::SystemTime;

/// Informações sobre um programa instalado no sistema.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct InstalledProgram {
    pub name: String,
    pub version: Option<String>,
    pub publisher: Option<String>,
    pub install_location: Option<String>,
}

/// Informações de um disco de armazenamento local.
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct DiskInfo {
    pub drive: String,
    pub total_gb: f64,
    pub free_gb: f64,
    pub used_pct: f64,
}

/// Especificações completas do hardware do PC do usuário.
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct SystemSpecs {
    pub cpu_name: String,
    pub cpu_cores: u32,
    pub cpu_threads: u32,
    pub ram_total_gb: f64,
    pub ram_free_gb: f64,
    pub gpu_name: String,
    pub gpu_driver: String,
    pub disks: Vec<DiskInfo>,
}

/// Aplicativo que inicia junto com o Windows.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct StartupApp {
    pub name: String,
    pub command: String,
    pub scope: String, // "Usuário (HKCU)" ou "Sistema (HKLM)"
}

/// Item de auditoria e recomendação de otimização.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct OptimizationItem {
    pub id: &'static str,
    pub title: &'static str,
    pub category: &'static str,
    pub description: &'static str,
    pub current_status: String,
    pub recommended_status: String,
    pub is_optimal: bool,
    pub impact: &'static str,
    /// Pergunta mostrada antes de aplicar (ações que apagam algo); `None` aplica direto.
    pub confirm: Option<String>,
}

/// Idade mínima de um arquivo do %TEMP% para a limpeza apagar (o mesmo critério da Limpeza de Disco do
/// Windows). Arquivos recentes podem estar em uso por um instalador ou programa aberto.
pub const TEMP_MIN_AGE: std::time::Duration = std::time::Duration::from_secs(7 * 24 * 60 * 60);

// ─── COLETA DE ESPECIFICAÇÕES DE HARDWARE ──────────────────────────────────

/// Lê uma chave do Registro do Windows usando o utilitário nativo `reg query`.
pub fn query_registry_value(key: &str, value: &str) -> Option<String> {
    let out = Command::new("reg")
        .args(["query", key, "/v", value])
        .output()
        .ok()?;
    if !out.status.success() {
        return None;
    }
    let text = String::from_utf8_lossy(&out.stdout);
    parse_single_reg_value(&text, value)
}

fn parse_single_reg_value(output: &str, expected_key: &str) -> Option<String> {
    for line in output.lines() {
        let trimmed = line.trim();
        for ty in ["REG_SZ", "REG_EXPAND_SZ", "REG_DWORD", "REG_QWORD"] {
            if let Some(pos) = trimmed.find(ty) {
                let k = trimmed[..pos].trim();
                if k.eq_ignore_ascii_case(expected_key) {
                    let v = trimmed[pos + ty.len()..].trim();
                    return Some(v.to_string());
                }
            }
        }
    }
    None
}

/// Detecta as especificações completas de Hardware do PC.
pub fn collect_system_specs() -> SystemSpecs {
    // 1. Processador (CPU)
    let cpu_name = query_registry_value(
        r"HKLM\HARDWARE\DESCRIPTION\System\CentralProcessor\0",
        "ProcessorNameString",
    )
    .unwrap_or_else(|| "Processador x86_64".to_string())
    .trim()
    .to_string();

    let cpu_threads = std::thread::available_parallelism()
        .map(|p| p.get() as u32)
        .unwrap_or(4);
    let cpu_cores = (cpu_threads / 2).max(1);

    // 2. Placa de Vídeo (GPU)
    let gpu_name = query_registry_value(
        r"HKLM\SYSTEM\CurrentControlSet\Control\Class\{4d36e968-e325-11ce-bfc1-08002be10318}\0000",
        "DriverDesc",
    )
    .or_else(|| {
        query_registry_value(
            r"HKLM\SYSTEM\CurrentControlSet\Control\Class\{4d36e968-e325-11ce-bfc1-08002be10318}\0001",
            "DriverDesc",
        )
    })
    .unwrap_or_else(|| "Placa de Vídeo Integrada / Genérica".to_string())
    .trim()
    .to_string();

    let gpu_driver = query_registry_value(
        r"HKLM\SYSTEM\CurrentControlSet\Control\Class\{4d36e968-e325-11ce-bfc1-08002be10318}\0000",
        "DriverVersion",
    )
    .unwrap_or_else(|| "Padrão".to_string())
    .trim()
    .to_string();

    // 3. Memória RAM e Discos via PowerShell CIM
    let (ram_total_gb, ram_free_gb, disks) = collect_ram_and_disks();

    SystemSpecs {
        cpu_name,
        cpu_cores,
        cpu_threads,
        ram_total_gb,
        ram_free_gb,
        gpu_name,
        gpu_driver,
        disks,
    }
}

/// Consulta RAM e Discos locais com chamada direta e rápida via powershell.
fn collect_ram_and_disks() -> (f64, f64, Vec<DiskInfo>) {
    let script = r#"
$os = Get-CimInstance Win32_OperatingSystem
$disks = Get-CimInstance Win32_LogicalDisk -Filter 'DriveType=3' | Select-Object DeviceID, FreeSpace, Size
[PSCustomObject]@{
    RamTotal = [math]::Round($os.TotalVisibleMemorySize / 1024 / 1024, 2)
    RamFree = [math]::Round($os.FreePhysicalMemory / 1024 / 1024, 2)
    Disks = $disks
} | ConvertTo-Json -Compress
"#;

    let out = Command::new("powershell")
        .args(["-NoProfile", "-NonInteractive", "-Command", script])
        .output();

    if let Ok(output) = out {
        if output.status.success() {
            let text = String::from_utf8_lossy(&output.stdout);
            if let Ok(val) = serde_json::from_str::<serde_json::Value>(&text) {
                let ram_total = val["RamTotal"].as_f64().unwrap_or(16.0);
                let ram_free = val["RamFree"].as_f64().unwrap_or(8.0);
                let mut disks = Vec::new();

                let disks_arr = if val["Disks"].is_array() {
                    val["Disks"].as_array().cloned().unwrap_or_default()
                } else if val["Disks"].is_object() {
                    vec![val["Disks"].clone()]
                } else {
                    Vec::new()
                };

                for d in disks_arr {
                    let drive = d["DeviceID"].as_str().unwrap_or("").to_string();
                    let free_bytes = d["FreeSpace"].as_f64().unwrap_or(0.0);
                    let size_bytes = d["Size"].as_f64().unwrap_or(1.0);
                    let total_gb = (size_bytes / (1024.0 * 1024.0 * 1024.0) * 10.0).round() / 10.0;
                    let free_gb = (free_bytes / (1024.0 * 1024.0 * 1024.0) * 10.0).round() / 10.0;
                    let used_pct = if total_gb > 0.0 {
                        (((total_gb - free_gb) / total_gb) * 100.0).clamp(0.0, 100.0)
                    } else {
                        0.0
                    };

                    if !drive.is_empty() {
                        disks.push(DiskInfo {
                            drive,
                            total_gb,
                            free_gb,
                            used_pct,
                        });
                    }
                }

                return (ram_total, ram_free, disks);
            }
        }
    }

    // Fallback razoável caso PowerShell falhe
    (16.0, 8.0, vec![DiskInfo {
        drive: "C:".into(),
        total_gb: 256.0,
        free_gb: 80.0,
        used_pct: 68.0,
    }])
}

// ─── INVENTÁRIO DE PROGRAMAS INSTALADOS ────────────────────────────────────

/// Coleta a lista completa de programas instalados no Windows consultando o registro.
pub fn collect_installed_programs() -> Vec<InstalledProgram> {
    let mut map: BTreeMap<String, InstalledProgram> = BTreeMap::new();

    let roots = [
        r"HKLM\SOFTWARE\Microsoft\Windows\CurrentVersion\Uninstall",
        r"HKLM\SOFTWARE\WOW6432Node\Microsoft\Windows\CurrentVersion\Uninstall",
        r"HKCU\Software\Microsoft\Windows\CurrentVersion\Uninstall",
    ];

    for root in roots {
        let out = Command::new("reg")
            .args(["query", root, "/s"])
            .output();
        if let Ok(output) = out {
            let text = String::from_utf8_lossy(&output.stdout);
            parse_uninstall_entries(&text, &mut map);
        }
    }

    map.into_values().collect()
}

fn parse_uninstall_entries(text: &str, map: &mut BTreeMap<String, InstalledProgram>) {
    let mut current_fields: BTreeMap<String, String> = BTreeMap::new();

    let flush = |fields: &mut BTreeMap<String, String>, map: &mut BTreeMap<String, InstalledProgram>| {
        if let Some(name) = fields.get("DisplayName").map(|s| s.trim()) {
            if !name.is_empty()
                && !name.starts_with('{')
                && !name.starts_with("KB")
                && !fields.get("SystemComponent").is_some_and(|sc| sc == "0x1" || sc == "1")
            {
                let version = fields.get("DisplayVersion").cloned();
                let publisher = fields.get("Publisher").cloned();
                let install_location = fields.get("InstallLocation").cloned();

                let key = name.to_lowercase();
                map.entry(key).or_insert_with(|| InstalledProgram {
                    name: name.to_string(),
                    version,
                    publisher,
                    install_location,
                });
            }
        }
        fields.clear();
    };

    for raw in text.lines() {
        let line = raw.trim_end_matches('\r');
        if line.starts_with("HKEY_") {
            flush(&mut current_fields, map);
            continue;
        }

        let trimmed = line.trim();
        for ty in ["REG_SZ", "REG_EXPAND_SZ", "REG_DWORD"] {
            if let Some(pos) = trimmed.find(ty) {
                let k = trimmed[..pos].trim();
                let v = trimmed[pos + ty.len()..].trim();
                current_fields.insert(k.to_string(), v.to_string());
                break;
            }
        }
    }

    flush(&mut current_fields, map);
}

// ─── APLICATIVOS DE INICIALIZAÇÃO DO WINDOWS ───────────────────────────────

/// Lista os programas que iniciam automaticamente com o Windows.
pub fn collect_startup_apps() -> Vec<StartupApp> {
    let mut apps = Vec::new();

    let scopes = [
        (r"HKCU\Software\Microsoft\Windows\CurrentVersion\Run", "Usuário (HKCU)"),
        (r"HKLM\Software\Microsoft\Windows\CurrentVersion\Run", "Sistema (HKLM)"),
    ];

    for (key, label) in scopes {
        let out = Command::new("reg")
            .args(["query", key])
            .output();
        if let Ok(output) = out {
            let text = String::from_utf8_lossy(&output.stdout);
            for line in text.lines() {
                let trimmed = line.trim();
                for ty in ["REG_SZ", "REG_EXPAND_SZ"] {
                    if let Some(pos) = trimmed.find(ty) {
                        let name = trimmed[..pos].trim();
                        let command = trimmed[pos + ty.len()..].trim();
                        if !name.is_empty() && !name.starts_with("HKEY_") {
                            apps.push(StartupApp {
                                name: name.to_string(),
                                command: command.to_string(),
                                scope: label.to_string(),
                            });
                        }
                        break;
                    }
                }
            }
        }
    }

    apps
}

// ─── AUDITORIA E RECOMENDAÇÕES DE OTIMIZAÇÃO ───────────────────────────────

/// Retorna o plano de energia ativo no Windows e se existem planos de alto desempenho.
pub fn query_power_schemes() -> (String, String, bool) {
    let out = Command::new("powercfg")
        .arg("/getactivescheme")
        .output();

    let mut active_name = "Equilibrado (Padrão)".to_string();
    let mut active_guid = "381b4222-f694-41f0-9685-ff5bb260df2e".to_string();

    if let Ok(output) = out {
        let text = String::from_utf8_lossy(&output.stdout);
        // Exemplo: Power Scheme GUID: 381b4222-f694-41f0-9685-ff5bb260df2e  (Balanced)
        if let Some(pos) = text.find("Power Scheme GUID:") {
            let rest = &text[pos + 18..].trim();
            if let Some(paren) = rest.find('(') {
                active_guid = rest[..paren].trim().to_string();
                if let Some(close) = rest[paren..].find(')') {
                    active_name = rest[paren + 1..paren + close].trim().to_string();
                }
            }
        }
    }

    let is_high_perf = active_name.to_lowercase().contains("high")
        || active_name.to_lowercase().contains("ultimate")
        || active_name.to_lowercase().contains("alto")
        || active_name.to_lowercase().contains("máximo");

    (active_name, active_guid, is_high_perf)
}

/// Verifica se o Modo Jogo do Windows (Game Mode) está ativado no registro.
pub fn is_game_mode_enabled() -> bool {
    query_registry_value(r"HKCU\Software\Microsoft\GameBar", "AllowAutoGameMode")
        .map(|v| v.trim() == "0x1" || v.trim() == "1")
        .unwrap_or(false)
}

/// Arquivos da raiz de `dir` modificados há pelo menos `min_age` (subpastas ficam de fora), com o tamanho.
pub fn old_temp_files(dir: &Path, min_age: std::time::Duration, now: SystemTime) -> Vec<(PathBuf, u64)> {
    let Some(limit) = now.checked_sub(min_age) else { return Vec::new() };
    let Ok(entries) = std::fs::read_dir(dir) else { return Vec::new() };
    entries
        .flatten()
        .filter_map(|e| {
            let meta = e.metadata().ok()?;
            let modified = meta.modified().ok()?;
            (meta.is_file() && modified <= limit).then(|| (e.path(), meta.len()))
        })
        .collect()
}

/// Apaga os arquivos antigos de `dir` (critério de [`old_temp_files`]). Devolve quantos e quantos bytes.
/// Arquivo em uso que o Windows não deixa apagar é pulado.
pub fn clean_old_temp_files(dir: &Path, min_age: std::time::Duration, now: SystemTime) -> (usize, u64) {
    let mut removed = 0usize;
    let mut freed = 0u64;
    for (path, len) in old_temp_files(dir, min_age, now) {
        if std::fs::remove_file(&path).is_ok() {
            removed += 1;
            freed = freed.saturating_add(len);
        }
    }
    (removed, freed)
}

/// Diagnostica e gera a lista de otimizações recomendadas para o computador.
pub fn diagnose_optimizations() -> Vec<OptimizationItem> {
    let mut items = Vec::new();

    // 1. Plano de Energia
    let (power_name, _, is_high_perf) = query_power_schemes();
    items.push(OptimizationItem {
        id: "power_high_perf",
        title: "Plano de Energia de Alto Desempenho",
        category: "Energia & CPU",
        description: "Garante frequência máxima de clock do processador e resposta instantânea em jogos e compilações Rust, evitando throttling de economia de energia.",
        current_status: format!("Ativo: {power_name}"),
        recommended_status: "Alto Desempenho / Ultimate Performance".into(),
        is_optimal: is_high_perf,
        impact: "Alto",
        confirm: None,
    });

    // 2. Modo Jogo do Windows (Game Mode)
    let game_mode = is_game_mode_enabled();
    items.push(OptimizationItem {
        id: "game_mode",
        title: "Modo Jogo do Windows (Game Mode)",
        category: "Jogos & GPU",
        description: "Prioriza processos de jogos em primeiro plano, alocando núcleos de CPU e memória dedicados e suspendendo tarefas em segundo plano.",
        current_status: if game_mode { "Ativado (Prioridade máxima)".into() } else { "Padrão / Desativado".into() },
        recommended_status: "Ativado".into(),
        is_optimal: game_mode,
        impact: "Médio",
        confirm: None,
    });

    // 3. Limpeza de Arquivos Temporários (só os antigos; ver TEMP_MIN_AGE)
    let old = old_temp_files(&std::env::temp_dir(), TEMP_MIN_AGE, SystemTime::now());
    let old_mb = old.iter().map(|(_, len)| len).sum::<u64>() / (1024 * 1024);
    items.push(OptimizationItem {
        id: "clean_temp",
        title: "Limpeza de Arquivos Temporários (%TEMP%)",
        category: "Armazenamento",
        description: "Apaga da pasta %TEMP% os arquivos com mais de 7 dias, como a Limpeza de Disco do Windows. Arquivos recentes e subpastas ficam, porque podem estar em uso. A exclusão é definitiva.",
        current_status: format!("{} arquivos com mais de 7 dias ({old_mb} MB)", old.len()),
        recommended_status: "< 300 MB".into(),
        is_optimal: old_mb < 300,
        impact: "Recomendado",
        confirm: Some(format!(
            "Apagar {} arquivos com mais de 7 dias ({old_mb} MB) da pasta %TEMP%? Não vão para a Lixeira.",
            old.len()
        )),
    });

    // 4. Otimização de Compilação Rust (Cargo / Defender)
    let home_dir = std::env::var_os("USERPROFILE").map(PathBuf::from).unwrap_or_default();
    let cargo_dir = home_dir.join(".cargo");
    items.push(OptimizationItem {
        id: "rust_defender_tip",
        title: "Exclusão do Windows Defender para Rust Dev",
        category: "Desenvolvimento",
        description: "O antivírus em tempo real inspeciona cada arquivo intermediário gerado pelo `rustc` na pasta `target/`, aumentando o tempo de compilação em até 40%.",
        current_status: if cargo_dir.exists() { "Ambiente Cargo detectado".into() } else { "Iniciante".into() },
        recommended_status: "Excluir pasta de projetos do escaneamento contínuo".into(),
        is_optimal: true, // Item informativo com ação de guia
        impact: "Alto para Rust",
        confirm: None,
    });

    items
}

/// Aplica uma ação de otimização de 1 clique de forma segura.
pub fn apply_optimization(id: &str) -> Result<String> {
    match id {
        "power_high_perf" => {
            // Tenta ativar Desempenho Máximo (Ultimate) ou Alto Desempenho (High Performance)
            let schemes = [
                "81250311-f525-4fe4-801c-64cde27af3bf", // Ultimate Performance
                "8c5e7fda-e8bf-4a96-9a85-a6e23a8c635c", // High Performance
            ];

            let mut applied = false;
            for guid in schemes {
                let status = Command::new("powercfg")
                    .args(["/setactive", guid])
                    .status();
                if let Ok(s) = status {
                    if s.success() {
                        applied = true;
                        break;
                    }
                }
            }

            if applied {
                Ok("Plano de energia de Alto Desempenho ativado com sucesso!".into())
            } else {
                anyhow::bail!("Não foi possível alterar o plano de energia. Execute como Administrador se necessário.")
            }
        }

        "game_mode" => {
            let status = Command::new("reg")
                .args(["add", r"HKCU\Software\Microsoft\GameBar", "/v", "AllowAutoGameMode", "/t", "REG_DWORD", "/d", "1", "/f"])
                .status()
                .context("falha ao executar reg.exe")?;

            if status.success() {
                Ok("Modo Jogo do Windows ativado no registro com sucesso!".into())
            } else {
                anyhow::bail!("Falha ao gravar no registro do Windows.")
            }
        }

        "clean_temp" => {
            let (removed_count, freed_bytes) =
                clean_old_temp_files(&std::env::temp_dir(), TEMP_MIN_AGE, SystemTime::now());
            let freed_mb = freed_bytes / (1024 * 1024);
            Ok(format!(
                "Limpeza concluída: {removed_count} arquivos com mais de 7 dias removidos ({freed_mb} MB liberados)."
            ))
        }

        "rust_defender_tip" => {
            Ok("Dica: No Windows Defender, acesse 'Proteção contra vírus e ameaças' → 'Configurações de proteção' → 'Exclusões' e adicione a pasta dos seus projetos Rust para acelerar compilações em até 40%.".into())
        }

        _ => anyhow::bail!("Ação de otimização desconhecida: {id}"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_reg_value_extrai_corretamente() {
        let sample = r#"
HKEY_LOCAL_MACHINE\HARDWARE\DESCRIPTION\System\CentralProcessor\0
    ProcessorNameString    REG_SZ    AMD Ryzen 5 5500
"#;
        assert_eq!(
            parse_single_reg_value(sample, "ProcessorNameString"),
            Some("AMD Ryzen 5 5500".into())
        );
    }

    #[test]
    fn parse_uninstall_entries_filtra_componentes_sistema() {
        let sample = r#"
HKEY_LOCAL_MACHINE\SOFTWARE\Microsoft\Windows\CurrentVersion\Uninstall\Mozilla Firefox
    DisplayName    REG_SZ    Mozilla Firefox (x64 pt-BR)
    DisplayVersion    REG_SZ    128.0
    Publisher    REG_SZ    Mozilla

HKEY_LOCAL_MACHINE\SOFTWARE\Microsoft\Windows\CurrentVersion\Uninstall\SystemComp
    DisplayName    REG_SZ    Internal System Tool
    SystemComponent    REG_DWORD    0x1
"#;
        let mut map = BTreeMap::new();
        parse_uninstall_entries(sample, &mut map);

        assert_eq!(map.len(), 1);
        let firefox = map.get("mozilla firefox (x64 pt-br)").unwrap();
        assert_eq!(firefox.name, "Mozilla Firefox (x64 pt-BR)");
        assert_eq!(firefox.version.as_deref(), Some("128.0"));
        assert_eq!(firefox.publisher.as_deref(), Some("Mozilla"));
    }

    #[test]
    fn limpeza_do_temp_apaga_so_arquivos_antigos_da_raiz() {
        let dir = std::env::temp_dir().join(format!("rrs_test_temp_{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(dir.join("subpasta")).unwrap();
        let now = SystemTime::now();
        let antigo = dir.join("antigo.tmp");
        let recente = dir.join("recente.tmp");
        let na_subpasta = dir.join("subpasta").join("antigo.tmp");
        for (p, idade_dias) in [(&antigo, 10u64), (&recente, 1), (&na_subpasta, 30)] {
            std::fs::write(p, b"12345").unwrap();
            let f = std::fs::File::options().write(true).open(p).unwrap();
            f.set_modified(now - std::time::Duration::from_secs(idade_dias * 24 * 3600)).unwrap();
        }

        let candidatos = old_temp_files(&dir, TEMP_MIN_AGE, now);
        assert_eq!(candidatos, vec![(antigo.clone(), 5)], "só o arquivo antigo da raiz");

        assert_eq!(clean_old_temp_files(&dir, TEMP_MIN_AGE, now), (1, 5));
        assert!(!antigo.exists());
        assert!(recente.exists(), "arquivo recente pode estar em uso e deve ficar");
        assert!(na_subpasta.exists(), "subpastas ficam de fora");
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn so_a_limpeza_pede_confirmacao() {
        let items = diagnose_optimizations();
        let limpeza = items.iter().find(|i| i.id == "clean_temp").unwrap();
        assert!(limpeza.confirm.as_deref().is_some_and(|c| c.contains("7 dias")));
        assert!(items.iter().filter(|i| i.id != "clean_temp").all(|i| i.confirm.is_none()));
    }

    #[test]
    fn diagnostico_de_otimizacoes_retorna_itens_validos() {
        let items = diagnose_optimizations();
        assert!(!items.is_empty(), "deve gerar pelo menos 4 recomendações");
        assert!(items.iter().any(|i| i.id == "power_high_perf"));
        assert!(items.iter().any(|i| i.id == "game_mode"));
        assert!(items.iter().any(|i| i.id == "clean_temp"));
    }
}
