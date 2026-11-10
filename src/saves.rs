//! Backup dos saves de jogos: o usuário aponta a pasta de saves de cada jogo e a RRS guarda cópias
//! compactadas em `backups/<dono>_<repo>/AAAA-MM-DD_HH-MM-SS.zip` (manual ou quando o jogo fecha).
//! Restaurar sempre faz antes um backup do estado atual ("antes-de-restaurar").

use crate::db;
use anyhow::{bail, Context, Result};
use rusqlite::Connection;
use std::path::{Path, PathBuf};

fn setting_key(full_name: &str) -> String {
    format!("save_dir:{}", full_name.to_lowercase())
}

pub fn save_dir(conn: &Connection, full_name: &str) -> Option<String> {
    db::get_setting(conn, &setting_key(full_name))
}

pub fn set_save_dir(conn: &Connection, full_name: &str, dir: &str) -> Result<()> {
    db::set_setting(conn, &setting_key(full_name), dir.trim())
}

pub fn backups_dir(base_dir: &Path, full_name: &str) -> PathBuf {
    base_dir
        .join("backups")
        .join(full_name.replace(['/', '\\', ':'], "_"))
}

/// Backups do jogo, do mais novo para o mais antigo.
pub fn list_backups(base_dir: &Path, full_name: &str) -> Vec<PathBuf> {
    let Ok(entries) = std::fs::read_dir(backups_dir(base_dir, full_name)) else {
        return Vec::new();
    };
    let mut list: Vec<PathBuf> = entries
        .flatten()
        .map(|e| e.path())
        .filter(|p| p.extension().is_some_and(|x| x.eq_ignore_ascii_case("zip")))
        .collect();
    list.sort();
    list.reverse();
    list
}

#[cfg(target_os = "windows")]
fn ps_quote(p: &Path) -> String {
    p.to_string_lossy().replace('\'', "''")
}

#[cfg(target_os = "windows")]
fn run_powershell(script: &str) -> Result<()> {
    let out = std::process::Command::new("powershell")
        .args(["-NoProfile", "-NonInteractive", "-WindowStyle", "Hidden", "-Command", script])
        .output()
        .context("não foi possível executar o PowerShell")?;
    if !out.status.success() {
        bail!("{}", String::from_utf8_lossy(&out.stderr).trim());
    }
    Ok(())
}

fn run_compress(source_dir: &Path, zip_file: &Path) -> Result<()> {
    #[cfg(target_os = "windows")]
    {
        let source = source_dir.join("*");
        run_powershell(&format!(
            "Compress-Archive -Path '{}' -DestinationPath '{}' -CompressionLevel Optimal",
            ps_quote(&source),
            ps_quote(zip_file)
        ))
        .context("falha ao compactar os saves")?;
    }
    #[cfg(not(target_os = "windows"))]
    {
        let status = std::process::Command::new("zip")
            .arg("-r")
            .arg("-q")
            .arg(zip_file)
            .arg(".")
            .current_dir(source_dir)
            .status()
            .context("falha ao executar 'zip' (verifique se está instalado no sistema)")?;
        if !status.success() {
            bail!("comando 'zip' terminou com erro: {:?}", status.code());
        }
    }
    Ok(())
}

fn run_extract(zip_file: &Path, dest_dir: &Path) -> Result<()> {
    #[cfg(target_os = "windows")]
    {
        run_powershell(&format!(
            "Expand-Archive -LiteralPath '{}' -DestinationPath '{}' -Force",
            ps_quote(zip_file),
            ps_quote(dest_dir)
        ))
        .context("falha ao restaurar o backup")?;
    }
    #[cfg(not(target_os = "windows"))]
    {
        let status = std::process::Command::new("unzip")
            .arg("-o")
            .arg("-q")
            .arg(zip_file)
            .arg("-d")
            .arg(dest_dir)
            .status()
            .context("falha ao executar 'unzip' (verifique se está instalado no sistema)")?;
        if !status.success() {
            bail!("comando 'unzip' terminou com erro: {:?}", status.code());
        }
    }
    Ok(())
}

