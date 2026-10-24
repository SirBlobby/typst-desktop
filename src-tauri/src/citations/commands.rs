use std::collections::HashSet;
use std::path::{Path, PathBuf};

use serde::Serialize;
use sha2::{Digest, Sha256};
use tauri::{AppHandle, State};

use super::format::{render_entry, render_library, title_of, BibliographyFormat};
use super::lookup::{fetch_csl, parse_identifier};
use super::storage::{CitationSync, ZoteroAccount};
use super::zotero;
use crate::db::Store;
use crate::workspace::{collect_files, project_file_path, resolve_target};

const MAX_LABEL_LENGTH: usize = 200;

#[derive(Serialize)]
pub struct LookupResult {
    pub key: String,
    pub title: String,
    pub format: &'static str,
    pub entry: String,
}

#[derive(Serialize)]
pub struct AddedEntry {
    pub key: String,
    pub title: String,
    pub file_name: String,
    pub entry: String,
    pub created_file: bool,
}

#[derive(Serialize)]
pub struct ZoteroStatus {
    pub connected: bool,
    pub username: Option<String>,
}

fn is_bibliography_file(name: &str) -> bool {
    let lower = name.to_ascii_lowercase();
    [".bib", ".yml", ".yaml"].iter().any(|extension| lower.ends_with(extension))
}

fn target_root(app: &AppHandle, store: &Store, path: &str) -> Result<(PathBuf, bool), String> {
    let target = resolve_target(app, store, path)?;
    Ok((target.root, target.standalone))
}

fn bibliography_path(root: &Path, standalone: bool, file_name: &str) -> Result<PathBuf, String> {
    let file_name = file_name.trim();
    if !is_bibliography_file(file_name) {
        return Err("Bibliography files must end in .bib, .yml or .yaml".to_string());
    }
    if standalone && file_name.contains(['/', '\\']) {
        return Err("File names cannot contain slashes".to_string());
    }
    project_file_path(root, file_name)
}

fn read_existing(path: &Path) -> Result<Option<String>, String> {
    match std::fs::read(path) {
        Ok(bytes) => Ok(Some(String::from_utf8_lossy(&bytes).to_string())),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(None),
        Err(error) => Err(error.to_string()),
    }
}

fn write_file(path: &Path, text: &str) -> Result<(), String> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    }
    std::fs::write(path, text).map_err(|e| e.to_string())
}

fn label(value: &str) -> String {
    value.trim().chars().take(MAX_LABEL_LENGTH).collect()
}

fn new_sync_id(target_path: &str, file_name: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(target_path.as_bytes());
    hasher.update(file_name.as_bytes());
    hasher.update(chrono::Utc::now().timestamp_nanos_opt().unwrap_or_default().to_le_bytes());
    hasher.finalize().iter().take(12).map(|byte| format!("{byte:02x}")).collect()
}

fn require_account(store: &Store) -> Result<ZoteroAccount, String> {
    store.zotero_account()?.ok_or_else(|| "Connect your Zotero account in Settings first".to_string())
}

#[tauri::command(async)]
pub fn citation_lookup(identifier: String, format: String) -> Result<LookupResult, String> {
    let format = BibliographyFormat::parse(&format)?;
    let item = fetch_csl(&parse_identifier(&identifier)?)?;
    let rendered = render_entry(format, &item, &mut HashSet::new());
    Ok(LookupResult { key: rendered.key, title: title_of(&item), format: format.as_str(), entry: rendered.text })
}

#[tauri::command]
pub fn citation_files(app: AppHandle, store: State<'_, Store>, path: String) -> Result<Vec<String>, String> {
    let (root, standalone) = target_root(&app, &store, &path)?;
    let names: Vec<String> = if standalone {
        std::fs::read_dir(&root)
            .map_err(|e| e.to_string())?
            .filter_map(|entry| entry.ok())
            .filter(|entry| entry.path().is_file())
            .map(|entry| entry.file_name().to_string_lossy().to_string())
            .collect()
    } else {
        collect_files(&root)?
    };
    let mut names: Vec<String> = names.into_iter().filter(|name| is_bibliography_file(name)).collect();
    names.sort();
    Ok(names)
}

#[tauri::command(async)]
pub fn citation_add(
    app: AppHandle,
    store: State<'_, Store>,
    path: String,
    file_name: String,
    identifier: String,
) -> Result<AddedEntry, String> {
    let (root, standalone) = target_root(&app, &store, &path)?;
    let file_path = bibliography_path(&root, standalone, &file_name)?;
    let format = BibliographyFormat::for_file(&file_name)?;
    let item = fetch_csl(&parse_identifier(&identifier)?)?;

    let existing = read_existing(&file_path)?;
    let current = existing.clone().unwrap_or_default();
    let mut taken = format.existing_keys(&current);
    let rendered = render_entry(format, &item, &mut taken);
    let trimmed = current.trim_end();
    let updated = if trimmed.is_empty() { rendered.text.clone() } else { format!("{trimmed}\n\n{}", rendered.text) };
    write_file(&file_path, &updated)?;

    Ok(AddedEntry {
        key: rendered.key,
        title: title_of(&item),
        file_name: file_name.trim().to_string(),
        entry: rendered.text,
        created_file: existing.is_none(),
    })
}

