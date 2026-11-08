use serde::{Deserialize, Serialize};
use tauri::{AppHandle, State};

use super::authors::{fill_authors, TemplateAuthor};
use super::files::{clean_path, template_files, TemplateFile};
use super::index;
use crate::db::Store;
use crate::join_path;
use crate::workspace::{self, ProjectMeta};

const MAX_TITLE_LENGTH: usize = 200;
const MAX_AUTHORS: usize = 50;

#[derive(Deserialize)]
pub struct TemplateRequest {
    pub name: String,
    pub version: String,
    pub title: String,
    pub parent: String,
    #[serde(default)]
    pub authors: Vec<TemplateAuthor>,
}

#[derive(Serialize)]
pub struct CreatedEntry {
    pub kind: &'static str,
    pub path: String,
    pub authors_filled: Option<bool>,
}

fn clean_title(title: &str) -> Result<String, String> {
    let title: String = title.trim().chars().take(MAX_TITLE_LENGTH).collect();
    if title.is_empty() {
        return Err("Give the new document a name".to_string());
    }
    if title.chars().any(|character| matches!(character, '/' | '\\' | ':' | '*' | '?' | '"' | '<' | '>' | '|')) {
        return Err("Names cannot contain / \\ : * ? \" < > or |".to_string());
    }
    Ok(title)
}

fn fill_entrypoint_authors(files: &mut [TemplateFile], entrypoint: &str, authors: &[TemplateAuthor]) -> bool {
    let Some(file) = files.iter_mut().find(|file| file.path == clean_path(entrypoint)) else {
        return false;
    };
    let Ok(source) = std::str::from_utf8(&file.data) else {
        return false;
    };
    match fill_authors(source, authors) {
        Some(filled) => {
            file.data = filled.into_bytes();
            true
        }
        None => false,
    }
}

fn write_document(app: &AppHandle, store: &Store, parent: &str, title: &str, file: &TemplateFile) -> Result<String, String> {
    let path = join_path(parent, &format!("{title}.typ"));
    let full = workspace::workspace_path(app, store, &path)?;
    if full.exists() {
        return Err(format!("'{title}.typ' already exists"));
    }
    if let Some(directory) = full.parent() {
        std::fs::create_dir_all(directory).map_err(|error| error.to_string())?;
    }
    std::fs::write(&full, &file.data).map_err(|error| error.to_string())?;
    Ok(path)
}

fn write_project(
    app: &AppHandle,
    store: &Store,
    parent: &str,
    title: &str,
    entrypoint: &str,
    files: &[TemplateFile],
) -> Result<String, String> {
    let path = join_path(parent, title);
    let full = workspace::workspace_path(app, store, &path)?;
    if full.exists() {
        return Err(format!("'{title}' already exists"));
    }
    std::fs::create_dir_all(&full).map_err(|error| error.to_string())?;

    for file in files {
        let destination = workspace::project_file_path(&full, &file.path)?;
        if let Some(directory) = destination.parent() {
            std::fs::create_dir_all(directory).map_err(|error| error.to_string())?;
        }
        std::fs::write(&destination, &file.data).map_err(|error| error.to_string())?;
    }

    workspace::ensure_project_marker(&full)?;
    store.save_meta(
        &path,
        &ProjectMeta {
            entrypoint: clean_path(entrypoint).to_string(),
            ..Default::default()
        },
    )?;
    Ok(path)
}

#[tauri::command(async)]
pub fn create_from_template(app: AppHandle, store: State<'_, Store>, request: TemplateRequest) -> Result<CreatedEntry, String> {
    let title = clean_title(&request.title)?;
    let template = index::find(&request.name, &request.version)?;
    let mut files = template_files(&template)?;

    let authors: Vec<TemplateAuthor> = request.authors.into_iter().take(MAX_AUTHORS).collect();
    let authors_filled = (!authors.is_empty()).then(|| fill_entrypoint_authors(&mut files, &template.entrypoint, &authors));

    let is_single_file = files.len() == 1 && files[0].path == clean_path(&template.entrypoint);
    if is_single_file {
        let path = write_document(&app, &store, &request.parent, &title, &files[0])?;
        return Ok(CreatedEntry { kind: "document", path, authors_filled });
    }

    let path = write_project(&app, &store, &request.parent, &title, &template.entrypoint, &files)?;
    Ok(CreatedEntry { kind: "project", path, authors_filled })
}
