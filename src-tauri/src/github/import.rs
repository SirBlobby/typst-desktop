use std::collections::{HashMap, HashSet};

use serde::Serialize;

use super::file_rules::{check_file_count, MAX_FILE_BYTES};
use super::local_files::LocalFile;
use super::remote::{self, RemoteEntry, RemoteRepo};
use super::storage::GitLink;

#[derive(Serialize)]
pub struct PreviewFile {
    pub path: String,
    pub size: Option<u64>,
}

#[derive(Serialize)]
pub struct ImportPreview {
    pub commit: String,
    pub files: Vec<PreviewFile>,
    pub too_large: Vec<PreviewFile>,
    pub ignored_count: usize,
    pub total_bytes: u64,
    pub replaced: Vec<String>,
    pub removed: Vec<String>,
}

fn preview_file(entry: &RemoteEntry) -> PreviewFile {
    PreviewFile { path: entry.path.clone(), size: entry.size }
}

fn sorted(mut paths: Vec<String>) -> Vec<String> {
    paths.sort();
    paths
}

pub fn preview_import(
    local: &HashMap<String, LocalFile>,
    link: &GitLink,
    repo: &RemoteRepo,
    replace_project_files: bool,
) -> Result<(ImportPreview, Vec<RemoteEntry>), String> {
    let rules = link.rules();
    let commit = remote::head(repo)?;
    let listed = remote::list_files(repo, &commit)?;
    let listed_count = listed.len();
    let mut managed: Vec<RemoteEntry> = listed.into_iter().filter(|entry| rules.is_managed(&entry.path)).collect();
    let ignored_count = listed_count - managed.len();
    check_file_count(managed.len())?;
    managed.sort_by(|first, second| first.path.cmp(&second.path));

    let (too_large, importable): (Vec<&RemoteEntry>, Vec<&RemoteEntry>) =
        managed.iter().partition(|entry| entry.size.is_some_and(|size| size > MAX_FILE_BYTES));
    let remote_paths: HashSet<&str> = importable.iter().map(|entry| entry.path.as_str()).collect();

    let replaced: Vec<String> = local.keys().filter(|path| remote_paths.contains(path.as_str())).cloned().collect();
    let removed: Vec<String> = if replace_project_files {
        local
            .keys()
            .filter(|path| !managed.iter().any(|entry| &entry.path == *path))
            .cloned()
            .collect()
    } else {
        Vec::new()
    };

    let preview = ImportPreview {
        commit,
        files: importable.iter().map(|entry| preview_file(entry)).collect(),
        too_large: too_large.iter().map(|entry| preview_file(entry)).collect(),
        ignored_count,
        total_bytes: importable.iter().filter_map(|entry| entry.size).sum(),
        replaced: sorted(replaced),
        removed: sorted(removed),
    };
    Ok((preview, managed))
}
