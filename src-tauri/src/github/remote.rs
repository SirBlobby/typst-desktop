use base64::{engine::general_purpose::STANDARD as BASE64, Engine as _};
use serde::Serialize;
use serde_json::{json, Value};

use super::http::{encode_segment, text_field, unreadable, Client};
use super::repo_url::{is_clean_path, RepoUrl};

const FILE_MODE: &str = "100644";
const SYMLINK_MODE: &str = "120000";

#[derive(Clone)]
pub struct RemoteRepo {
    pub url: RepoUrl,
    pub branch: String,
    pub folder: String,
    pub token: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct RemoteEntry {
    pub path: String,
    pub sha: String,
    pub size: Option<u64>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ChangeKind {
    Added,
    Modified,
    Deleted,
}

impl ChangeKind {
    pub fn label(self) -> &'static str {
        match self {
            Self::Added => "added",
            Self::Modified => "modified",
            Self::Deleted => "deleted",
        }
    }
}

pub struct FileChange {
    pub path: String,
    pub kind: ChangeKind,
    pub content: Vec<u8>,
}

impl RemoteRepo {
    fn full_path(&self, path: &str) -> String {
        if self.folder.is_empty() {
            path.to_string()
        } else {
            format!("{}/{path}", self.folder)
        }
    }

    fn client(&self) -> Client {
        Client::new(format!("{}/repos/{}", self.url.api_base(), self.url.path), self.token.as_deref())
    }

    fn branch_ref(&self) -> String {
        let segments: Vec<String> = self.branch.split('/').map(encode_segment).collect();
        format!("heads/{}", segments.join("/"))
    }
}

fn is_object_id(value: &str) -> bool {
    matches!(value.len(), 40 | 64) && value.chars().all(|character| character.is_ascii_hexdigit())
}

fn checked_object_id(value: String) -> Result<String, String> {
    if is_object_id(&value) {
        Ok(value.to_ascii_lowercase())
    } else {
        Err("GitHub sent an unexpected commit id".to_string())
    }
}

fn is_safe_entry(entry: &RemoteEntry) -> bool {
    is_object_id(&entry.sha) && is_clean_path(&entry.path)
}

fn commit_tree(client: &Client, commit: &str) -> Result<String, String> {
    let body = client.get_json(&format!("/git/commits/{commit}"))?;
    text_field(&body, "/tree/sha").ok_or_else(unreadable)
}

fn folder_tree(client: &Client, repo: &RemoteRepo, commit: &str) -> Result<String, String> {
    let mut tree_sha = commit_tree(client, commit)?;
    for segment in repo.folder.split('/').filter(|segment| !segment.is_empty()) {
        let listing = client.get_json(&format!("/git/trees/{tree_sha}"))?;
        let entries = listing.get("tree").and_then(Value::as_array).ok_or_else(unreadable)?;
        tree_sha = entries
            .iter()
            .find(|entry| {
                entry.get("path").and_then(Value::as_str) == Some(segment)
                    && entry.get("type").and_then(Value::as_str) == Some("tree")
            })
            .and_then(|entry| text_field(entry, "/sha"))
            .ok_or_else(|| format!("The folder {} does not exist on this branch", repo.folder))?;
    }
    Ok(tree_sha)
}

pub fn token_owner(url: &RepoUrl, token: &str) -> Result<String, String> {
    let body = Client::new(url.api_base(), Some(token)).get_json("/user")?;
    text_field(&body, "/login").ok_or_else(unreadable)
}

pub fn head(repo: &RemoteRepo) -> Result<String, String> {
    let body = repo.client().get_json(&format!("/git/ref/{}", repo.branch_ref()))?;
    checked_object_id(text_field(&body, "/object/sha").ok_or_else(unreadable)?)
}

pub fn list_files(repo: &RemoteRepo, commit: &str) -> Result<Vec<RemoteEntry>, String> {
    let client = repo.client();
    let tree_sha = folder_tree(&client, repo, commit)?;
    let listing = client.get_json(&format!("/git/trees/{tree_sha}?recursive=1"))?;
    if listing.get("truncated").and_then(Value::as_bool).unwrap_or(false) {
        return Err("This folder has too many files to sync. Link a smaller folder.".to_string());
    }
    let entries = listing.get("tree").and_then(Value::as_array).ok_or_else(unreadable)?;
    Ok(entries
        .iter()
        .filter(|entry| entry.get("type").and_then(Value::as_str) == Some("blob"))
        .filter(|entry| entry.get("mode").and_then(Value::as_str) != Some(SYMLINK_MODE))
        .filter_map(|entry| {
            Some(RemoteEntry {
                path: text_field(entry, "/path")?,
                sha: text_field(entry, "/sha")?,
                size: entry.get("size").and_then(Value::as_u64),
            })
        })
        .filter(is_safe_entry)
        .collect())
}

pub fn read_blob(repo: &RemoteRepo, sha: &str) -> Result<Option<Vec<u8>>, String> {
    let client = repo.client();
    let response = client.get(&format!("/git/blobs/{sha}"), &[("Accept", "application/vnd.github.raw+json")])?;
    client.read_limited(response)
}

fn tree_item(client: &Client, repo: &RemoteRepo, change: &FileChange) -> Result<Value, String> {
    let path = repo.full_path(&change.path);
    if change.kind == ChangeKind::Deleted {
        return Ok(json!({ "path": path, "mode": FILE_MODE, "type": "blob", "sha": Value::Null }));
    }
    let blob = client.send_json(
        "POST",
        "/git/blobs",
        &json!({ "content": BASE64.encode(&change.content), "encoding": "base64" }),
    )?;
    let sha = text_field(&blob, "/sha").ok_or_else(unreadable)?;
    Ok(json!({ "path": path, "mode": FILE_MODE, "type": "blob", "sha": sha }))
}

pub fn commit(repo: &RemoteRepo, base_commit: &str, changes: &[FileChange], message: &str) -> Result<String, String> {
    let client = repo.client();
    let base_tree = commit_tree(&client, base_commit)?;
    let items = changes
        .iter()
        .map(|change| tree_item(&client, repo, change))
        .collect::<Result<Vec<Value>, String>>()?;

    let tree = client.send_json("POST", "/git/trees", &json!({ "base_tree": base_tree, "tree": items }))?;
    let tree_sha = text_field(&tree, "/sha").ok_or_else(unreadable)?;

    let commit = client.send_json(
        "POST",
        "/git/commits",
        &json!({ "message": message, "tree": tree_sha, "parents": [base_commit] }),
    )?;
    let commit_sha = checked_object_id(text_field(&commit, "/sha").ok_or_else(unreadable)?)?;

    client.send_json(
        "PATCH",
        &format!("/git/refs/{}", repo.branch_ref()),
        &json!({ "sha": commit_sha, "force": false }),
    )?;
    Ok(commit_sha)
}