#[tauri::command]
pub fn zotero_status(store: State<'_, Store>) -> Result<ZoteroStatus, String> {
    let account = store.zotero_account()?;
    Ok(ZoteroStatus { connected: account.is_some(), username: account.map(|account| account.username) })
}

#[tauri::command(async)]
pub fn zotero_connect(store: State<'_, Store>, api_key: String) -> Result<ZoteroStatus, String> {
    let api_key = api_key.trim().to_string();
    let info = zotero::key_info(&api_key)?;
    store.save_zotero_account(&ZoteroAccount { user_id: info.user_id, username: info.username.clone(), api_key })?;
    Ok(ZoteroStatus { connected: true, username: Some(info.username) })
}

#[tauri::command]
pub fn zotero_disconnect(store: State<'_, Store>) -> Result<(), String> {
    store.forget_zotero_account()
}

#[tauri::command(async)]
pub fn zotero_libraries(store: State<'_, Store>) -> Result<Vec<zotero::Library>, String> {
    let account = require_account(&store)?;
    zotero::libraries(&account.user_id, &account.username, &account.api_key)
}

#[tauri::command(async)]
pub fn zotero_collections(store: State<'_, Store>, library: String) -> Result<Vec<zotero::Collection>, String> {
    let account = require_account(&store)?;
    zotero::collections(&library, &account.api_key)
}

#[tauri::command]
pub fn citation_syncs(store: State<'_, Store>, path: String) -> Result<Vec<CitationSync>, String> {
    store.citation_syncs(&path)
}

fn run(app: &AppHandle, store: &Store, sync: &mut CitationSync) -> Result<(), String> {
    let account = require_account(store)?;
    let (root, standalone) = target_root(app, store, &sync.target_path)?;
    let file_path = bibliography_path(&root, standalone, &sync.file_name)?;
    let format = BibliographyFormat::for_file(&sync.file_name)?;
    let items = zotero::items(&sync.library_id, sync.collection_key.as_deref(), &account.api_key)?;
    let (body, count) = render_library(format, &items);
    write_file(&file_path, &format!("{}{body}", format.generated_notice()))?;

    let synced_at = chrono::Utc::now().format("%Y-%m-%d %H:%M:%S").to_string();
    store.record_citation_sync(&sync.id, count as i64, &synced_at)?;
    sync.item_count = count as i64;
    sync.last_synced_at = Some(synced_at);
    Ok(())
}

#[tauri::command(async)]
#[allow(clippy::too_many_arguments)]
pub fn citation_sync_create(
    app: AppHandle,
    store: State<'_, Store>,
    path: String,
    library_id: String,
    library_name: String,
    collection_key: Option<String>,
    collection_name: Option<String>,
    file_name: String,
) -> Result<CitationSync, String> {
    require_account(&store)?;
    let (root, standalone) = target_root(&app, &store, &path)?;
    bibliography_path(&root, standalone, &file_name)?;
    zotero::library_prefix(&library_id)?;
    let collection_key = collection_key.filter(|key| !key.trim().is_empty());
    if let Some(key) = &collection_key {
        zotero::check_collection_key(key)?;
    }

    let file_name = file_name.trim().to_string();
    let mut sync = CitationSync {
        id: new_sync_id(&path, &file_name),
        target_path: path,
        library_id,
        library_name: label(&library_name),
        collection_name: collection_key.as_ref().and(collection_name.as_deref()).map(label),
        collection_key,
        file_name,
        item_count: 0,
        last_synced_at: None,
    };
    store.insert_citation_sync(&sync)?;
    if let Err(error) = run(&app, &store, &mut sync) {
        let _ = store.delete_citation_sync(&sync.id);
        return Err(error);
    }
    Ok(sync)
}

#[tauri::command(async)]
pub fn citation_sync_run(app: AppHandle, store: State<'_, Store>, id: String) -> Result<CitationSync, String> {
    let mut sync = store.citation_sync(&id)?.ok_or_else(|| "Sync not found".to_string())?;
    run(&app, &store, &mut sync)?;
    Ok(sync)
}

#[tauri::command]
pub fn citation_sync_delete(store: State<'_, Store>, id: String) -> Result<(), String> {
    store.delete_citation_sync(&id)
}
