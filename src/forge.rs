//! Forjas além do GitHub: Gitea/Forgejo (ex.: Codeberg) e GitLab. As respostas são convertidas para as
//! mesmas estruturas do GitHub (`Repo`, `Release`, `Asset`), então o resto da loja não muda.
//!
//! Nome no catálogo: `dono/repo` para o GitHub; `host/dono/repo` para as outras forjas
//! (no GitLab o caminho pode ter subgrupos: `gitlab.com/grupo/sub/projeto`).
//! Outras forjas não publicam SHA-256 dos arquivos nem atestados de build: o hash é só calculado.

use crate::github::{self, Asset, Github, License, Release, Repo};
use anyhow::{anyhow, bail, Context, Result};
use reqwest::{header, Client, StatusCode};
use serde_json::Value;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ForgeKind {
    GitHub,
    Gitea,
    GitLab,
}

/// Hosts conhecidos de Gitea/Forgejo.
const GITEA_HOSTS: &[&str] = &["codeberg.org", "gitea.com", "git.disroot.org", "next.forgejo.org"];
/// Hosts conhecidos de GitLab (além de qualquer `gitlab.*`).
const GITLAB_HOSTS: &[&str] = &[
    "gitlab.com",
    "gitlab.gnome.org",
    "invent.kde.org",
    "gitlab.freedesktop.org",
    "framagit.org",
    "salsa.debian.org",
];

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RepoRef {
    pub kind: ForgeKind,
    pub host: String,
    /// `dono/repo` (ou `grupo/sub/projeto` no GitLab).
    pub path: String,
}

impl RepoRef {
    /// Interpreta `dono/repo`, URLs do GitHub, `codeberg.org/dono/repo`, `https://gitlab.com/grupo/projeto`...
    /// `None` para forjas desconhecidas ou caminhos incompletos.
    pub fn parse(input: &str) -> Option<RepoRef> {
        let clean = github::normalize_repo_name(input);
        let (first, rest) = clean.split_once('/')?;
        if !first.contains('.') {
            let (owner, repo) = (first, rest);
            if owner.is_empty() || repo.is_empty() || repo.contains('/') {
                return None;
            }
            return Some(RepoRef {
                kind: ForgeKind::GitHub,
                host: "github.com".into(),
                path: clean,
            });
        }
        let host = first.to_lowercase();
        let kind = if GITEA_HOSTS.contains(&host.as_str()) {
            ForgeKind::Gitea
        } else if GITLAB_HOSTS.contains(&host.as_str()) || host.starts_with("gitlab.") {
            ForgeKind::GitLab
        } else {
            return None;
        };
        let segments: Vec<&str> = rest.split('/').filter(|s| !s.is_empty()).collect();
        let ok_len = match kind {
            ForgeKind::Gitea => segments.len() == 2,
            _ => segments.len() >= 2,
        };
        if !ok_len {
            return None;
        }
        Some(RepoRef {
            kind,
            host,
            path: segments.join("/"),
        })
    }

    /// Nome usado no catálogo.
    pub fn full_name(&self) -> String {
        match self.kind {
            ForgeKind::GitHub => self.path.clone(),
            _ => format!("{}/{}", self.host, self.path),
        }
    }
}

/// `true` quando o nome do catálogo é de outra forja (tem host na frente).
pub fn is_other_forge(full_name: &str) -> bool {
    full_name.split('/').next().is_some_and(|first| first.contains('.'))
}

/// Cliente único para todas as forjas.
pub struct Forge {
    gh: Github,
    http: Client,
}

impl Forge {
    pub fn new(github_token: Option<&str>) -> Result<Self> {
        let mut headers = header::HeaderMap::new();
        headers.insert(header::USER_AGENT, "gitstore/0.3".parse()?);
        headers.insert(header::ACCEPT, "application/json".parse()?);
        Ok(Self {
            gh: Github::new(github_token)?,
            http: Client::builder().default_headers(headers).build()?,
        })
    }

    pub fn has_token(&self) -> bool {
        self.gh.has_token
    }

    fn parse(full_name: &str) -> Result<RepoRef> {
        RepoRef::parse(full_name).ok_or_else(|| {
            anyhow!(
                "'{full_name}' não é reconhecido. Use dono/repo (GitHub), codeberg.org/dono/repo ou gitlab.com/grupo/projeto."
            )
        })
    }

    async fn get_json(&self, url: &str) -> Result<Option<Value>> {
        let resp = self
            .http
            .get(url)
            .send()
            .await
            .with_context(|| format!("falha ao acessar {url}"))?;
        if resp.status() == StatusCode::NOT_FOUND {
            return Ok(None);
        }
        if !resp.status().is_success() {
            bail!("{url} respondeu HTTP {}", resp.status());
        }
        Ok(Some(resp.json().await?))
    }