/// Compacta o conteúdo da pasta de saves. `label` vai no nome do arquivo (ex.: "antes-de-restaurar").
pub fn backup(base_dir: &Path, full_name: &str, save_dir: &Path, label: Option<&str>) -> Result<PathBuf> {
    if !save_dir.is_dir() {
        bail!("a pasta de saves não existe: {}", save_dir.display());
    }
    if std::fs::read_dir(save_dir)?.next().is_none() {
        bail!("a pasta de saves está vazia: {}", save_dir.display());
    }
    let dir = backups_dir(base_dir, full_name);
    std::fs::create_dir_all(&dir)?;
    let stamp = chrono::Local::now().format("%Y-%m-%d_%H-%M-%S").to_string();
    let name = match label {
        Some(l) => format!("{stamp}_{l}.zip"),
        None => format!("{stamp}.zip"),
    };
    let zip = dir.join(name);
    run_compress(save_dir, &zip)?;
    Ok(zip)
}

/// Restaura um backup por cima da pasta de saves (depois de guardar o estado atual).
pub fn restore(base_dir: &Path, full_name: &str, zip: &Path, save_dir: &Path) -> Result<Option<PathBuf>> {
    if !zip.is_file() {
        bail!("backup não encontrado: {}", zip.display());
    }
    std::fs::create_dir_all(save_dir)?;
    let safety = match backup(base_dir, full_name, save_dir, Some("antes-de-restaurar")) {
        Ok(p) => Some(p),
        Err(_) if std::fs::read_dir(save_dir)?.next().is_none() => None,
        Err(e) => return Err(e.context("restauração cancelada: não deu para guardar os saves atuais")),
    };
    run_extract(zip, save_dir)?;
    Ok(safety)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn temp_dir(tag: &str) -> PathBuf {
        let nanos = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let dir = std::env::temp_dir().join(format!("rrs-teste-saves-{tag}-{}-{nanos}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        dir
    }

    #[test]
    fn backup_e_restauracao_com_copia_de_seguranca() {
        let base = temp_dir("base");
        let saves = temp_dir("saves");
        std::fs::write(saves.join("slot1.sav"), b"fase 3").unwrap();
        std::fs::create_dir_all(saves.join("perfis")).unwrap();
        std::fs::write(saves.join("perfis").join("p1.cfg"), b"som=on").unwrap();

        let zip = backup(&base, "dono/jogo", &saves, None).unwrap();
        assert!(zip.is_file());
        assert_eq!(list_backups(&base, "dono/jogo"), vec![zip.clone()]);

        // O jogo estraga o save; restaurar volta ao estado do backup e guarda o estado estragado
        std::fs::write(saves.join("slot1.sav"), b"corrompido").unwrap();
        std::thread::sleep(std::time::Duration::from_millis(1100));
        let safety = restore(&base, "dono/jogo", &zip, &saves).unwrap();
        assert_eq!(std::fs::read(saves.join("slot1.sav")).unwrap(), b"fase 3");
        assert_eq!(std::fs::read(saves.join("perfis").join("p1.cfg")).unwrap(), b"som=on");
        let safety = safety.expect("deveria guardar os saves atuais antes de restaurar");
        assert!(safety.to_string_lossy().contains("antes-de-restaurar"));
        assert_eq!(list_backups(&base, "dono/jogo").len(), 2);

        std::fs::remove_dir_all(&base).unwrap();
        std::fs::remove_dir_all(&saves).unwrap();
    }

    #[test]
    fn pasta_vazia_ou_inexistente_nao_gera_backup() {
        let base = temp_dir("base2");
        let vazia = temp_dir("vazia");
        assert!(backup(&base, "a/b", &vazia, None).is_err());
        assert!(backup(&base, "a/b", &vazia.join("nao-existe"), None).is_err());
        assert!(list_backups(&base, "a/b").is_empty());
        std::fs::remove_dir_all(&base).unwrap();
        std::fs::remove_dir_all(&vazia).unwrap();
    }

    #[test]
    fn pasta_de_save_fica_guardada_por_jogo() {
        let conn = db::open(":memory:").unwrap();
        set_save_dir(&conn, "Dono/Jogo", "C:\\Saves\\Jogo ").unwrap();
        assert_eq!(save_dir(&conn, "dono/jogo").as_deref(), Some("C:\\Saves\\Jogo"));
        assert!(save_dir(&conn, "outro/jogo").is_none());
    }
}
