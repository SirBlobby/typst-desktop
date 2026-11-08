use std::collections::{BTreeMap, HashMap, HashSet};

use serde::Serialize;

use super::file_rules::FileRules;
use super::local_files::LocalFile;
use super::remote::{ChangeKind, RemoteEntry};
use super::storage::Snapshot;

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct Change {
    pub path: String,
    pub change: &'static str,
}

pub fn local_changes(local: &HashMap<String, LocalFile>, snapshot: &Snapshot, rules: &FileRules) -> BTreeMap<String, ChangeKind> {
    let mut changes = BTreeMap::new();
    for (path, file) in local {
        match snapshot.get(path) {
            None => {
                changes.insert(path.clone(), ChangeKind::Added);
            }
            Some(entry) if entry.content_hash != file.hash => {
                changes.insert(path.clone(), ChangeKind::Modified);
            }
            Some(_) => {}
        }
    }
    for path in snapshot.keys().filter(|path| rules.is_managed(path) && !local.contains_key(*path)) {
        changes.insert(path.clone(), ChangeKind::Deleted);
    }
    changes
}

pub fn remote_changes(entries: &[RemoteEntry], snapshot: &Snapshot, rules: &FileRules) -> BTreeMap<String, ChangeKind> {
    let mut changes = BTreeMap::new();
    for entry in entries.iter().filter(|entry| rules.is_managed(&entry.path)) {
        match snapshot.get(&entry.path) {
            None => {
                changes.insert(entry.path.clone(), ChangeKind::Added);
            }
            Some(saved) if saved.blob_sha != entry.sha => {
                changes.insert(entry.path.clone(), ChangeKind::Modified);
            }
            Some(_) => {}
        }
    }
    let remote_paths: HashSet<&str> = entries.iter().map(|entry| entry.path.as_str()).collect();
    for path in snapshot.keys().filter(|path| rules.is_managed(path) && !remote_paths.contains(path.as_str())) {
        changes.insert(path.clone(), ChangeKind::Deleted);
    }
    changes
}

pub fn as_list(changes: &BTreeMap<String, ChangeKind>) -> Vec<Change> {
    changes
        .iter()
        .map(|(path, kind)| Change { path: path.clone(), change: kind.label() })
        .collect()
}