    pub async fn search_repos(&self, query: &str, page: u32) -> Result<Vec<Repo>> {
        self.gh.search_repos(query, page).await
    }

    pub async fn repo(&self, full_name: &str) -> Result<Repo> {
        let r = Self::parse(full_name)?;
        match r.kind {
            ForgeKind::GitHub => self.gh.repo(&r.path).await,
            ForgeKind::Gitea => {
                let url = format!("https://{}/api/v1/repos/{}", r.host, r.path);
                let v = self
                    .get_json(&url)
                    .await?
                    .ok_or_else(|| anyhow!("repositório '{}' não encontrado", r.full_name()))?;
                gitea_repo_from_json(&r, &v)
            }
            ForgeKind::GitLab => {
                let id = urlencode_path(&r.path);
                let url = format!("https://{}/api/v4/projects/{id}?license=true", r.host);
                let v = self
                    .get_json(&url)
                    .await?
                    .ok_or_else(|| anyhow!("projeto '{}' não encontrado", r.full_name()))?;
                let langs = self
                    .get_json(&format!("https://{}/api/v4/projects/{id}/languages", r.host))
                    .await
                    .ok()
                    .flatten();
                gitlab_repo_from_json(&r, &v, langs.as_ref())
            }
        }
    }

    pub async fn releases(&self, full_name: &str, per_page: u32) -> Result<Vec<Release>> {
        let r = Self::parse(full_name)?;
        match r.kind {
            ForgeKind::GitHub => self.gh.releases(&r.path, per_page).await,
            ForgeKind::Gitea => {
                let url = format!("https://{}/api/v1/repos/{}/releases?limit={per_page}", r.host, r.path);
                let v = self.get_json(&url).await?.unwrap_or(Value::Array(vec![]));
                Ok(gitea_releases_from_json(&v)
                    .into_iter()
                    .filter(|x| !x.draft)
                    .collect())
            }
            ForgeKind::GitLab => {
                let url = format!(
                    "https://{}/api/v4/projects/{}/releases?per_page={per_page}",
                    r.host,
                    urlencode_path(&r.path)
                );
                let v = self.get_json(&url).await?.unwrap_or(Value::Array(vec![]));
                Ok(gitlab_releases_from_json(&v))
            }
        }
    }

    pub async fn latest_release(&self, full_name: &str) -> Result<Option<Release>> {
        let r = Self::parse(full_name)?;
        match r.kind {
            ForgeKind::GitHub => self.gh.latest_release(&r.path).await,
            ForgeKind::Gitea => {
                let url = format!("https://{}/api/v1/repos/{}/releases/latest", r.host, r.path);
                Ok(self.get_json(&url).await?.map(|v| gitea_release_from_json(&v)))
            }
            ForgeKind::GitLab => Ok(self
                .releases(full_name, 10)
                .await?
                .into_iter()
                .find(|x| !x.prerelease)),
        }
    }

    pub async fn release_by_tag(&self, full_name: &str, tag: &str) -> Result<Release> {
        let r = Self::parse(full_name)?;
        match r.kind {
            ForgeKind::GitHub => self.gh.release_by_tag(&r.path, tag).await,
            _ => self
                .releases(full_name, 50)
                .await?
                .into_iter()
                .find(|x| x.tag_name == tag)
                .ok_or_else(|| anyhow!("versão '{tag}' de {full_name} não encontrada")),
        }
    }

    /// Contribuidores (só o GitHub informa pela paginação).
    pub async fn contributors(&self, full_name: &str) -> Result<Option<u64>> {
        let r = Self::parse(full_name)?;
        match r.kind {
            ForgeKind::GitHub => self.gh.contributors(&r.path).await,
            _ => Ok(None),
        }
    }

    /// Atestados de build existem só no GitHub; nas outras forjas a resposta é "não consultado".
    pub async fn has_attestation(&self, full_name: &str, digest: &str) -> Result<bool> {
        let r = Self::parse(full_name)?;
        match r.kind {
            ForgeKind::GitHub => self.gh.has_attestation(&r.path, digest).await,
            _ => bail!("atestados de build só existem no GitHub"),
        }
    }
}

fn urlencode_path(path: &str) -> String {
    path.replace('/', "%2F")
}

fn str_of(v: &Value, key: &str) -> String {
    v[key].as_str().unwrap_or_default().to_string()
}

fn topics_of(v: &Value) -> Vec<String> {
    v["topics"]
        .as_array()
        .map(|a| a.iter().filter_map(|t| t.as_str().map(String::from)).collect())
        .unwrap_or_default()
}

