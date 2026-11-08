use std::collections::{HashMap, HashSet};
use std::path::Path;

use super::file_rules::MAX_FILE_BYTES;
use super::local_files;
use super::pull::now;
use super::remote::{self, ChangeKind, FileChange, RemoteRepo};
use super::storage::{GitLink, SnapshotEntry, SnapshotUpdate};
use super::sync_plan::local_changes;
use crate::db::Store;

const MAX_MESSAGE_LENGTH: usize = 5000;

pub struct PushOutcome {
    pub commit: String,
    pub pushed: Vec<String>,
}

fn clean_message(message: &str) -> Result<String, String> {
    let message = message.trim();
    if message.is_empty() {
        return Err("Write a short message that describes your changes".to_string());
    }
    if message.chars().count() > MAX_MESSAGE_LENGTH {
        return Err("The commit message is too long".to_string());
    }
    Ok(message.to_string())
}

pub fn push_project(
    store: &Store,
    project_dir: &Path,
    link: &GitLink,
    repo: &RemoteRepo,
    message: &str,
    paths: Option<Vec<String>>,
) -> Result<PushOutcome, String> {
    let message = clean_message(message)?;
    if repo.token.is_none() {
        return Err("Add an access token before pushing".to_string());
    }

    let head = remote::head(repo)?;
    let base_commit = link.base_commit.clone().unwrap_or_default();
    if head != base_commit {
        return Err("The repository has new commits. Pull them first.".to_string());
    }

    let rules = link.rules();
    let mut local = local_files::load(project_dir, &rules)?;
    let saved = store.git_snapshot(&link.project_path)?;
    let selected: Option<HashSet<String>> = paths.map(|paths| paths.into_iter().collect());
    let changes: Vec<(String, ChangeKind)> = local_changes(&local, &saved, &rules)
        .into_iter()
        .filter(|(path, _)| selected.as_ref().is_none_or(|selected| selected.contains(path)))
        .collect();
    if changes.is_empty() {
        return Err("There are no changes to commit".to_string());
    }

    let remote_paths: HashSet<String> = remote::list_files(repo, &head)?.into_iter().map(|entry| entry.path).collect();
    for (path, kind) in &changes {
        if *kind == ChangeKind::Added && remote_paths.contains(path) {
            return Err(format!("{path} exists in the repository but is not synced here. Rename it or exclude it."));
        }
        if local.get(path).is_some_and(|file| file.content.len() as u64 > MAX_FILE_BYTES) {
            return Err(format!("{path} is larger than 50 MB and cannot be pushed"));
        }
    }

    let hashes: HashMap<String, String> = changes
        .iter()
        .filter_map(|(path, _)| local.get(path).map(|file| (path.clone(), file.hash.clone())))
        .collect();
    let file_changes: Vec<FileChange> = changes
        .iter()
        .map(|(path, kind)| FileChange {
            path: path.clone(),
            kind: *kind,
            content: local.remove(path).map(|file| file.content).unwrap_or_default(),
        })
        .collect();

    let commit = remote::commit(repo, &base_commit, &file_changes, &message)?;

    let pushed_shas: HashMap<String, String> = remote::list_files(repo, &commit)?
        .into_iter()
        .filter(|entry| hashes.contains_key(&entry.path))
        .map(|entry| (entry.path, entry.sha))
        .collect();
    let mut update = SnapshotUpdate::default();
    for (path, kind) in &changes {
        match (kind, pushed_shas.get(path), hashes.get(path)) {
            (ChangeKind::Deleted, _, _) => update.removed.push(path.clone()),
            (_, Some(blob_sha), Some(content_hash)) => update.saved.push((
                path.clone(),
                SnapshotEntry { blob_sha: blob_sha.clone(), content_hash: content_hash.clone() },
            )),
            _ => {}
        }
    }
    store.apply_git_snapshot(&link.project_path, &update, &commit, &now())?;

    Ok(PushOutcome {
        commit,
        pushed: changes.into_iter().map(|(path, _)| path).collect(),
    })
}
