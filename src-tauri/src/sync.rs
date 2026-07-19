use base64::{engine::general_purpose::STANDARD as BASE64, Engine as _};
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::path::Path;

use crate::workspace::{
    collect_files, content_hash, is_text_file, project_file_path, ProjectMeta,
};
use crate::db::Store;

const REQUEST_TIMEOUT_SECS: u64 = 30;

fn agent() -> ureq::Agent {
    ureq::AgentBuilder::new()
        .timeout(std::time::Duration::from_secs(REQUEST_TIMEOUT_SECS))
        .build()
}

fn endpoint(server_url: &str, path: &str) -> String {
    format!("{}/api/desktop{}", server_url.trim_end_matches('/'), path)
}

fn describe(error: ureq::Error) -> String {
    match error {
        ureq::Error::Status(code, response) => {
            let body = response.into_string().unwrap_or_default();
            if body.is_empty() {
                format!("Server returned {}", code)
            } else {
                body
            }
        }
        other => other.to_string(),
    }
}

#[derive(Deserialize, Serialize, Clone)]
pub struct Account {
    pub user_id: String,
    pub username: String,
    pub email: String,
}

#[derive(Deserialize)]
pub struct LoginResponse {
    pub token: String,
    pub user_id: String,
    pub username: String,
    pub email: String,
}

pub fn login(
    server_url: &str,
    email: &str,
    password: &str,
    device_name: &str,
) -> Result<LoginResponse, String> {
    agent()
        .post(&endpoint(server_url, "/auth/login"))
        .send_json(ureq::json!({
            "email": email,
            "password": password,
            "device_name": device_name,
        }))
        .map_err(describe)?
        .into_json::<LoginResponse>()
        .map_err(|e| e.to_string())
}

pub fn logout(server_url: &str, token: &str) -> Result<(), String> {
    agent()
        .post(&endpoint(server_url, "/auth/logout"))
        .set("Authorization", &format!("Bearer {}", token))
        .call()
        .map_err(describe)?;
    Ok(())
}

pub fn me(server_url: &str, token: &str) -> Result<Account, String> {
    agent()
        .get(&endpoint(server_url, "/auth/me"))
        .set("Authorization", &format!("Bearer {}", token))
        .call()
        .map_err(describe)?
        .into_json::<Account>()
        .map_err(|e| e.to_string())
}

#[derive(Deserialize, Serialize, Clone)]
pub struct SpaceSummary {
    pub id: String,
    pub name: String,
    pub entrypoint: String,
    pub role: String,
    pub updated_at: String,
}

pub fn list_spaces(server_url: &str, token: &str) -> Result<Vec<SpaceSummary>, String> {
    agent()
        .get(&endpoint(server_url, "/spaces"))
        .set("Authorization", &format!("Bearer {}", token))
        .call()
        .map_err(describe)?
        .into_json::<Vec<SpaceSummary>>()
        .map_err(|e| e.to_string())
}

pub fn create_space(server_url: &str, token: &str, name: &str) -> Result<SpaceSummary, String> {
    agent()
        .post(&endpoint(server_url, "/spaces"))
        .set("Authorization", &format!("Bearer {}", token))
        .send_json(ureq::json!({ "name": name }))
        .map_err(describe)?
        .into_json::<SpaceSummary>()
        .map_err(|e| e.to_string())
}

pub fn delete_space(server_url: &str, token: &str, space_id: &str) -> Result<(), String> {
    agent()
        .delete(&endpoint(server_url, &format!("/spaces/{}", space_id)))
        .set("Authorization", &format!("Bearer {}", token))
        .call()
        .map_err(describe)?;
    Ok(())
}

#[derive(Deserialize)]
pub struct ManifestEntry {
    pub path: String,
    pub hash: String,
}

#[derive(Deserialize)]
pub struct SpaceManifest {
    pub entrypoint: String,
    pub files: Vec<ManifestEntry>,
}

pub fn get_manifest(
    server_url: &str,
    token: &str,
    space_id: &str,
) -> Result<SpaceManifest, String> {
    agent()
        .get(&endpoint(
            server_url,
            &format!("/spaces/{}/manifest", space_id),
        ))
        .set("Authorization", &format!("Bearer {}", token))
        .call()
        .map_err(describe)?
        .into_json::<SpaceManifest>()
        .map_err(|e| e.to_string())
}