pub fn gitea_repo_from_json(r: &RepoRef, v: &Value) -> Result<Repo> {
    if !v.is_object() {
        bail!("resposta inesperada da forja para {}", r.full_name());
    }
    let license = v["licenses"]
        .as_array()
        .and_then(|a| a.first())
        .and_then(|l| l.as_str())
        .map(|s| License {
            spdx_id: Some(s.to_string()),
        });
    let updated = str_of(v, "updated_at");
    Ok(Repo {
        full_name: r.full_name(),
        description: v["description"].as_str().filter(|s| !s.is_empty()).map(String::from),
        html_url: str_of(v, "html_url"),
        stargazers_count: v["stars_count"].as_u64().unwrap_or(0),
        fork: v["fork"].as_bool().unwrap_or(false),
        archived: v["archived"].as_bool().unwrap_or(false),
        created_at: str_of(v, "created_at"),
        pushed_at: (!updated.is_empty()).then_some(updated),
        license,
        topics: topics_of(v),
        has_issues: v["has_issues"].as_bool().unwrap_or(true),
        language: v["language"].as_str().filter(|s| !s.is_empty()).map(String::from),
    })
}

pub fn gitea_release_from_json(v: &Value) -> Release {
    let assets = v["assets"]
        .as_array()
        .map(|a| {
            a.iter()
                .map(|x| Asset {
                    name: str_of(x, "name"),
                    size: x["size"].as_u64().unwrap_or(0),
                    browser_download_url: str_of(x, "browser_download_url"),
                    digest: None,
                })
                .collect()
        })
        .unwrap_or_default();
    Release {
        tag_name: str_of(v, "tag_name"),
        published_at: v["published_at"].as_str().map(String::from),
        prerelease: v["prerelease"].as_bool().unwrap_or(false),
        draft: v["draft"].as_bool().unwrap_or(false),
        assets,
    }
}

pub fn gitea_releases_from_json(v: &Value) -> Vec<Release> {
    v.as_array()
        .map(|a| a.iter().map(gitea_release_from_json).collect())
        .unwrap_or_default()
}

pub fn gitlab_repo_from_json(r: &RepoRef, v: &Value, languages: Option<&Value>) -> Result<Repo> {
    if !v.is_object() {
        bail!("resposta inesperada do GitLab para {}", r.full_name());
    }
    // Linguagem principal = maior porcentagem em /languages
    let language = languages.and_then(|l| l.as_object()).and_then(|m| {
        m.iter()
            .filter_map(|(k, p)| p.as_f64().map(|p| (k.clone(), p)))
            .max_by(|a, b| a.1.partial_cmp(&b.1).unwrap_or(std::cmp::Ordering::Equal))
            .map(|(k, _)| k)
    });
    let license = v["license"]["nickname"]
        .as_str()
        .or_else(|| v["license"]["key"].as_str())
        .map(|s| License {
            spdx_id: Some(s.to_string()),
        });
    let activity = str_of(v, "last_activity_at");
    Ok(Repo {
        full_name: r.full_name(),
        description: v["description"].as_str().filter(|s| !s.is_empty()).map(String::from),
        html_url: str_of(v, "web_url"),
        stargazers_count: v["star_count"].as_u64().unwrap_or(0),
        fork: v["forked_from_project"].is_object(),
        archived: v["archived"].as_bool().unwrap_or(false),
        created_at: str_of(v, "created_at"),
        pushed_at: (!activity.is_empty()).then_some(activity),
        license,
        topics: topics_of(v),
        has_issues: v["issues_enabled"].as_bool().unwrap_or(true),
        language,
    })
}

