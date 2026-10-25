//! winget (gerenciador de pacotes do Windows) como fonte extra de apps. A RRS só busca e chama o
//! `winget install`; quem baixa e confere o SHA-256 (do manifesto da Microsoft) é o próprio winget.
//! Pacotes do winget não recebem nota de confiança da RRS.

use anyhow::{bail, Context, Result};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WingetPackage {
    pub name: String,
    pub id: String,
    pub version: String,
}

/// IDs do winget são do tipo `Editora.Nome` (letras, números, ponto, hífen, sublinhado, +).
pub fn is_valid_id(id: &str) -> bool {
    !id.is_empty()
        && id.len() <= 128
        && id.contains('.')
        && id
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || matches!(c, '.' | '-' | '_' | '+'))
}

pub fn search(query: &str) -> Result<Vec<WingetPackage>> {
    let query = query.trim();
    if query.is_empty() {
        bail!("digite algo para buscar no winget");
    }
    let out = std::process::Command::new("winget")
        .args([
            "search",
            query,
            "--source",
            "winget",
            "--accept-source-agreements",
            "--disable-interactivity",
        ])
        .output()
        .context("winget não encontrado (instale o 'Instalador de Aplicativo' da Microsoft Store)")?;
    Ok(parse_search_output(&String::from_utf8_lossy(&out.stdout)))
}

/// Lê a tabela do `winget search`. As colunas são encontradas pela linha de cabeçalho (idioma do
/// Windows: "Nome ID Versão..." ou "Name Id Version...") logo acima da linha de traços.
pub fn parse_search_output(output: &str) -> Vec<WingetPackage> {
    // Remove a animação de progresso que o winget escreve com \r antes da tabela
    let lines: Vec<&str> = output
        .lines()
        .map(|l| l.rsplit('\r').next().unwrap_or(l))
        .collect();
    let Some(sep) = lines
        .iter()
        .position(|l| l.trim().len() >= 10 && l.trim().chars().all(|c| c == '-'))
    else {
        return Vec::new();
    };
    if sep == 0 {
        return Vec::new();
    }
    let header: Vec<char> = lines[sep - 1].chars().collect();
    // Início de cada coluna = posição (em caracteres) onde começa uma palavra do cabeçalho
    let mut starts = Vec::new();
    for (i, c) in header.iter().enumerate() {
        if !c.is_whitespace() && (i == 0 || header[i - 1].is_whitespace()) {
            starts.push(i);
        }
    }
    if starts.len() < 3 {
        return Vec::new();
    }
    let cell = |row: &[char], col: usize| -> String {
        let from = starts[col].min(row.len());
        let to = starts.get(col + 1).copied().unwrap_or(row.len()).min(row.len());
        row[from..to].iter().collect::<String>().trim().to_string()
    };
    lines[sep + 1..]
        .iter()
        .filter(|l| !l.trim().is_empty())
        .map(|l| l.chars().collect::<Vec<char>>())
        .map(|row| WingetPackage {
            name: cell(&row, 0),
            id: cell(&row, 1),
            version: cell(&row, 2),
        })
        .filter(|p| is_valid_id(&p.id))
        .collect()
}

/// Abre o `winget install` numa janela de console (o usuário acompanha o progresso e os avisos).
pub fn install(id: &str) -> Result<()> {
    if !is_valid_id(id) {
        bail!("ID do winget inválido: {id}");
    }
    std::process::Command::new("winget")
        .args([
            "install",
            "--id",
            id,
            "--exact",
            "--source",
            "winget",
            "--accept-package-agreements",
            "--accept-source-agreements",
        ])
        .spawn()
        .context("não foi possível iniciar o winget")?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    const PT: &str = "   - \r   \\ \rNome             ID                      Versão Correspondência\n---------------------------------------------------------------\nIndexSearch      Abyss116.IndexSearch    0.5.0  Tag: ripgrep\nRipGrep MSVC     BurntSushi.ripgrep.MSVC 15.2.0 Tag: ripgrep\n";
    // O winget alinha as colunas pelo conteúdo e corta valores longos com "…"
    const EN: &str = "Name               Id                         Version Match\n------------------------------------------------------------------\nVisual Studio Code Microsoft.VisualStudioCode 1.105.1 Moniker: vscode\nAlgum App Longo    Editora.NomeMuitoComprido… 2.0     Tag: app\n";

    #[test]
    fn le_a_tabela_em_portugues() {
        let p = parse_search_output(PT);
        assert_eq!(p.len(), 2, "{p:?}");
        assert_eq!(p[1].name, "RipGrep MSVC");
        assert_eq!(p[1].id, "BurntSushi.ripgrep.MSVC");
        assert_eq!(p[1].version, "15.2.0");
    }

    #[test]
    fn le_a_tabela_em_ingles_e_descarta_id_cortado() {
        let p = parse_search_output(EN);
        assert_eq!(p.len(), 1, "o ID cortado com \"…\" não pode virar instalação: {p:?}");
        assert_eq!(p[0].id, "Microsoft.VisualStudioCode");
        assert_eq!(p[0].version, "1.105.1");
    }

    #[test]
    fn valida_ids() {
        assert!(is_valid_id("BurntSushi.ripgrep.MSVC"));
        assert!(is_valid_id("Notepad++.Notepad++"));
        assert!(!is_valid_id("semponto"));
        assert!(!is_valid_id("a.b & calc"));
        assert!(!is_valid_id(""));
        assert!(parse_search_output("Nenhum pacote encontrado.").is_empty());
    }
}
