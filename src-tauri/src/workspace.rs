use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use tauri::{AppHandle, Manager};

use crate::db::Store;
use walkdir::WalkDir;

pub const PROJECT_META_FILE: &str = ".typst-desktop.json";

const TEXT_EXTENSIONS: [&str; 10] = [
    "typ", "toml", "bib", "csl", "yml", "yaml", "json", "md", "txt", "csv",
];

pub fn is_text_file(path: &str) -> bool {
    Path::new(path)
        .extension()
        .and_then(|ext| ext.to_str())
        .map(|ext| TEXT_EXTENSIONS.contains(&ext.to_lowercase().as_str()))
        .unwrap_or(false)
}

pub fn content_hash(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}

#[derive(Serialize, Deserialize, Clone)]
pub struct Settings {
    pub workspace_root: String,
    pub server_url: String,
    #[serde(default)]
    pub device_token: Option<String>,
    #[serde(default)]
    pub account_email: Option<String>,
    #[serde(default)]
    pub account_username: Option<String>,
}

impl Settings {
    pub fn fallback(app: &AppHandle) -> Self {
        let home = app
            .path()
            .home_dir()
            .unwrap_or_else(|_| PathBuf::from("."));
        Settings {
            workspace_root: home.join("typst").to_string_lossy().to_string(),
            server_url: "http://localhost:3000".to_string(),
            device_token: None,
            account_email: None,
            account_username: None,
        }
    }
}

pub fn load_settings(app: &AppHandle, store: &Store) -> Result<Settings, String> {
    match store.settings()? {
        Some(settings) => Ok(settings),
        None => {
            let settings = Settings::fallback(app);
            store.save_settings(&settings)?;
            Ok(settings)
        }
    }
}

pub fn save_settings(store: &Store, settings: &Settings) -> Result<(), String> {
    store.save_settings(settings)
}

pub fn workspace_root(app: &AppHandle, store: &Store) -> Result<PathBuf, String> {
    let settings = load_settings(app, store)?;
    let root = PathBuf::from(&settings.workspace_root);
    std::fs::create_dir_all(&root)
        .map_err(|e| format!("Cannot create workspace directory: {}", e))?;
    Ok(root)
}

#[derive(Serialize, Deserialize, Clone)]
pub struct ProjectMeta {
    pub entrypoint: String,
    pub space_id: Option<String>,
    pub last_synced_at: Option<String>,
    pub base_hashes: HashMap<String, String>,
}

impl Default for ProjectMeta {
    fn default() -> Self {
        ProjectMeta {
            entrypoint: "main.typ".to_string(),
            space_id: None,
            last_synced_at: None,
            base_hashes: HashMap::new(),
        }
    }
}

pub fn workspace_path(app: &AppHandle, store: &Store, relative: &str) -> Result<PathBuf, String> {
    let root = workspace_root(app, store)?;
    if relative.is_empty() {
        return Ok(root);
    }
    project_file_path(&root, relative)
}

pub fn is_project_dir(path: &Path) -> bool {
    path.join(PROJECT_META_FILE).exists() || path.join("typst.toml").exists()
}

pub fn is_typst_file(path: &str) -> bool {
    path.to_lowercase().ends_with(".typ")
}

#[derive(Serialize)]
pub struct BrowseEntry {
    pub name: String,
    pub path: String,
    pub kind: String,
    pub size: u64,
    pub modified: Option<String>,
    pub space_id: Option<String>,
    pub last_synced_at: Option<String>,
    pub child_count: usize,
}

fn modified_at(path: &Path) -> Option<String> {
    let modified = std::fs::metadata(path).ok()?.modified().ok()?;
    let datetime: chrono::DateTime<chrono::Utc> = modified.into();
    Some(datetime.to_rfc3339())
}

pub fn browse(app: &AppHandle, store: &Store, relative: &str) -> Result<Vec<BrowseEntry>, String> {
    let dir = workspace_path(app, store, relative)?;
    if !dir.is_dir() {
        return Err(format!("'{}' is not a folder", relative));
    }

    let prefix = if relative.is_empty() {
        String::new()
    } else {
        format!("{}/", relative.trim_end_matches('/'))
    };

    let mut entries = Vec::new();

    for entry in std::fs::read_dir(&dir).map_err(|e| e.to_string())? {
        let entry = entry.map_err(|e| e.to_string())?;
        let name = entry.file_name().to_string_lossy().to_string();
        if name.starts_with('.') {
            continue;
        }

        let path = format!("{}{}", prefix, name);
        let full = entry.path();

        if full.is_dir() {
            let project = is_project_dir(&full) || store.has_project(&path)?;
            let meta = if project {
                Some(store.meta(&path)?)
            } else {
                None
            };
            let child_count = std::fs::read_dir(&full)
                .map(|children| {
                    children
                        .filter_map(|child| child.ok())
                        .filter(|child| {
                            !child.file_name().to_string_lossy().starts_with('.')
                        })
                        .count()
                })
                .unwrap_or(0);

            entries.push(BrowseEntry {
                name,
                path,
                kind: if project { "project" } else { "folder" }.to_string(),
                size: 0,
                modified: modified_at(&full),
                space_id: meta.as_ref().and_then(|m| m.space_id.clone()),
                last_synced_at: meta.as_ref().and_then(|m| m.last_synced_at.clone()),
                child_count,
            });
        } else {
            let kind = if is_typst_file(&name) { "document" } else { "file" };
            entries.push(BrowseEntry {
                name,
                path,
                kind: kind.to_string(),
                size: std::fs::metadata(&full).map(|m| m.len()).unwrap_or(0),
                modified: modified_at(&full),
                space_id: None,
                last_synced_at: None,
                child_count: 0,
            });
        }
    }

    entries.sort_by(|a, b| {
        let rank = |kind: &str| match kind {
            "project" => 0,
            "folder" => 1,
            "document" => 2,
            _ => 3,
        };
        rank(&a.kind)
            .cmp(&rank(&b.kind))
            .then_with(|| a.name.to_lowercase().cmp(&b.name.to_lowercase()))
    });

    Ok(entries)
}

