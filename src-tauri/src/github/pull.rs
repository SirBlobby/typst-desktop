use std::collections::{HashMap, HashSet};
use std::path::Path;

use serde::{Deserialize, Serialize};

use super::file_rules::{check_file_count, MAX_FILE_BYTES};
use super::local_files::{self, normalize_content, LocalFile};
use super::remote::{self, ChangeKind, RemoteEntry, RemoteRepo};
use super::storage::{GitLink, Snapshot, SnapshotEntry, SnapshotUpdate};
use super::sync_plan::remote_changes;
use crate::db::Store;
use crate::workspace::{self, content_hash};

const DOWNLOAD_THREADS: usize = 6;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Resolution {
    #[default]
    Ask,
    KeepRepository,
    KeepMine,
}

#[derive(Serialize, Default)]
pub struct PullOutcome {
    pub updated: Vec<String>,
    pub removed: Vec<String>,
    pub conflicts: Vec<String>,
    pub too_large: Vec<String>,
    pub applied: bool,
}

pub struct PullInput<'a> {
    pub store: &'a Store,
    pub project_dir: &'a Path,
    pub link: &'a GitLink,
    pub repo: &'a RemoteRepo,
    pub head: &'a str,
    pub entries: Vec<RemoteEntry>,
    pub local: HashMap<String, LocalFile>,
    pub snapshot: Snapshot,
    pub resolution: Resolution,
    pub remove_local_only: bool,
}

struct Download {
    path: String,
    sha: String,
    content: Vec<u8>,
    hash: String,
}

enum Action {
    Write(String, Vec<u8>),
    Remove(String),
}

fn is_locally_changed(path: &str, local: &HashMap<String, LocalFile>, snapshot: &Snapshot) -> bool {
    match (local.get(path), snapshot.get(path)) {
        (Some(file), Some(saved)) => file.hash != saved.content_hash,
        (None, None) => false,
        _ => true,
    }
}

fn fetch_all(repo: &RemoteRepo, entries: Vec<RemoteEntry>) -> Result<Vec<(RemoteEntry, Option<Vec<u8>>)>, String> {
    if entries.is_empty() {
        return Ok(Vec::new());
    }
    let chunk_size = entries.len().div_ceil(DOWNLOAD_THREADS);
    let chunks: Vec<Vec<RemoteEntry>> = entries.chunks(chunk_size).map(<[RemoteEntry]>::to_vec).collect();

    std::thread::scope(|scope| {
        let workers: Vec<_> = chunks
            .into_iter()
            .map(|chunk| {
                scope.spawn(move || {
                    chunk
                        .into_iter()
                        .map(|entry| {
                            let content = remote::read_blob(repo, &entry.sha)?;
                            Ok((entry, content))
                        })
                        .collect::<Result<Vec<_>, String>>()
                })
            })
            .collect();

        let mut fetched = Vec::new();
        for worker in workers {
            fetched.extend(worker.join().map_err(|_| "A download stopped unexpectedly".to_string())??);
        }
        Ok(fetched)
    })
}

fn download(repo: &RemoteRepo, entries: Vec<RemoteEntry>) -> Result<(Vec<Download>, Vec<String>), String> {
    let (oversized, wanted): (Vec<RemoteEntry>, Vec<RemoteEntry>) =
        entries.into_iter().partition(|entry| entry.size.is_some_and(|size| size > MAX_FILE_BYTES));
    let mut too_large: Vec<String> = oversized.into_iter().map(|entry| entry.path).collect();

    let mut downloads = Vec::new();
    for (entry, content) in fetch_all(repo, wanted)? {
        match content {
            Some(content) => {
                let content = normalize_content(&entry.path, content);
                let hash = content_hash(&content);
                downloads.push(Download { path: entry.path, sha: entry.sha, content, hash });
            }
            None => too_large.push(entry.path),
        }
    }
    too_large.sort();
    Ok((downloads, too_large))
}

fn apply_actions(project_dir: &Path, actions: &[Action]) -> Result<(), String> {
    for action in actions {
        match action {
            Action::Write(path, content) => local_files::write(project_dir, path, content)?,
            Action::Remove(path) => local_files::remove(project_dir, path)?,
        }
    }
    Ok(())
}

fn repair_entrypoint(store: &Store, project_path: &str, project_dir: &Path) -> Result<(), String> {
    let mut meta = store.meta(project_path)?;
    if project_dir.join(&meta.entrypoint).is_file() {
        return Ok(());
    }
    let paths = workspace::collect_files(project_dir)?;
    meta.entrypoint = crate::target_entries::entrypoint_after_removal(&meta.entrypoint, &paths);
    store.save_meta(project_path, &meta)
}

pub fn now() -> String {
    chrono::Utc::now().to_rfc3339()
}

pub fn pull_into_project(input: PullInput<'_>) -> Result<PullOutcome, String> {
    let PullInput { store, project_dir, link, repo, head, entries, local, snapshot, resolution, remove_local_only } = input;
    let rules = link.rules();
    check_file_count(entries.iter().filter(|entry| rules.is_managed(&entry.path)).count())?;
    let changes = remote_changes(&entries, &snapshot, &rules);
    let remote_paths: HashSet<String> = entries.iter().map(|entry| entry.path.clone()).collect();
    let wanted: Vec<RemoteEntry> = entries
        .into_iter()
        .filter(|entry| matches!(changes.get(&entry.path), Some(ChangeKind::Added | ChangeKind::Modified)))
        .collect();
    let (downloads, too_large) = download(repo, wanted)?;

    let mut outcome = PullOutcome { too_large, ..PullOutcome::default() };
    let mut actions = Vec::new();
    let mut update = SnapshotUpdate::default();

    for download in downloads {
        let unchanged = local.get(&download.path).is_some_and(|file| file.hash == download.hash);
        let conflict = !unchanged && is_locally_changed(&download.path, &local, &snapshot);
        if conflict && resolution == Resolution::Ask {
            outcome.conflicts.push(download.path);
            continue;
        }
        if !unchanged && (!conflict || resolution == Resolution::KeepRepository) {
            outcome.updated.push(download.path.clone());
            actions.push(Action::Write(download.path.clone(), download.content));
        }
        update.saved.push((download.path, SnapshotEntry { blob_sha: download.sha, content_hash: download.hash }));
    }

    for (path, _) in changes.iter().filter(|(_, kind)| **kind == ChangeKind::Deleted) {
        let exists_locally = local.contains_key(path);
        let conflict = exists_locally && is_locally_changed(path, &local, &snapshot);
        if conflict && resolution == Resolution::Ask {
            outcome.conflicts.push(path.clone());
            continue;
        }
        if exists_locally && (!conflict || resolution == Resolution::KeepRepository) {
            outcome.removed.push(path.clone());
            actions.push(Action::Remove(path.clone()));
        }
        update.removed.push(path.clone());
    }

    if remove_local_only {
        let mut local_only: Vec<&String> = local.keys().filter(|path| !remote_paths.contains(*path)).collect();
        local_only.sort();
        for path in local_only {
            outcome.removed.push(path.clone());
            actions.push(Action::Remove(path.clone()));
        }
    }

    if !outcome.conflicts.is_empty() {
        outcome.conflicts.sort();
        return Ok(outcome);
    }

    apply_actions(project_dir, &actions)?;
    store.apply_git_snapshot(&link.project_path, &update, head, &now())?;
    if !actions.is_empty() {
        repair_entrypoint(store, &link.project_path, project_dir)?;
    }
    outcome.applied = true;
    Ok(outcome)
}
