use std::time::Duration;

use serde::Serialize;
use serde_json::Value;

const API_BASE: &str = "https://api.zotero.org";
const PAGE_SIZE: usize = 100;
const MAX_ITEMS: usize = 5000;
const REQUEST_TIMEOUT_SECONDS: u64 = 20;
const SKIPPED_ITEM_TYPES: &[&str] = &["attachment", "note", "annotation"];

pub struct KeyInfo {
    pub user_id: String,
    pub username: String,
}

#[derive(Serialize)]
pub struct Library {
    pub id: String,
    pub name: String,
}

#[derive(Serialize)]
pub struct Collection {
    pub key: String,
    pub name: String,
    pub parent_key: Option<String>,
}

struct Page {
    body: Value,
    total: Option<usize>,
}

fn agent() -> ureq::Agent {
    ureq::AgentBuilder::new()
        .timeout(Duration::from_secs(REQUEST_TIMEOUT_SECONDS))
        .user_agent("Typst Desktop (Zotero sync)")
        .build()
}

fn is_safe_segment(value: &str) -> bool {
    !value.is_empty() && value.len() <= 64 && value.chars().all(|c| c.is_ascii_alphanumeric())
}

fn id_text(value: Option<&Value>) -> String {
    value.map(|id| id.to_string().trim_matches('"').to_string()).unwrap_or_default()
}

pub fn library_prefix(library_id: &str) -> Result<String, String> {
    let invalid = || "Unknown Zotero library".to_string();
    let (kind, id) = library_id.split_once(':').ok_or_else(invalid)?;
    if !is_safe_segment(id) {
        return Err(invalid());
    }
    match kind {
        "user" => Ok(format!("users/{id}")),
        "group" => Ok(format!("groups/{id}")),
        _ => Err(invalid()),
    }
}

pub fn check_collection_key(key: &str) -> Result<(), String> {
    if is_safe_segment(key) {
        Ok(())
    } else {
        Err("Unknown Zotero collection".to_string())
    }
}

fn get(path: &str, api_key: &str) -> Result<Page, String> {
    let response = agent()
        .get(&format!("{API_BASE}/{path}"))
        .set("Zotero-API-Version", "3")
        .set("Zotero-API-Key", api_key)
        .call();
    let response = match response {
        Ok(response) => response,
        Err(ureq::Error::Status(403, _)) => return Err("Zotero refused the API key. Reconnect Zotero in Settings.".to_string()),
        Err(ureq::Error::Status(404, _)) => return Err("That Zotero library or collection no longer exists".to_string()),
        Err(ureq::Error::Status(429, _)) => return Err("Zotero is rate limiting requests. Try again in a minute.".to_string()),
        Err(ureq::Error::Status(code, _)) => return Err(format!("Zotero answered with status {code}")),
        Err(_) => return Err("Could not reach Zotero".to_string()),
    };
    let total = response.header("Total-Results").and_then(|value| value.parse().ok());
    let body = response.into_json::<Value>().map_err(|_| "Zotero sent an unreadable answer".to_string())?;
    Ok(Page { body, total })
}

pub fn key_info(api_key: &str) -> Result<KeyInfo, String> {
    if !is_safe_segment(api_key) {
        return Err("That does not look like a Zotero API key".to_string());
    }
    let page = get(&format!("keys/{api_key}"), api_key).map_err(|_| "Zotero does not recognise that API key".to_string())?;
    let user_id = id_text(page.body.get("userID"));
    if !is_safe_segment(&user_id) {
        return Err("Zotero did not say which account the key belongs to".to_string());
    }
    let username = page.body.get("username").and_then(Value::as_str).unwrap_or_default().to_string();
    Ok(KeyInfo { user_id, username })
}

fn all_pages(path: &str, query: &str, api_key: &str) -> Result<Vec<Value>, String> {
    let mut results = Vec::new();
    let mut start = 0;
    loop {
        let separator = if query.is_empty() { "" } else { "&" };
        let page = get(&format!("{path}?{query}{separator}limit={PAGE_SIZE}&start={start}"), api_key)?;
        let entries = page.body.as_array().cloned().unwrap_or_default();
        let fetched = entries.len();
        results.extend(entries);
        start += fetched;
        if fetched < PAGE_SIZE || page.total.is_some_and(|total| start >= total) {
            break;
        }
        if start >= MAX_ITEMS {
            return Err(format!("Only libraries with up to {MAX_ITEMS} items can be synced. Pick a collection instead."));
        }
    }
    Ok(results)
}

pub fn libraries(user_id: &str, username: &str, api_key: &str) -> Result<Vec<Library>, String> {
    let mut libraries = vec![Library { id: format!("user:{user_id}"), name: format!("My Library ({username})") }];
    for group in all_pages(&format!("users/{user_id}/groups"), "", api_key)? {
        let id = id_text(group.get("id"));
        let name = group.pointer("/data/name").and_then(Value::as_str).unwrap_or("Group").to_string();
        if is_safe_segment(&id) {
            libraries.push(Library { id: format!("group:{id}"), name });
        }
    }
    Ok(libraries)
}

pub fn collections(library_id: &str, api_key: &str) -> Result<Vec<Collection>, String> {
    let prefix = library_prefix(library_id)?;
    let mut collections: Vec<Collection> = all_pages(&format!("{prefix}/collections"), "", api_key)?
        .into_iter()
        .filter_map(|entry| {
            let key = entry.get("key").and_then(Value::as_str)?.to_string();
            let name = entry.pointer("/data/name").and_then(Value::as_str).unwrap_or("Untitled").to_string();
            let parent_key = entry.pointer("/data/parentCollection").and_then(Value::as_str).map(str::to_string);
            Some(Collection { key, name, parent_key })
        })
        .collect();
    collections.sort_by(|first, second| first.name.to_lowercase().cmp(&second.name.to_lowercase()));
    Ok(collections)
}

pub fn items(library_id: &str, collection_key: Option<&str>, api_key: &str) -> Result<Vec<Value>, String> {
    let prefix = library_prefix(library_id)?;
    let path = match collection_key {
        Some(key) => {
            check_collection_key(key)?;
            format!("{prefix}/collections/{key}/items")
        }
        None => format!("{prefix}/items"),
    };
    Ok(all_pages(&path, "include=csljson,data", api_key)?
        .into_iter()
        .filter(|entry| {
            let item_type = entry.pointer("/data/itemType").and_then(Value::as_str).unwrap_or_default();
            !SKIPPED_ITEM_TYPES.contains(&item_type)
        })
        .filter_map(|entry| {
            let mut csl = entry.get("csljson").cloned()?;
            let citation_key = entry.pointer("/data/citationKey").and_then(Value::as_str).filter(|key| !key.is_empty());
            if let (Some(key), Some(object)) = (citation_key, csl.as_object_mut()) {
                object.insert("citation-key".to_string(), Value::String(key.to_string()));
            }
            Some(csl)
        })
        .collect())
}
