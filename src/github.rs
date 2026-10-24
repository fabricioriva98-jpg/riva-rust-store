//! Cliente da API do GitHub e seleção inteligente de instaladores por plataforma/arquitetura.

use anyhow::{bail, Context, Result};
use reqwest::{header, Client, Response, StatusCode};
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Clone)]
pub struct License {
    pub spdx_id: Option<String>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct Repo {
    pub full_name: String,
    pub description: Option<String>,
    pub html_url: String,
    pub stargazers_count: u64,
    pub fork: bool,
    pub archived: bool,
    pub created_at: String,
    pub pushed_at: Option<String>,
    pub license: Option<License>,
    #[serde(default)]
    pub topics: Vec<String>,
    #[serde(default)]
    pub has_issues: bool,
}

#[derive(Debug, Deserialize, Serialize, Clone, PartialEq)]
pub struct Asset {
    pub name: String,
    pub size: u64,
    pub browser_download_url: String,
    /// Formato "sha256:HEX", calculado pelo próprio GitHub.
    pub digest: Option<String>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct Release {
    pub tag_name: String,
    #[allow(dead_code)]
    pub published_at: Option<String>,
    #[allow(dead_code)]
    #[serde(default)]
    pub prerelease: bool,
    pub assets: Vec<Asset>,
}

#[derive(Deserialize)]
struct SearchResult {
    items: Vec<Repo>,
}

pub struct Github {
    client: Client,
    pub has_token: bool,
}

impl Github {
    pub fn new(custom_token: Option<&str>) -> Result<Self> {
        let mut headers = header::HeaderMap::new();
        headers.insert(header::USER_AGENT, "gitstore/0.2".parse()?);
        headers.insert("X-GitHub-Api-Version", "2022-11-28".parse()?);
        let token = custom_token
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .or_else(|| std::env::var("GITHUB_TOKEN").ok().filter(|t| !t.trim().is_empty()));
        if let Some(t) = &token {
            headers.insert(header::AUTHORIZATION, format!("Bearer {t}").parse()?);
        }
        Ok(Self {
            client: Client::builder().default_headers(headers).build()?,
            has_token: token.is_some(),
        })
    }

    async fn get(&self, url: &str, accept: &str) -> Result<Response> {
        let resp = self
            .client
            .get(url)
            .header(header::ACCEPT, accept)
            .send()
            .await
            .with_context(|| format!("falha ao acessar {url}"))?;
        if resp.status() == StatusCode::FORBIDDEN || resp.status() == StatusCode::TOO_MANY_REQUESTS {
            bail!("Limite da API do GitHub atingido (60 req/h sem token). Configure um GITHUB_TOKEN gratuito.");
        }
        Ok(resp)
    }

    pub async fn search_repos(&self, query: &str, page: u32) -> Result<Vec<Repo>> {
        let url = format!(
            "https://api.github.com/search/repositories?q={}&sort=stars&order=desc&per_page=50&page={page}",
            urlencode(query)
        );
        let resp = self.get(&url, "application/vnd.github+json").await?;
        if !resp.status().is_success() {
            bail!("busca falhou: HTTP {}", resp.status());
        }
        Ok(resp.json::<SearchResult>().await?.items)
    }

    pub async fn repo(&self, full_name: &str) -> Result<Repo> {
        let clean = normalize_repo_name(full_name);
        let url = format!("https://api.github.com/repos/{clean}");
        let resp = self.get(&url, "application/vnd.github+json").await?;
        if !resp.status().is_success() {
            bail!("repositório '{clean}' não encontrado: HTTP {}", resp.status());
        }
        Ok(resp.json().await?)
    }

    pub async fn latest_release(&self, full_name: &str) -> Result<Option<Release>> {
        let clean = normalize_repo_name(full_name);
        let url = format!("https://api.github.com/repos/{clean}/releases/latest");
        let resp = self.get(&url, "application/vnd.github+json").await?;
        if resp.status() == StatusCode::NOT_FOUND {
            return Ok(None);
        }
        if !resp.status().is_success() {
            bail!("release falhou: HTTP {}", resp.status());
        }
        Ok(Some(resp.json().await?))
    }

    /// Número de contribuidores, lido da paginação (per_page=1).
    pub async fn contributors(&self, full_name: &str) -> Result<Option<u64>> {
        let clean = normalize_repo_name(full_name);
        let url = format!(
            "https://api.github.com/repos/{clean}/contributors?per_page=1&anon=true"
        );
        let resp = self.get(&url, "application/vnd.github+json").await?;
        if !resp.status().is_success() {
            return Ok(None);
        }
        if let Some(link) = resp.headers().get(header::LINK).and_then(|v| v.to_str().ok()) {
            if let Some(n) = last_page(link) {
                return Ok(Some(n));
            }
        }
        let list: Vec<serde_json::Value> = resp.json().await?;
        Ok(Some(list.len() as u64))
    }
}

/// Aceita tanto `dono/repo` quanto `https://github.com/dono/repo`.
pub fn normalize_repo_name(input: &str) -> String {
    let s = input.trim().trim_end_matches('/');
    let s = s
        .strip_prefix("https://github.com/")
        .or_else(|| s.strip_prefix("http://github.com/"))
        .or_else(|| s.strip_prefix("github.com/"))
        .unwrap_or(s);
    let mut parts = s.split('/');
    match (parts.next(), parts.next()) {
        (Some(owner), Some(repo)) => format!("{owner}/{}", repo.trim_end_matches(".git")),
        _ => s.to_string(),
    }
}

fn last_page(link: &str) -> Option<u64> {
    let part = link.split(',').find(|p| p.contains("rel=\"last\""))?;
    let url = part.split('>').next()?;
    let idx = url.rfind("&page=").or_else(|| url.rfind("?page="))?;
    url[idx + 6..].parse().ok()
}

fn urlencode(s: &str) -> String {
    s.bytes()
        .map(|b| match b {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => (b as char).to_string(),
            _ => format!("%{b:02X}"),
        })
        .collect()
}

/// Plataforma de um arquivo de release, ou `None` se for arquivo auxiliar/código-fonte.
pub fn platform_of(name: &str) -> Option<&'static str> {
    let n = name.to_lowercase();
    let ends = |exts: &[&str]| exts.iter().any(|e| n.ends_with(e));

