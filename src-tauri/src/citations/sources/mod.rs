mod doi;
mod pubmed;
mod semantic_scholar;

use std::time::Duration;

use serde_json::Value;

use super::identifiers::{Identifier, ARXIV_DOI_PREFIX};

const REQUEST_TIMEOUT_SECONDS: u64 = 15;
const USER_AGENT: &str = "Typst Desktop (citation lookup)";

pub struct JsonRequest {
    pub url: String,
    pub accept: &'static str,
    pub service: &'static str,
    pub api_key: Option<(&'static str, String)>,
}

impl JsonRequest {
    pub fn new(url: String, accept: &'static str, service: &'static str) -> Self {
        Self { url, accept, service, api_key: None }
    }
}

pub fn not_found() -> String {
    "No publication was found for that identifier".to_string()
}

fn unreadable(service: &str) -> String {
    format!("{service} sent an unreadable answer")
}

pub fn get_json(request: JsonRequest) -> Result<Option<Value>, String> {
    let agent = ureq::AgentBuilder::new()
        .timeout(Duration::from_secs(REQUEST_TIMEOUT_SECONDS))
        .user_agent(USER_AGENT)
        .redirects(5)
        .build();
    let mut call = agent.get(&request.url).set("Accept", request.accept);
    if let Some((header, key)) = &request.api_key {
        call = call.set(header, key);
    }
    match call.call() {
        Ok(response) => response.into_json::<Value>().map(Some).map_err(|_| unreadable(request.service)),
        Err(ureq::Error::Status(404 | 400 | 410, _)) => Ok(None),
        Err(ureq::Error::Status(429, _)) => Err(format!("{} is busy. Try again in a moment.", request.service)),
        Err(ureq::Error::Status(code, _)) => Err(format!("{} answered with status {code}", request.service)),
        Err(_) => Err(format!("Could not reach {}", request.service)),
    }
}

pub fn arxiv_doi(id: &str) -> String {
    format!("{ARXIV_DOI_PREFIX}{id}")
}

pub fn mark_arxiv(item: &mut Value, id: &str) {
    if let Some(object) = item.as_object_mut() {
        object.insert("arxiv".to_string(), Value::String(id.to_string()));
        object.entry("URL".to_string()).or_insert_with(|| Value::String(format!("https://arxiv.org/abs/{id}")));
    }
}

pub fn fetch_csl(identifier: &Identifier) -> Result<Value, String> {
    let item = match identifier {
        Identifier::Doi(doi) => doi::fetch(doi)?,
        Identifier::Arxiv(id) => {
            let mut item = doi::fetch(&arxiv_doi(id))?;
            mark_arxiv(&mut item, id);
            item
        }
        Identifier::Pubmed(id) => pubmed::fetch(pubmed::Database::Pubmed, id)?,
        Identifier::Pmc(id) => pubmed::fetch(pubmed::Database::Pmc, id)?,
        Identifier::SemanticScholar(id) => semantic_scholar::fetch(id)?,
    };
    if !item.is_object() {
        return Err(unreadable("The citation service"));
    }
    Ok(item)
}
