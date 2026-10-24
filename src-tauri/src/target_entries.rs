use std::path::PathBuf;
use tauri::{AppHandle, State};

use crate::{
    db::Store,
    workspace::{collect_files, project_file_path, resolve_target},
};

fn project_root(app: &AppHandle, store: &Store, path: &str) -> Result<PathBuf, String> {
    let target = resolve_target(app, store, path)?;
    if target.standalone {
        return Err("A standalone document has no project files".to_string());
    }
    Ok(target.root)
}

fn is_within(path: &str, entry: &str) -> bool {
    path == entry || path.starts_with(&format!("{entry}/"))
}

fn moved_path(path: &str, from: &str, to: &str) -> Option<String> {
    if path == from {
        return Some(to.to_string());
    }
    path.strip_prefix(&format!("{from}/"))
        .map(|rest| format!("{to}/{rest}"))
}

fn is_entrypoint_candidate(path: &str) -> bool {
    let lower = path.to_lowercase();
    lower.ends_with(".typ") || lower.ends_with(".tex")
}

fn entrypoint_after_removal(current: &str, remaining_paths: &[String]) -> String {
    let mut candidates: Vec<&String> = remaining_paths
        .iter()
        .filter(|path| is_entrypoint_candidate(path))
        .collect();
    candidates.sort_by_key(|path| {
        let is_main = path.eq_ignore_ascii_case("main.typ") || path.eq_ignore_ascii_case("main.tex");
        (!is_main, path.matches('/').count(), path.to_lowercase())
    });
    candidates
        .first()
        .map(|path| path.to_string())
        .unwrap_or_else(|| current.to_string())
}

#[tauri::command]
pub fn rename_target_entry(
    app: AppHandle,
    store: State<'_, Store>,
    path: String,
    from: String,
    to: String,
) -> Result<String, String> {
    let root = project_root(&app, &store, &path)?;
    let mut meta = store.meta(&path)?;
    let to = to.trim().trim_matches('/').to_string();

    if to.is_empty() {
        return Err("Path cannot be empty".to_string());
    }
    if to == from {
        return Ok(meta.entrypoint);
    }
    if to.starts_with(&format!("{from}/")) {
        return Err("A folder cannot be moved inside itself".to_string());
    }

    let source = project_file_path(&root, &from)?;
    let destination = project_file_path(&root, &to)?;
    if !source.exists() {
        return Err(format!("'{from}' does not exist"));
    }
    if destination.exists() {
        return Err(format!("'{to}' already exists"));
    }
    if let Some(parent) = destination.parent() {
        std::fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    }
    std::fs::rename(&source, &destination).map_err(|e| e.to_string())?;

    if let Some(entrypoint) = moved_path(&meta.entrypoint, &from, &to) {
        meta.entrypoint = entrypoint;
        store.save_meta(&path, &meta)?;
    }
    Ok(meta.entrypoint)
}

#[tauri::command]
pub fn delete_target_entries(
    app: AppHandle,
    store: State<'_, Store>,
    path: String,
    files: Vec<String>,
) -> Result<String, String> {
    let root = project_root(&app, &store, &path)?;
    let mut meta = store.meta(&path)?;

    for file in &files {
        let full = project_file_path(&root, file)?;
        if full.is_dir() {
            std::fs::remove_dir_all(&full).map_err(|e| e.to_string())?;
        } else if full.exists() {
            std::fs::remove_file(&full).map_err(|e| e.to_string())?;
        }
    }

    let removed_entrypoint = files
        .iter()
        .any(|file| is_within(&meta.entrypoint, file));
    if removed_entrypoint {
        let remaining = collect_files(&root)?;
        meta.entrypoint = entrypoint_after_removal(&meta.entrypoint, &remaining);
        store.save_meta(&path, &meta)?;
    }
    Ok(meta.entrypoint)
}