    // Ignorar assinaturas, hashes, símbolos de debug, código-fonte e metadados de atualizadores
    if ends(&[
        ".sig", ".asc", ".sha256", ".sha256sum", ".sha512", ".md5", ".txt", ".blockmap", ".pdb",
        ".dwp", ".sym", ".zsync", ".torrent", ".yml", ".json", ".aab", ".dockerimage",
    ]) || n.contains("debug")
        || n.contains("symbols")
        || n.starts_with("pdb-")
        || n.contains("pluginsdk")
        || n.contains("-src")
        || n.contains("sources")
        || n.contains("source-code")
        || n.contains("vendor")
        || n.contains("yarn-cache")
    {
        return None;
    }

    if ends(&[".exe", ".msi", ".msix"]) {
        return Some("windows");
    }
    if ends(&[".apk"]) {
        return Some("android");
    }
    if ends(&[".dmg", ".pkg"]) && !n.contains("ps4") {
        return Some("macos");
    }
    if ends(&[".appimage", ".deb", ".rpm", ".flatpak"]) && !n.contains("iphoneos") {
        return Some("linux");
    }
    if ends(&[".jar"])
        && !["server", "sources", "javadoc", "assets", "dependencies", "lib", "api"]
            .iter()
            .any(|w| n.contains(w))
    {
        return Some("java");
    }
    if ends(&[".zip", ".7z", ".tar.gz", ".tgz", ".tar.xz"]) {
        if n.starts_with("ghidra_") && n.contains("_public_") {
            return Some("java");
        }
        if n.starts_with("snapshot_") {
            return Some("windows");
        }
        if n.contains("win") || n.contains("msvc") || n.contains("msys2") || n.contains("mxe") {
            return Some("windows");
        }
        if n.contains("linux") || n.contains("ubuntu") || n.contains("fedora") || n.contains("debian") {
            return Some("linux");
        }
        if n.contains("mac") || n.contains("osx") || n.contains("darwin") {
            return Some("macos");
        }
        if n.contains("android") {
            return Some("android");
        }
    }
    None
}

/// Escolhe automaticamente o índice do melhor instalador para o sistema atual (Windows x64 por padrão).
pub fn pick_best_asset(assets: &[Asset]) -> Option<usize> {
    if assets.is_empty() {
        return None;
    }
    let current_os = match std::env::consts::OS {
        "windows" => "windows",
        "linux" => "linux",
        "macos" => "macos",
        other => other,
    };
    let is_64 = std::env::consts::ARCH == "x86_64";

    assets
        .iter()
        .enumerate()
        .map(|(idx, a)| {
            let n = a.name.to_lowercase();
            let plat = platform_of(&a.name).unwrap_or("");
            let mut score: i32 = 0;

            if plat == current_os {
                score += 100;
            } else if plat == "java" {
                score += 50;
            } else {
                score -= 100;
            }

            // Preferência por instaladores diretos no Windows
            if n.ends_with(".msi") || n.ends_with(".exe") {
                score += 30;
            } else if n.ends_with(".zip") || n.ends_with(".7z") {
                score += 15;
            }

            if n.contains("setup") || n.contains("installer") {
                score += 10;
            }

            // Arquitetura
            if is_64 {
                let is_x64_name = n.contains("x64")
                    || n.contains("x86_64")
                    || n.contains("x86-64")
                    || n.contains("win64")
                    || n.contains("amd64");
                let is_arm_name = n.contains("arm64")
                    || n.contains("arm-64")
                    || n.contains("aarch64")
                    || n.contains("armv7")
                    || n.contains("arm32");

                if is_x64_name {
                    score += 25;
                }
                if is_arm_name {
                    score -= 60;
                }
                if (n.contains("win32") || n.contains("x86") || n.contains("i686") || n.contains("32bit"))
                    && !is_x64_name
                {
                    score -= 20;
                }
            }

            // Penalizar pacotes de servidor, não assinados ou linha de comando quando há cliente gráfico
            if n.contains("server") || n.contains("headless") || n.contains("cli") {
                score -= 40;
            }
            if n.contains("unsigned") {
                score -= 15;
            }

            // Preferir arquivos que possuem SHA-256 oficial do GitHub
            if a.digest.is_some() {
                score += 5;
            }

            (idx, score)
        })
        .max_by_key(|&(_, score)| score)
        .map(|(idx, _)| idx)
}