#[derive(Deserialize)]
pub struct FileContent {
    pub kind: String,
    pub hash: String,
    pub encoding: String,
    pub content: String,
}

impl FileContent {
    pub fn bytes(&self) -> Result<Vec<u8>, String> {
        if self.encoding == "base64" {
            BASE64
                .decode(self.content.as_bytes())
                .map_err(|e| format!("Invalid base64 from server: {}", e))
        } else {
            Ok(self.content.clone().into_bytes())
        }
    }
}

pub fn pull_file(
    server_url: &str,
    token: &str,
    space_id: &str,
    path: &str,
) -> Result<FileContent, String> {
    agent()
        .get(&endpoint(server_url, &format!("/spaces/{}/file", space_id)))
        .query("path", path)
        .set("Authorization", &format!("Bearer {}", token))
        .call()
        .map_err(describe)?
        .into_json::<FileContent>()
        .map_err(|e| e.to_string())
}

#[derive(Deserialize)]
struct ConflictBody {
    server_hash: String,
    encoding: String,
    server_content: String,
}

pub enum PushResult {
    Applied,
    Conflict { server_hash: String, server_text: String },
}

pub fn push_file(
    server_url: &str,
    token: &str,
    space_id: &str,
    path: &str,
    bytes: &[u8],
    base_hash: Option<&str>,
) -> Result<PushResult, String> {
    let (encoding, content) = if is_text_file(path) {
        ("utf8", String::from_utf8_lossy(bytes).to_string())
    } else {
        ("base64", BASE64.encode(bytes))
    };

    let response = agent()
        .put(&endpoint(server_url, &format!("/spaces/{}/file", space_id)))
        .set("Authorization", &format!("Bearer {}", token))
        .send_json(ureq::json!({
            "path": path,
            "content": content,
            "encoding": encoding,
            "base_hash": base_hash,
        }));

    match response {
        Ok(_) => Ok(PushResult::Applied),
        Err(ureq::Error::Status(409, body)) => {
            let conflict = body
                .into_json::<ConflictBody>()
                .map_err(|e| format!("Malformed conflict response: {}", e))?;
            let server_text = if conflict.encoding == "base64" {
                String::new()
            } else {
                conflict.server_content
            };
            Ok(PushResult::Conflict {
                server_hash: conflict.server_hash,
                server_text,
            })
        }
        Err(other) => Err(describe(other)),
    }
}

pub fn delete_remote_file(
    server_url: &str,
    token: &str,
    space_id: &str,
    path: &str,
) -> Result<(), String> {
    let response = agent()
        .delete(&endpoint(server_url, &format!("/spaces/{}/file", space_id)))
        .query("path", path)
        .set("Authorization", &format!("Bearer {}", token))
        .call();

    match response {
        Ok(_) => Ok(()),
        Err(ureq::Error::Status(404, _)) => Ok(()),
        Err(other) => Err(describe(other)),
    }
}

#[derive(Serialize, Clone)]
pub struct Conflict {
    pub path: String,
    pub local_text: String,
    pub remote_text: String,
    pub merged_text: String,
    pub server_hash: String,
    pub auto_merged: bool,
    pub binary: bool,
}

#[derive(Serialize, Default)]
pub struct SyncReport {
    pub pushed: Vec<String>,
    pub pulled: Vec<String>,
    pub deleted_local: Vec<String>,
    pub deleted_remote: Vec<String>,
    pub merged: Vec<String>,
    pub conflicts: Vec<Conflict>,
}

fn read_local(project_dir: &Path, relative: &str) -> Result<Vec<u8>, String> {
    let full = project_file_path(project_dir, relative)?;
    std::fs::read(&full).map_err(|e| e.to_string())
}

fn write_local(project_dir: &Path, relative: &str, bytes: &[u8]) -> Result<(), String> {
    let full = project_file_path(project_dir, relative)?;
    if let Some(parent) = full.parent() {
        std::fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    }
    std::fs::write(&full, bytes).map_err(|e| e.to_string())
}

