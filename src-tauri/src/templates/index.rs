use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use serde::{Deserialize, Serialize};

use super::{http_agent, unreachable_registry, REGISTRY_URL};

const REFRESH_AFTER: Duration = Duration::from_secs(6 * 60 * 60);

#[derive(Deserialize)]
struct IndexEntry {
    name: String,
    version: String,
    #[serde(default)]
    description: Option<String>,
    #[serde(default)]
    authors: Vec<String>,
    #[serde(default)]
    categories: Vec<String>,
    #[serde(default)]
    disciplines: Vec<String>,
    #[serde(default)]
    keywords: Vec<String>,
    #[serde(default)]
    template: Option<IndexTemplate>,
    #[serde(default, rename = "updatedAt")]
    updated_at: Option<i64>,
}

#[derive(Deserialize)]
struct IndexTemplate {
    path: String,
    entrypoint: String,
    #[serde(default)]
    thumbnail: Option<String>,
}

#[derive(Clone, Serialize)]
pub struct Template {
    pub name: String,
    pub version: String,
    pub description: String,
    pub authors: Vec<String>,
    pub categories: Vec<String>,
    pub disciplines: Vec<String>,
    pub keywords: Vec<String>,
    pub thumbnail_url: Option<String>,
    pub updated_at: Option<i64>,
    #[serde(skip)]
    pub template_path: String,
    #[serde(skip)]
    pub entrypoint: String,
}

struct CachedIndex {
    templates: Arc<Vec<Template>>,
    fetched_at: Instant,
}

static INDEX: Mutex<Option<CachedIndex>> = Mutex::new(None);

fn version_parts(version: &str) -> (u64, u64, u64) {
    let mut parts = version.split('.').map(|part| part.parse::<u64>().unwrap_or(0));
    (parts.next().unwrap_or(0), parts.next().unwrap_or(0), parts.next().unwrap_or(0))
}

fn author_name(author: &str) -> String {
    match author.find('<') {
        Some(position) => author[..position].trim().to_string(),
        None => author.trim().to_string(),
    }
}

fn thumbnail_url(name: &str, version: &str) -> String {
    format!("{REGISTRY_URL}/preview/thumbnails/{name}-{version}-small.webp")
}

fn latest_templates(entries: Vec<IndexEntry>) -> Vec<Template> {
    let mut latest: HashMap<String, IndexEntry> = HashMap::new();
    for entry in entries {
        let is_newer = latest
            .get(&entry.name)
            .is_none_or(|current| version_parts(&entry.version) > version_parts(&current.version));
        if is_newer {
            latest.insert(entry.name.clone(), entry);
        }
    }

    let mut templates: Vec<Template> = latest
        .into_values()
        .filter_map(|entry| {
            let template = entry.template?;
            Some(Template {
                thumbnail_url: template.thumbnail.as_ref().map(|_| thumbnail_url(&entry.name, &entry.version)),
                name: entry.name,
                version: entry.version,
                description: entry.description.unwrap_or_default(),
                authors: entry.authors.iter().map(|author| author_name(author)).filter(|name| !name.is_empty()).collect(),
                categories: entry.categories,
                disciplines: entry.disciplines,
                keywords: entry.keywords,
                updated_at: entry.updated_at,
                template_path: template.path,
                entrypoint: template.entrypoint,
            })
        })
        .collect();
    templates.sort_by(|left, right| left.name.cmp(&right.name));
    templates
}

fn download_index() -> Result<Vec<Template>, String> {
    let url = format!("{REGISTRY_URL}/preview/index.json");
    let entries = http_agent()
        .get(&url)
        .call()
        .map_err(|_| unreachable_registry())?
        .into_json::<Vec<IndexEntry>>()
        .map_err(|error| format!("Could not read the Typst Universe index: {error}"))?;
    Ok(latest_templates(entries))
}

pub fn templates() -> Result<Arc<Vec<Template>>, String> {
    let mut cached = INDEX.lock().map_err(|_| "Template index lock poisoned".to_string())?;
    let is_fresh = cached.as_ref().is_some_and(|index| index.fetched_at.elapsed() < REFRESH_AFTER);
    if !is_fresh {
        match download_index() {
            Ok(templates) => {
                *cached = Some(CachedIndex { templates: Arc::new(templates), fetched_at: Instant::now() });
            }
            Err(error) if cached.is_none() => return Err(error),
            Err(_) => {}
        }
    }
    cached.as_ref().map(|index| index.templates.clone()).ok_or_else(unreachable_registry)
}

pub fn find(name: &str, version: &str) -> Result<Template, String> {
    templates()?
        .iter()
        .find(|template| template.name == name && template.version == version)
        .cloned()
        .ok_or_else(|| "Template not found".to_string())
}

#[tauri::command(async)]
pub fn list_templates() -> Result<Vec<Template>, String> {
    Ok(templates()?.as_ref().clone())
}
