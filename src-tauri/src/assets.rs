use serde::Serialize;
use std::collections::{BTreeSet, HashMap};
use std::path::{Path, PathBuf};
use tauri::AppHandle;

use crate::db::Store;
use crate::workspace::workspace_root;

pub const ASSETS_DIR: &str = ".assets";

const FONT_EXTENSIONS: [&str; 4] = ["ttf", "otf", "ttc", "otc"];
const IMAGE_EXTENSIONS: [&str; 6] = ["png", "jpg", "jpeg", "gif", "svg", "webp"];

fn extension_of(name: &str) -> String {
    Path::new(name)
        .extension()
        .and_then(|ext| ext.to_str())
        .unwrap_or("")
        .to_lowercase()
}

pub fn is_font(name: &str) -> bool {
    FONT_EXTENSIONS.contains(&extension_of(name).as_str())
}

pub fn is_image(name: &str) -> bool {
    IMAGE_EXTENSIONS.contains(&extension_of(name).as_str())
}

pub fn assets_dir(app: &AppHandle, store: &Store) -> Result<PathBuf, String> {
    let dir = workspace_root(app, store)?.join(ASSETS_DIR);
    std::fs::create_dir_all(&dir)
        .map_err(|e| format!("Cannot create assets folder: {}", e))?;
    Ok(dir)
}

fn families_in(data: &[u8]) -> Vec<String> {
    let mut families = BTreeSet::new();
    for font in typst::text::Font::iter(typst::foundations::Bytes::new(data.to_vec())) {
        families.insert(font.info().family.clone());
    }
    families.into_iter().collect()
}

#[derive(Serialize)]
pub struct Asset {
    pub name: String,
    pub kind: String,
    pub size: u64,
    pub font_families: Vec<String>,
}

pub fn list_assets(app: &AppHandle, store: &Store) -> Result<Vec<Asset>, String> {
    let dir = assets_dir(app, store)?;
    let mut assets = Vec::new();

    for entry in std::fs::read_dir(&dir).map_err(|e| e.to_string())? {
        let entry = entry.map_err(|e| e.to_string())?;
        if !entry.path().is_file() {
            continue;
        }

        let name = entry.file_name().to_string_lossy().to_string();
        if name.starts_with('.') {
            continue;
        }

        let size = entry.metadata().map(|m| m.len()).unwrap_or(0);
        let font_families = if is_font(&name) {
            std::fs::read(entry.path())
                .map(|data| families_in(&data))
                .unwrap_or_default()
        } else {
            Vec::new()
        };

        assets.push(Asset {
            kind: if is_font(&name) {
                "font"
            } else if is_image(&name) {
                "image"
            } else {
                "file"
            }
            .to_string(),
            name,
            size,
            font_families,
        });
    }

    assets.sort_by(|a, b| a.name.to_lowercase().cmp(&b.name.to_lowercase()));
    Ok(assets)
}

pub fn font_families(files: &HashMap<String, Vec<u8>>) -> Vec<String> {
    let mut families = BTreeSet::new();

    for data in typst_assets::fonts() {
        for font in typst::text::Font::iter(typst::foundations::Bytes::new(data)) {
            families.insert(font.info().family.clone());
        }
    }

    for (name, data) in files {
        if is_font(name) {
            for family in families_in(data) {
                families.insert(family);
            }
        }
    }

    families.into_iter().collect()
}

fn unique_destination(dir: &Path, name: &str) -> PathBuf {
    let candidate = dir.join(name);
    if !candidate.exists() {
        return candidate;
    }

    let stem = Path::new(name)
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("file")
        .to_string();
    let extension = Path::new(name)
        .extension()
        .and_then(|e| e.to_str())
        .map(|e| format!(".{}", e))
        .unwrap_or_default();

    for index in 2..1000 {
        let candidate = dir.join(format!("{}-{}{}", stem, index, extension));
        if !candidate.exists() {
            return candidate;
        }
    }

    dir.join(name)
}

fn copy_tree(source: &Path, destination: &Path) -> Result<(), String> {
    std::fs::create_dir_all(destination).map_err(|e| e.to_string())?;

    for entry in std::fs::read_dir(source).map_err(|e| e.to_string())? {
        let entry = entry.map_err(|e| e.to_string())?;
        let name = entry.file_name();
        let from = entry.path();
        let to = destination.join(&name);

        if from.is_dir() {
            copy_tree(&from, &to)?;
        } else {
            std::fs::copy(&from, &to).map_err(|e| e.to_string())?;
        }
    }

    Ok(())
}

pub fn import_paths(sources: &[String], destination: &Path) -> Result<Vec<String>, String> {
    std::fs::create_dir_all(destination).map_err(|e| e.to_string())?;

    let mut imported = Vec::new();

    for source in sources {
        let source_path = Path::new(source);
        let name = source_path
            .file_name()
            .map(|n| n.to_string_lossy().to_string())
            .ok_or_else(|| format!("'{}' has no name", source))?;

        if source_path.is_dir() {
            let target = unique_destination(destination, &name);
            copy_tree(source_path, &target)?;
        } else if source_path.is_file() {
            let target = unique_destination(destination, &name);
            std::fs::copy(source_path, &target)
                .map_err(|e| format!("Could not import '{}': {}", name, e))?;
        } else {
            return Err(format!("'{}' could not be read", source));
        }

        imported.push(name);
    }

    Ok(imported)
}

pub fn import_files(sources: &[String], destination: &Path) -> Result<Vec<String>, String> {
    std::fs::create_dir_all(destination).map_err(|e| e.to_string())?;

    let mut imported = Vec::new();

    for source in sources {
        let source_path = Path::new(source);
        if !source_path.is_file() {
            return Err(format!("'{}' is not a file", source));
        }

        let name = source_path
            .file_name()
            .map(|n| n.to_string_lossy().to_string())
            .ok_or_else(|| format!("'{}' has no file name", source))?;

        let target = unique_destination(destination, &name);
        std::fs::copy(source_path, &target)
            .map_err(|e| format!("Could not import '{}': {}", name, e))?;

        imported.push(
            target
                .file_name()
                .map(|n| n.to_string_lossy().to_string())
                .unwrap_or(name),
        );
    }

    Ok(imported)
}

pub fn delete_asset(app: &AppHandle, store: &Store, name: &str) -> Result<(), String> {
    if name.contains('/') || name.contains('\\') || name.contains("..") {
        return Err("Invalid asset name".to_string());
    }
    let path = assets_dir(app, store)?.join(name);
    std::fs::remove_file(path).map_err(|e| e.to_string())
}

pub fn asset_files(app: &AppHandle, store: &Store) -> HashMap<String, Vec<u8>> {
    let Ok(dir) = assets_dir(app, store) else {
        return HashMap::new();
    };

    let mut files = HashMap::new();
    let Ok(entries) = std::fs::read_dir(&dir) else {
        return files;
    };

    for entry in entries.filter_map(|entry| entry.ok()) {
        if !entry.path().is_file() {
            continue;
        }
        let name = entry.file_name().to_string_lossy().to_string();
        if name.starts_with('.') {
            continue;
        }
        if let Ok(data) = std::fs::read(entry.path()) {
            files.insert(name, data);
        }
    }

    files
}