pub fn pull_project(
    server_url: &str,
    token: &str,
    store: &Store,
    project: &str,
    project_dir: &Path,
    meta: &mut ProjectMeta,
) -> Result<SyncReport, String> {
    let space_id = meta
        .space_id
        .clone()
        .ok_or("Project is not linked to a cloud space")?;

    let manifest = get_manifest(server_url, token, &space_id)?;
    let mut report = SyncReport::default();

    let local_files: HashSet<String> = collect_files(project_dir)?.into_iter().collect();
    let mut remote_paths = HashSet::new();

    for entry in &manifest.files {
        remote_paths.insert(entry.path.clone());

        let base = meta.base_hashes.get(&entry.path).cloned();
        let local_exists = local_files.contains(&entry.path);

        if !local_exists {
            if base.is_some() {
                continue;
            }
            let remote = pull_file(server_url, token, &space_id, &entry.path)?;
            write_local(project_dir, &entry.path, &remote.bytes()?)?;
            meta.base_hashes.insert(entry.path.clone(), remote.hash);
            report.pulled.push(entry.path.clone());
            continue;
        }

        let local_bytes = read_local(project_dir, &entry.path)?;
        let local_hash = content_hash(&local_bytes);

        if local_hash == entry.hash {
            meta.base_hashes.insert(entry.path.clone(), entry.hash.clone());
            continue;
        }

        if base.as_deref() == Some(entry.hash.as_str()) {
            continue;
        }

        let remote = pull_file(server_url, token, &space_id, &entry.path)?;
        let remote_bytes = remote.bytes()?;

        if base.as_deref() == Some(local_hash.as_str()) {
            write_local(project_dir, &entry.path, &remote_bytes)?;
            meta.base_hashes.insert(entry.path.clone(), remote.hash);
            report.pulled.push(entry.path.clone());
            continue;
        }

        if !is_text_file(&entry.path) || remote.kind == "binary" {
            report.conflicts.push(Conflict {
                path: entry.path.clone(),
                local_text: String::new(),
                remote_text: String::new(),
                merged_text: String::new(),
                server_hash: remote.hash,
                auto_merged: false,
                binary: true,
            });
            continue;
        }

        let local_text = String::from_utf8_lossy(&local_bytes).to_string();
        let remote_text = String::from_utf8_lossy(&remote_bytes).to_string();
        let base_text = read_base_snapshot(store, project, &entry.path);

        match diffy::merge(&base_text, &local_text, &remote_text) {
            Ok(merged) => {
                write_local(project_dir, &entry.path, merged.as_bytes())?;
                meta.base_hashes
                    .insert(entry.path.clone(), content_hash(merged.as_bytes()));
                report.merged.push(entry.path.clone());
            }
            Err(conflicted) => {
                report.conflicts.push(Conflict {
                    path: entry.path.clone(),
                    local_text,
                    remote_text,
                    merged_text: conflicted,
                    server_hash: remote.hash,
                    auto_merged: false,
                    binary: false,
                });
            }
        }
    }

    let vanished: Vec<String> = meta
        .base_hashes
        .keys()
        .filter(|path| !remote_paths.contains(*path) && local_files.contains(*path))
        .cloned()
        .collect();

    for path in vanished {
        let local_bytes = read_local(project_dir, &path)?;
        let base = meta.base_hashes.get(&path).cloned().unwrap_or_default();
        if content_hash(&local_bytes) == base {
            let full = project_file_path(project_dir, &path)?;
            let _ = std::fs::remove_file(full);
            meta.base_hashes.remove(&path);
            report.deleted_local.push(path);
        }
    }

    meta.entrypoint = manifest.entrypoint;
    meta.last_synced_at = Some(chrono::Utc::now().to_rfc3339());
    store.save_meta(project, meta)?;
    save_base_snapshots(store, project, project_dir, meta)?;

    Ok(report)
}