pub fn gitlab_releases_from_json(v: &Value) -> Vec<Release> {
    v.as_array()
        .map(|a| {
            a.iter()
                .map(|rel| {
                    let assets = rel["assets"]["links"]
                        .as_array()
                        .map(|links| {
                            links
                                .iter()
                                .map(|l| {
                                    let url = l["direct_asset_url"]
                                        .as_str()
                                        .or_else(|| l["url"].as_str())
                                        .unwrap_or_default()
                                        .to_string();
                                    Asset {
                                        name: str_of(l, "name"),
                                        size: 0,
                                        browser_download_url: url,
                                        digest: None,
                                    }
                                })
                                .collect()
                        })
                        .unwrap_or_default();
                    Release {
                        tag_name: str_of(rel, "tag_name"),
                        published_at: rel["released_at"].as_str().map(String::from),
                        prerelease: rel["upcoming_release"].as_bool().unwrap_or(false),
                        draft: false,
                        assets,
                    }
                })
                .collect()
        })
        .unwrap_or_default()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reconhece_github_codeberg_e_gitlab() {
        let gh = RepoRef::parse("https://github.com/obsproject/obs-studio/releases").unwrap();
        assert_eq!((gh.kind, gh.full_name().as_str()), (ForgeKind::GitHub, "obsproject/obs-studio"));

        let cb = RepoRef::parse("https://codeberg.org/forgejo/forgejo.git").unwrap();
        assert_eq!(cb.kind, ForgeKind::Gitea);
        assert_eq!(cb.full_name(), "codeberg.org/forgejo/forgejo");

        let gl = RepoRef::parse("gitlab.com/inkscape/inkscape/-/releases").unwrap();
        assert_eq!(gl.kind, ForgeKind::GitLab);
        assert_eq!(gl.full_name(), "gitlab.com/inkscape/inkscape");

        let sub = RepoRef::parse("https://gitlab.gnome.org/World/Shortwave").unwrap();
        assert_eq!(sub.kind, ForgeKind::GitLab);

        assert!(RepoRef::parse("exemplo.com/a/b").is_none(), "forja desconhecida");
        assert!(RepoRef::parse("codeberg.org/so-dono").is_none());
        assert!(RepoRef::parse("sem-barra").is_none());
        assert!(is_other_forge("codeberg.org/a/b"));
        assert!(!is_other_forge("a/b"));
    }

    #[test]
    fn converte_resposta_do_gitea() {
        let r = RepoRef::parse("codeberg.org/dono/app").unwrap();
        let v: Value = serde_json::from_str(
            r#"{"full_name":"dono/app","description":"Um app","html_url":"https://codeberg.org/dono/app",
                "stars_count":42,"fork":false,"archived":false,"created_at":"2020-01-01T00:00:00Z",
                "updated_at":"2026-10-01T00:00:00Z","has_issues":true,"language":"Rust",
                "topics":["jogo"],"licenses":["GPL-3.0-only"]}"#,
        )
        .unwrap();
        let repo = gitea_repo_from_json(&r, &v).unwrap();
        assert_eq!(repo.full_name, "codeberg.org/dono/app");
        assert_eq!(repo.stargazers_count, 42);
        assert_eq!(repo.language.as_deref(), Some("Rust"));
        assert_eq!(repo.license.unwrap().spdx_id.as_deref(), Some("GPL-3.0-only"));

        let rel = gitea_release_from_json(
            &serde_json::from_str(
                r#"{"tag_name":"v1.0","published_at":"2026-09-01T00:00:00Z","prerelease":false,"draft":false,
                    "assets":[{"name":"app-1.0-windows-x86_64.zip","size":1000,
                               "browser_download_url":"https://codeberg.org/dono/app/releases/download/v1.0/app.zip"}]}"#,
            )
            .unwrap(),
        );
        assert_eq!(rel.tag_name, "v1.0");
        assert_eq!(rel.installable_assets().len(), 1);
        assert!(rel.assets[0].digest.is_none(), "Gitea não publica SHA-256");
    }

    #[test]
    fn converte_resposta_do_gitlab() {
        let r = RepoRef::parse("gitlab.com/grupo/projeto").unwrap();
        let v: Value = serde_json::from_str(
            r#"{"path_with_namespace":"grupo/projeto","description":"","web_url":"https://gitlab.com/grupo/projeto",
                "star_count":7,"archived":false,"created_at":"2019-05-01T00:00:00Z",
                "last_activity_at":"2026-10-20T00:00:00Z","issues_enabled":false,"topics":[],
                "license":{"key":"mit","nickname":null},"forked_from_project":{"id":1}}"#,
        )
        .unwrap();
        let langs: Value = serde_json::from_str(r#"{"C++":20.5,"Rust":70.1,"Shell":9.4}"#).unwrap();
        let repo = gitlab_repo_from_json(&r, &v, Some(&langs)).unwrap();
        assert_eq!(repo.language.as_deref(), Some("Rust"));
        assert!(repo.fork);
        assert!(!repo.has_issues);
        assert!(repo.description.is_none());
        assert_eq!(repo.license.unwrap().spdx_id.as_deref(), Some("mit"));

        let rels = gitlab_releases_from_json(
            &serde_json::from_str(
                r#"[{"tag_name":"v2.0","released_at":"2026-10-01T00:00:00Z","upcoming_release":false,
                     "assets":{"links":[{"name":"projeto-2.0-win64-setup.exe","url":"https://x/a",
                                         "direct_asset_url":"https://gitlab.com/grupo/projeto/-/releases/v2.0/downloads/setup.exe"}]}}]"#,
            )
            .unwrap(),
        );
        assert_eq!(rels.len(), 1);
        assert!(rels[0].assets[0].browser_download_url.contains("/downloads/"));
        assert_eq!(rels[0].installable_assets().len(), 1);
    }
}
