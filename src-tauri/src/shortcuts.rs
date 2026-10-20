use serde::Serialize;
use std::path::PathBuf;
use tauri::State;

use crate::db::Store;
use crate::workspace::project_file_path;

pub const SHORTCUT_PREFIX: &str = "shortcut:";

#[derive(Serialize, Clone)]
pub struct Shortcut {
    pub id: String,
    pub name: String,
    pub path: String,
    pub browse_path: String,
    pub available: bool,
}

impl Shortcut {
    pub fn new(id: String, name: String, path: String) -> Self {
        Shortcut {
            browse_path: format!("{}{}", SHORTCUT_PREFIX, id),
            available: PathBuf::from(&path).is_dir(),
            id,
            name,
            path,
        }
    }
}

pub fn ensure_not_root(relative: &str) -> Result<(), String> {
    match relative.strip_prefix(SHORTCUT_PREFIX) {
        Some(rest) if !rest.contains('/') => {
            Err("A shortcut's folder cannot be changed from here. Remove the shortcut instead.".to_string())
        }
        _ => Ok(()),
    }
}

pub fn resolve(store: &Store, relative: &str) -> Option<Result<PathBuf, String>> {
    let rest = relative.strip_prefix(SHORTCUT_PREFIX)?;
    let (id, inner) = rest.split_once('/').unwrap_or((rest, ""));

    Some((|| {
        let shortcut = store
            .shortcut(id)?
            .ok_or_else(|| "This shortcut no longer exists".to_string())?;
        let root = PathBuf::from(&shortcut.path);
        if !root.is_dir() {
            return Err(format!("'{}' is not available. It may have been moved or deleted.", shortcut.path));
        }
        if inner.is_empty() {
            Ok(root)
        } else {
            project_file_path(&root, inner)
        }
    })())
}

#[tauri::command]
pub fn list_shortcuts(store: State<'_, Store>) -> Result<Vec<Shortcut>, String> {
    store.shortcuts()
}

#[tauri::command]
pub fn add_shortcut(store: State<'_, Store>, path: String) -> Result<Shortcut, String> {
    let folder = PathBuf::from(&path);
    if !folder.is_dir() {
        return Err(format!("'{}' is not a folder", path));
    }

    let name = folder
        .file_name()
        .map(|name| name.to_string_lossy().to_string())
        .unwrap_or_else(|| path.clone());
    let id = format!("{:x}", chrono::Utc::now().timestamp_millis());

    store.add_shortcut(&id, &name, &path)?;
    Ok(Shortcut::new(id, name, path))
}

#[tauri::command]
pub fn remove_shortcut(store: State<'_, Store>, id: String) -> Result<(), String> {
    store.remove_shortcut(&id)?;
    store.forget_folder_colors(&format!("{}{}", SHORTCUT_PREFIX, id))
}