pub fn push_project(
    server_url: &str,
    token: &str,
    store: &Store,
    project: &str,
    project_dir: &Path,
    meta: &mut ProjectMeta,
) -> Result<SyncReport, String> {
    let space_id = meta
        .space_id
        .clone()
        .ok_or("Project is not linked to a cloud space")?;

    let mut report = SyncReport::default();
    let local_files = collect_files(project_dir)?;
    let local_set: HashSet<String> = local_files.iter().cloned().collect();

    for path in &local_files {
        let bytes = read_local(project_dir, path)?;
        let hash = content_hash(&bytes);
        let base = meta.base_hashes.get(path).cloned();

        if base.as_deref() == Some(hash.as_str()) {
            continue;
        }

        match push_file(server_url, token, &space_id, path, &bytes, base.as_deref())? {
            PushResult::Applied => {
                meta.base_hashes.insert(path.clone(), hash);
                report.pushed.push(path.clone());
            }
            PushResult::Conflict {
                server_hash,
                server_text,
            } => {
                let binary = !is_text_file(path);
                report.conflicts.push(Conflict {
                    path: path.clone(),
                    local_text: if binary {
                        String::new()
                    } else {
                        String::from_utf8_lossy(&bytes).to_string()
                    },
                    remote_text: server_text.clone(),
                    merged_text: server_text,
                    server_hash,
                    auto_merged: false,
                    binary,
                });
            }
        }
    }

    let removed: Vec<String> = meta
        .base_hashes
        .keys()
        .filter(|path| !local_set.contains(*path))
        .cloned()
        .collect();

    for path in removed {
        delete_remote_file(server_url, token, &space_id, &path)?;
        meta.base_hashes.remove(&path);
        report.deleted_remote.push(path);
    }

    meta.last_synced_at = Some(chrono::Utc::now().to_rfc3339());
    store.save_meta(project, meta)?;
    save_base_snapshots(store, project, project_dir, meta)?;

    Ok(report)
}

#[derive(Serialize, Clone)]
pub struct DownloadProgress {
    pub label: String,
    pub current: usize,
    pub total: usize,
    pub done: bool,
}

pub const PROGRESS_EVENT: &str = "download://progress";

pub fn report_progress(
    app: &tauri::AppHandle,
    label: &str,
    current: usize,
    total: usize,
    done: bool,
) {
    use tauri::Emitter;
    let _ = app.emit(
        PROGRESS_EVENT,
        DownloadProgress {
            label: label.to_string(),
            current,
            total,
            done,
        },
    );
}

pub fn clone_space(
    server_url: &str,
    token: &str,
    app: &tauri::AppHandle,
    store: &Store,
    project: &str,
    project_dir: &Path,
    space_id: &str,
) -> Result<SyncReport, String> {
    std::fs::create_dir_all(project_dir).map_err(|e| e.to_string())?;

    let manifest = get_manifest(server_url, token, space_id)?;
    let mut meta = store.meta(project)?;
    meta.space_id = Some(space_id.to_string());
    meta.entrypoint = manifest.entrypoint.clone();

    let mut report = SyncReport::default();
    let total = manifest.files.len();

    for (index, entry) in manifest.files.iter().enumerate() {
        report_progress(app, project, index, total, false);

        let remote = pull_file(server_url, token, space_id, &entry.path)?;
        write_local(project_dir, &entry.path, &remote.bytes()?)?;
        meta.base_hashes.insert(entry.path.clone(), remote.hash);
        report.pulled.push(entry.path.clone());
    }

    report_progress(app, project, total, total, true);

    meta.last_synced_at = Some(chrono::Utc::now().to_rfc3339());
    store.save_meta(project, &meta)?;
    save_base_snapshots(store, project, project_dir, &meta)?;

    Ok(report)
}

pub fn save_base_snapshots(
    store: &Store,
    project: &str,
    project_dir: &Path,
    meta: &ProjectMeta,
) -> Result<(), String> {
    for (path, base) in &meta.base_hashes {
        let full = project_file_path(project_dir, path)?;
        let Ok(bytes) = std::fs::read(&full) else {
            continue;
        };
        if content_hash(&bytes) == *base {
            store.save_base_snapshot(project, path, base, &bytes)?;
        }
    }

    Ok(())
}

fn read_base_snapshot(store: &Store, project: &str, relative: &str) -> String {
    store
        .base_snapshot(project, relative)
        .ok()
        .flatten()
        .map(|bytes| String::from_utf8_lossy(&bytes).to_string())
        .unwrap_or_default()
}

pub fn resolve_conflict(
    store: &Store,
    project: &str,
    project_dir: &Path,
    meta: &mut ProjectMeta,
    path: &str,
    content: &str,
    server_hash: &str,
) -> Result<(), String> {
    write_local(project_dir, path, content.as_bytes())?;
    meta.base_hashes
        .insert(path.to_string(), server_hash.to_string());
    store.save_meta(project, meta)
}

