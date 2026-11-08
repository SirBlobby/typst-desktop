use std::collections::HashMap;
use std::path::Path;

use super::file_rules::FileRules;
use crate::workspace::{self, content_hash, is_text_file};

pub struct LocalFile {
    pub content: Vec<u8>,
    pub hash: String,
}

pub fn normalize_content(path: &str, content: Vec<u8>) -> Vec<u8> {
    if !is_text_file(path) {
        return content;
    }
    match String::from_utf8(content) {
        Ok(text) => text.into_bytes(),
        Err(error) => String::from_utf8_lossy(error.as_bytes()).into_owned().into_bytes(),
    }
}

pub fn load(project_dir: &Path, rules: &FileRules) -> Result<HashMap<String, LocalFile>, String> {
    let mut files = HashMap::new();
    for relative in workspace::collect_files(project_dir)? {
        if !rules.is_managed(&relative) {
            continue;
        }
        let full = workspace::project_file_path(project_dir, &relative)?;
        let content = std::fs::read(&full).map_err(|error| format!("Could not read {relative}: {error}"))?;
        let hash = content_hash(&content);
        files.insert(relative, LocalFile { content, hash });
    }
    Ok(files)
}

pub fn write(project_dir: &Path, path: &str, content: &[u8]) -> Result<(), String> {
    let full = workspace::project_file_path(project_dir, path)?;
    if let Some(directory) = full.parent() {
        std::fs::create_dir_all(directory).map_err(|error| error.to_string())?;
    }
    std::fs::write(&full, content).map_err(|error| format!("Could not write {path}: {error}"))
}

pub fn remove(project_dir: &Path, path: &str) -> Result<(), String> {
    let full = workspace::project_file_path(project_dir, path)?;
    if full.is_file() {
        std::fs::remove_file(&full).map_err(|error| format!("Could not remove {path}: {error}"))?;
    }
    let mut directory = full.parent();
    while let Some(current) = directory {
        if current == project_dir || std::fs::remove_dir(current).is_err() {
            break;
        }
        directory = current.parent();
    }
    Ok(())
}