pub struct Target {
    pub root: PathBuf,
    pub entrypoint: String,
    pub standalone: bool,
}

pub fn resolve_target(app: &AppHandle, store: &Store, path: &str) -> Result<Target, String> {
    let full = workspace_path(app, store, path)?;

    if full.is_dir() {
        let meta = store.meta(path)?;
        return Ok(Target {
            root: full,
            entrypoint: meta.entrypoint,
            standalone: false,
        });
    }

    if !full.is_file() {
        return Err(format!("'{}' does not exist", path));
    }

    let parent = full
        .parent()
        .ok_or("File has no parent folder")?
        .to_path_buf();
    let name = full
        .file_name()
        .map(|n| n.to_string_lossy().to_string())
        .ok_or("File has no name")?;

    Ok(Target {
        root: parent,
        entrypoint: name,
        standalone: true,
    })
}

pub fn read_target_files(
    app: &AppHandle,
    store: &Store,
    target: &Target,
) -> Result<HashMap<String, Vec<u8>>, String> {
    let mut map = crate::assets::asset_files(app, store);

    if !target.standalone {
        for (path, bytes) in read_all_files(&target.root)? {
            map.insert(path, bytes);
        }
        return Ok(map);
    }

    collect_loose_files(&target.root, "", &mut map)?;
    Ok(map)
}

fn collect_loose_files(
    dir: &Path,
    prefix: &str,
    map: &mut HashMap<String, Vec<u8>>,
) -> Result<(), String> {
    for entry in std::fs::read_dir(dir).map_err(|e| e.to_string())? {
        let entry = entry.map_err(|e| e.to_string())?;
        let name = entry.file_name().to_string_lossy().to_string();
        if name.starts_with('.') {
            continue;
        }

        let path = entry.path();
        let key = if prefix.is_empty() {
            name
        } else {
            format!("{}/{}", prefix, name)
        };

        if path.is_dir() {
            if is_project_dir(&path) {
                continue;
            }
            collect_loose_files(&path, &key, map)?;
        } else if let Ok(bytes) = std::fs::read(&path) {
            map.insert(key, bytes);
        }
    }

    Ok(())
}

pub fn project_file_path(project_dir: &Path, relative: &str) -> Result<PathBuf, String> {
    if relative.is_empty() {
        return Err("Path cannot be empty".to_string());
    }

    let mut resolved = project_dir.to_path_buf();
    for component in relative.replace('\\', "/").split('/') {
        if component.is_empty() || component == "." {
            continue;
        }
        if component == ".." {
            return Err("Path cannot escape the project".to_string());
        }
        resolved.push(component);
    }

    if !resolved.starts_with(project_dir) {
        return Err("Path cannot escape the project".to_string());
    }

    Ok(resolved)
}

pub fn relative_path(project_dir: &Path, path: &Path) -> Option<String> {
    path.strip_prefix(project_dir)
        .ok()
        .map(|rest| rest.to_string_lossy().replace('\\', "/"))
}

pub fn collect_files(project_dir: &Path) -> Result<Vec<String>, String> {
    let mut files = Vec::new();

    for entry in WalkDir::new(project_dir).into_iter().filter_map(|e| e.ok()) {
        if !entry.file_type().is_file() {
            continue;
        }
        let Some(relative) = relative_path(project_dir, entry.path()) else {
            continue;
        };
        if relative == PROJECT_META_FILE || relative.starts_with('.') {
            continue;
        }
        files.push(relative);
    }

    files.sort();
    Ok(files)
}

pub fn read_all_files(project_dir: &Path) -> Result<HashMap<String, Vec<u8>>, String> {
    let mut map = HashMap::new();
    for relative in collect_files(project_dir)? {
        let full = project_file_path(project_dir, &relative)?;
        let bytes = std::fs::read(&full).map_err(|e| e.to_string())?;
        map.insert(relative, bytes);
    }
    Ok(map)
}

#[derive(Serialize)]
pub struct FileEntry {
    pub path: String,
    pub name: String,
    pub is_text: bool,
    pub size: u64,
}

pub fn list_files(project_dir: &Path) -> Result<Vec<FileEntry>, String> {
    let mut entries = Vec::new();
    for relative in collect_files(project_dir)? {
        let full = project_file_path(project_dir, &relative)?;
        let size = std::fs::metadata(&full).map(|m| m.len()).unwrap_or(0);
        let name = relative
            .rsplit('/')
            .next()
            .unwrap_or(&relative)
            .to_string();
        entries.push(FileEntry {
            is_text: is_text_file(&relative),
            path: relative,
            name,
            size,
        });
    }
    Ok(entries)
}

pub const NEW_PROJECT_MAIN: &str = "= New Project\n\nStart writing here.\n";

pub fn manifest_for(name: &str) -> String {
    let slug: String = name
        .to_lowercase()
        .chars()
        .map(|c| if c.is_ascii_alphanumeric() { c } else { '-' })
        .collect();
    let slug = slug.trim_matches('-').to_string();
    let slug = if slug.is_empty() {
        "my-project".to_string()
    } else {
        slug
    };

    format!(
        "[package]\nname = \"{slug}\"\nversion = \"0.1.0\"\nentrypoint = \"main.typ\"\nauthors = [\"Anonymous\"]\nlicense = \"MIT\"\ndescription = \"\"\n"
    )
}