#[derive(Deserialize, Serialize, Clone)]
pub struct CloudFolder {
    pub id: String,
    pub name: String,
    pub parent_id: Option<String>,
}

#[derive(Deserialize, Serialize, Clone)]
pub struct CloudDocument {
    pub id: String,
    pub title: String,
    pub folder_id: Option<String>,
    pub role: String,
    pub updated_at: String,
}

#[derive(Deserialize, Serialize)]
pub struct SharedItems {
    pub documents: Vec<CloudDocument>,
    pub spaces: Vec<SpaceSummary>,
}

pub fn list_folders(server_url: &str, token: &str) -> Result<Vec<CloudFolder>, String> {
    agent()
        .get(&endpoint(server_url, "/folders"))
        .set("Authorization", &format!("Bearer {}", token))
        .call()
        .map_err(describe)?
        .into_json::<Vec<CloudFolder>>()
        .map_err(|e| e.to_string())
}

pub fn list_documents(
    server_url: &str,
    token: &str,
    folder_id: Option<&str>,
) -> Result<Vec<CloudDocument>, String> {
    let mut request = agent()
        .get(&endpoint(server_url, "/documents"))
        .set("Authorization", &format!("Bearer {}", token));

    if let Some(folder) = folder_id {
        request = request.query("folder_id", folder);
    }

    request
        .call()
        .map_err(describe)?
        .into_json::<Vec<CloudDocument>>()
        .map_err(|e| e.to_string())
}

pub fn list_shared(server_url: &str, token: &str) -> Result<SharedItems, String> {
    agent()
        .get(&endpoint(server_url, "/shared"))
        .set("Authorization", &format!("Bearer {}", token))
        .call()
        .map_err(describe)?
        .into_json::<SharedItems>()
        .map_err(|e| e.to_string())
}

#[derive(Deserialize, Serialize)]
pub struct DocumentContent {
    pub id: String,
    pub title: String,
    pub role: String,
    pub hash: String,
    pub content: String,
}

pub fn pull_document(
    server_url: &str,
    token: &str,
    document_id: &str,
) -> Result<DocumentContent, String> {
    agent()
        .get(&endpoint(server_url, &format!("/documents/{}", document_id)))
        .set("Authorization", &format!("Bearer {}", token))
        .call()
        .map_err(describe)?
        .into_json::<DocumentContent>()
        .map_err(|e| e.to_string())
}

pub fn sync_document(
    server_url: &str,
    token: &str,
    app: &tauri::AppHandle,
    store: &Store,
    path: &str,
) -> Result<SyncReport, String> {
    let link = store
        .document_link(path)?
        .ok_or("This document is not linked to the cloud")?;

    let full = crate::workspace::workspace_path(app, store, path)?;
    let local = std::fs::read_to_string(&full).map_err(|e| e.to_string())?;
    let remote = pull_document(server_url, token, &link.document_id)?;

    let mut report = SyncReport::default();

    let local_hash = content_hash(local.as_bytes());
    let editable = remote.role == "owner" || remote.role == "editor";

    if local_hash == remote.hash {
        store.save_document_link(path, &link.document_id, &remote.hash, &remote.role, &local)?;
        return Ok(report);
    }

    if local_hash == link.base_hash {
        std::fs::write(&full, &remote.content).map_err(|e| e.to_string())?;
        store.save_document_link(
            path,
            &link.document_id,
            &remote.hash,
            &remote.role,
            &remote.content,
        )?;
        report.pulled.push(path.to_string());
        return Ok(report);
    }

    if remote.hash == link.base_hash {
        if !editable {
            return Err("You only have view access to this document".to_string());
        }
        match push_document(
            server_url,
            token,
            &link.document_id,
            &local,
            Some(&link.base_hash),
        )? {
            PushResult::Applied => {
                store.save_document_link(
                    path,
                    &link.document_id,
                    &local_hash,
                    &remote.role,
                    &local,
                )?;
                report.pushed.push(path.to_string());
            }
            PushResult::Conflict {
                server_hash,
                server_text,
            } => {
                report.conflicts.push(Conflict {
                    path: path.to_string(),
                    local_text: local,
                    remote_text: server_text.clone(),
                    merged_text: server_text,
                    server_hash,
                    auto_merged: false,
                    binary: false,
                });
            }
        }
        return Ok(report);
    }

    match diffy::merge(&link.base_content, &local, &remote.content) {
        Ok(merged) => {
            std::fs::write(&full, &merged).map_err(|e| e.to_string())?;

            if editable {
                match push_document(
                    server_url,
                    token,
                    &link.document_id,
                    &merged,
                    Some(&remote.hash),
                )? {
                    PushResult::Applied => {
                        store.save_document_link(
                            path,
                            &link.document_id,
                            &content_hash(merged.as_bytes()),
                            &remote.role,
                            &merged,
                        )?;
                    }
                    PushResult::Conflict {
                        server_hash,
                        server_text,
                    } => {
                        report.conflicts.push(Conflict {
                            path: path.to_string(),
                            local_text: merged.clone(),
                            remote_text: server_text.clone(),
                            merged_text: server_text,
                            server_hash,
                            auto_merged: false,
                            binary: false,
                        });
                        return Ok(report);
                    }
                }
            }

            report.merged.push(path.to_string());
        }
        Err(conflicted) => {
            report.conflicts.push(Conflict {
                path: path.to_string(),
                local_text: local,
                remote_text: remote.content,
                merged_text: conflicted,
                server_hash: remote.hash,
                auto_merged: false,
                binary: false,
            });
        }
    }

    Ok(report)
}

