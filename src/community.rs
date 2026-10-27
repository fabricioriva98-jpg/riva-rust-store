//! Central de bate-papo pelo GitHub Discussions (decisão do Fabricio em 24/10/2026): conversas de cada
//! app (nas Discussions do repositório dele) e uma comunidade geral da RRS (repositório configurável).
//! A RRS lê as conversas pela API GraphQL com o login do GitHub; responder e abrir conversa nova é no
//! site do GitHub (assim o login não precisa de permissão de escrita em repositórios).

use anyhow::{bail, Context, Result};
use serde_json::{json, Value};

pub const SETTING_COMMUNITY_REPO: &str = "comunidade_repo";
const MAX_BODY: usize = 4000;

#[derive(Debug, Clone, PartialEq)]
pub struct DiscussionSummary {
    pub number: u64,
    pub title: String,
    pub url: String,
    pub author: String,
    pub updated_at: String,
    pub comments: u64,
    pub category: String,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Comment {
    pub author: String,
    pub body: String,
    pub created_at: String,
}

#[derive(Debug, Clone, PartialEq)]
pub struct DiscussionThread {
    pub title: String,
    pub url: String,
    pub author: String,
    pub body: String,
    pub comments: Vec<Comment>,
}

/// `dono/repo` do GitHub (as Discussions só existem lá).
pub fn split_repo(repo: &str) -> Result<(String, String)> {
    let r = crate::forge::RepoRef::parse(repo).context("repositório inválido")?;
    if r.kind != crate::forge::ForgeKind::GitHub {
        bail!("a central de bate-papo usa o GitHub Discussions; {repo} não está no GitHub");
    }
    let (o, n) = r.path.split_once('/').context("repositório inválido")?;
    Ok((o.to_string(), n.to_string()))
}

pub fn discussions_url(repo: &str) -> String {
    format!("https://github.com/{repo}/discussions")
}

pub fn new_discussion_url(repo: &str) -> String {
    format!("https://github.com/{repo}/discussions/new/choose")
}

pub fn list_query(owner: &str, name: &str) -> Value {
    json!({
        "query": "query($owner:String!,$name:String!){repository(owner:$owner,name:$name){hasDiscussionsEnabled discussions(first:25,orderBy:{field:UPDATED_AT,direction:DESC}){nodes{number title url updatedAt author{login} comments{totalCount} category{name}}}}}",
        "variables": {"owner": owner, "name": name},
    })
}

pub fn thread_query(owner: &str, name: &str, number: u64) -> Value {
    json!({
        "query": "query($owner:String!,$name:String!,$number:Int!){repository(owner:$owner,name:$name){discussion(number:$number){title url body author{login} comments(first:50){nodes{body createdAt author{login}}}}}}",
        "variables": {"owner": owner, "name": name, "number": number},
    })
}

fn clip(s: &str, max: usize) -> String {
    if s.chars().count() <= max {
        s.to_string()
    } else {
        format!("{}…", s.chars().take(max).collect::<String>())
    }
}

fn author(v: &Value) -> String {
    v["author"]["login"].as_str().unwrap_or("(conta apagada)").to_string()
}

fn graphql_errors(v: &Value) -> Result<()> {
    if let Some(errs) = v["errors"].as_array().filter(|e| !e.is_empty()) {
        let msg = errs[0]["message"].as_str().unwrap_or("erro na consulta");
        bail!("GitHub: {msg}");
    }
    Ok(())
}

/// (Discussions ligadas?, conversas)
pub fn parse_list(v: &Value) -> Result<(bool, Vec<DiscussionSummary>)> {
    graphql_errors(v)?;
    let repo = &v["data"]["repository"];
    if repo.is_null() {
        bail!("repositório não encontrado");
    }
    let enabled = repo["hasDiscussionsEnabled"].as_bool().unwrap_or(false);
    let list = repo["discussions"]["nodes"]
        .as_array()
        .map(|nodes| {
            nodes
                .iter()
                .map(|n| DiscussionSummary {
                    number: n["number"].as_u64().unwrap_or(0),
                    title: clip(n["title"].as_str().unwrap_or(""), 200),
                    url: n["url"].as_str().unwrap_or("").to_string(),
                    author: author(n),
                    updated_at: n["updatedAt"].as_str().unwrap_or("").to_string(),
                    comments: n["comments"]["totalCount"].as_u64().unwrap_or(0),
                    category: n["category"]["name"].as_str().unwrap_or("").to_string(),
                })
                .filter(|d| d.number > 0)
                .collect()
        })
        .unwrap_or_default();
    Ok((enabled, list))
}

pub fn parse_thread(v: &Value) -> Result<DiscussionThread> {
    graphql_errors(v)?;
    let d = &v["data"]["repository"]["discussion"];
    if d.is_null() {
        bail!("conversa não encontrada");
    }
    Ok(DiscussionThread {
        title: clip(d["title"].as_str().unwrap_or(""), 200),
        url: d["url"].as_str().unwrap_or("").to_string(),
        author: author(d),
        body: clip(d["body"].as_str().unwrap_or(""), MAX_BODY),
        comments: d["comments"]["nodes"]
            .as_array()
            .map(|c| {
                c.iter()
                    .map(|n| Comment {
                        author: author(n),
                        body: clip(n["body"].as_str().unwrap_or(""), MAX_BODY),
                        created_at: n["createdAt"].as_str().unwrap_or("").to_string(),
                    })
                    .collect()
            })
            .unwrap_or_default(),
    })
}

async fn graphql(token: &str, body: &Value) -> Result<Value> {
    if token.trim().is_empty() {
        bail!("entre com o GitHub (Configurações › Conta) para ver as conversas");
    }
    let resp = reqwest::Client::builder()
        .user_agent("gitstore/0.3")
        .connect_timeout(std::time::Duration::from_secs(15))
        .timeout(std::time::Duration::from_secs(30))
        .build()?
        .post("https://api.github.com/graphql")
        .header("Authorization", format!("Bearer {}", token.trim()))
        .json(body)
        .send()
        .await
        .context("sem conexão com o GitHub")?;
    if resp.status() == reqwest::StatusCode::UNAUTHORIZED {
        bail!("login do GitHub inválido; entre de novo");
    }
    if !resp.status().is_success() {
        bail!("o GitHub respondeu HTTP {}", resp.status());
    }
    Ok(resp.json().await?)
}

pub async fn list_discussions(token: &str, repo: &str) -> Result<(bool, Vec<DiscussionSummary>)> {
    let (o, n) = split_repo(repo)?;
    parse_list(&graphql(token, &list_query(&o, &n)).await?)
}

pub async fn get_thread(token: &str, repo: &str, number: u64) -> Result<DiscussionThread> {
    let (o, n) = split_repo(repo)?;
    parse_thread(&graphql(token, &thread_query(&o, &n, number)).await?)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn consulta_usa_variaveis_e_nao_interpola_texto() {
        let q = list_query("dono\"){", "repo");
        assert_eq!(q["variables"]["owner"], "dono\"){");
        assert!(!q["query"].as_str().unwrap().contains("dono"));
    }

    #[test]
    fn le_a_lista_de_conversas() {
        let v = json!({"data": {"repository": {"hasDiscussionsEnabled": true, "discussions": {"nodes": [
            {"number": 12, "title": "Ideias para a 2.0", "url": "https://github.com/a/b/discussions/12",
             "updatedAt": "2026-10-20T10:00:00Z", "author": {"login": "fulano"}, "comments": {"totalCount": 3}, "category": {"name": "Ideas"}},
            {"number": 13, "title": "Conta apagada", "url": "u", "updatedAt": "", "author": null, "comments": {"totalCount": 0}, "category": {"name": "Q&A"}}
        ]}}}});
        let (enabled, list) = parse_list(&v).unwrap();
        assert!(enabled);
        assert_eq!(list.len(), 2);
        assert_eq!(list[0].comments, 3);
        assert_eq!(list[1].author, "(conta apagada)");
        let desligado = json!({"data": {"repository": {"hasDiscussionsEnabled": false, "discussions": {"nodes": []}}}});
        assert_eq!(parse_list(&desligado).unwrap(), (false, vec![]));
    }

    #[test]
    fn erros_do_graphql_viram_mensagem() {
        let v = json!({"data": {"repository": null}, "errors": [{"message": "Could not resolve to a Repository"}]});
        assert!(parse_list(&v).unwrap_err().to_string().contains("Could not resolve"));
    }

    #[test]
    fn le_uma_conversa_com_comentarios() {
        let v = json!({"data": {"repository": {"discussion": {"title": "Oi", "url": "u", "body": "Primeira mensagem",
            "author": {"login": "a"}, "comments": {"nodes": [{"body": "Resposta", "createdAt": "2026-10-21T00:00:00Z", "author": {"login": "b"}}]}}}}});
        let t = parse_thread(&v).unwrap();
        assert_eq!(t.comments.len(), 1);
        assert_eq!(t.comments[0].author, "b");
    }

    #[test]
    fn so_aceita_repositorios_do_github() {
        assert_eq!(split_repo("https://github.com/dono/repo").unwrap(), ("dono".into(), "repo".into()));
        assert!(split_repo("codeberg.org/a/b").is_err());
        assert_eq!(new_discussion_url("a/b"), "https://github.com/a/b/discussions/new/choose");
    }
}