pub fn resolve_document_conflict(
    server_url: &str,
    token: &str,
    app: &tauri::AppHandle,
    store: &Store,
    path: &str,
    content: &str,
    server_hash: &str,
) -> Result<(), String> {
    let link = store
        .document_link(path)?
        .ok_or("This document is not linked to the cloud")?;

    let full = crate::workspace::workspace_path(app, store, path)?;
    std::fs::write(&full, content).map_err(|e| e.to_string())?;

    match push_document(server_url, token, &link.document_id, content, Some(server_hash))? {
        PushResult::Applied => store.save_document_link(
            path,
            &link.document_id,
            &content_hash(content.as_bytes()),
            &link.role,
            content,
        ),
        PushResult::Conflict { .. } => {
            Err("The document changed again in the cloud. Sync and merge once more.".to_string())
        }
    }
}

pub fn push_document(
    server_url: &str,
    token: &str,
    document_id: &str,
    content: &str,
    base_hash: Option<&str>,
) -> Result<PushResult, String> {
    let response = agent()
        .put(&endpoint(server_url, &format!("/documents/{}", document_id)))
        .set("Authorization", &format!("Bearer {}", token))
        .send_json(ureq::json!({
            "content": content,
            "base_hash": base_hash,
        }));

    match response {
        Ok(_) => Ok(PushResult::Applied),
        Err(ureq::Error::Status(409, body)) => {
            let conflict = body
                .into_json::<ConflictBody>()
                .map_err(|e| format!("Malformed conflict response: {}", e))?;
            Ok(PushResult::Conflict {
                server_hash: conflict.server_hash,
                server_text: conflict.server_content,
            })
        }
        Err(other) => Err(describe(other)),
    }
}

#[derive(Deserialize, Serialize, Clone)]
pub struct CloudFile {
    pub id: String,
    pub name: String,
    pub mime_type: String,
    pub folder_id: Option<String>,
    pub created_at: String,
}

#[derive(Deserialize)]
pub struct CloudFileContent {
    pub name: String,
    pub content: String,
}

pub fn list_account_files(
    server_url: &str,
    token: &str,
    folder_id: Option<&str>,
) -> Result<Vec<CloudFile>, String> {
    let mut request = agent()
        .get(&endpoint(server_url, "/files"))
        .set("Authorization", &format!("Bearer {}", token));

    if let Some(folder) = folder_id {
        request = request.query("folder_id", folder);
    }

    request
        .call()
        .map_err(describe)?
        .into_json::<Vec<CloudFile>>()
        .map_err(|e| e.to_string())
}

pub fn pull_account_file(
    server_url: &str,
    token: &str,
    file_id: &str,
) -> Result<CloudFileContent, String> {
    agent()
        .get(&endpoint(server_url, &format!("/files/{}", file_id)))
        .set("Authorization", &format!("Bearer {}", token))
        .call()
        .map_err(describe)?
        .into_json::<CloudFileContent>()
        .map_err(|e| e.to_string())
}
