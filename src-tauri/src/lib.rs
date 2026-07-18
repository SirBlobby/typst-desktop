mod assets;
mod compiler;
mod db;
mod lsp;
mod sync;
mod thumbnails;
mod workspace;
mod world;

use base64::{engine::general_purpose::STANDARD as BASE64, Engine as _};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use tauri::{AppHandle, Manager, State};

use assets::Asset;
use db::Store;
use compiler::{CompileResult, Diagnostic};
use lsp::{LspHandle, LspState};
use sync::{Account, SpaceSummary, SyncReport};
use workspace::{
    browse, is_project_dir, is_text_file, list_files, load_settings, project_file_path,
    read_target_files, resolve_target, save_settings, workspace_path, BrowseEntry,
    FileEntry, ProjectMeta, Settings, NEW_PROJECT_MAIN,
};

#[derive(Serialize)]
pub struct CompileFailure {
    pub diagnostics: Vec<Diagnostic>,
}

fn failure(message: String) -> CompileFailure {
    CompileFailure {
        diagnostics: vec![Diagnostic {
            message,
            severity: "error".to_string(),
            line: None,
            column: None,
        }],
    }
}

fn cloud_credentials(app: &AppHandle, store: &Store) -> Result<(String, String), String> {
    let settings = load_settings(app, store)?;
    let token = settings.device_token.ok_or("Not signed in to TypstDrive")?;
    Ok((settings.server_url, token))
}

fn load_project(
    app: &AppHandle,
    store: &Store,
    project: &str,
) -> Result<(PathBuf, ProjectMeta), String> {
    let dir = workspace_path(app, store, project)?;
    if !dir.is_dir() {
        return Err(format!("Project '{}' not found", project));
    }
    Ok((dir, store.meta(project)?))
}

#[derive(Serialize)]
pub struct AppInfo {
    pub version: String,
    pub typst_version: String,
    pub authors: String,
    pub license: String,
    pub tauri_version: String,
}

#[tauri::command]
fn app_info() -> AppInfo {
    AppInfo {
        version: env!("CARGO_PKG_VERSION").to_string(),
        typst_version: "0.14.2".to_string(),
        authors: "SirBlobby".to_string(),
        license: "Apache-2.0".to_string(),
        tauri_version: tauri::VERSION.to_string(),
    }
}

#[tauri::command]
fn get_settings(app: AppHandle, store: State<'_, Store>) -> Result<Settings, String> {
    load_settings(&app, &store)
}

#[tauri::command]
fn update_settings(
    app: AppHandle,
    store: State<'_, Store>,
    workspace_root: Option<String>,
    server_url: Option<String>,
    autosave_seconds: Option<u32>,
    sync_minutes: Option<u32>,
) -> Result<Settings, String> {
    let mut settings = load_settings(&app, &store)?;
    if let Some(root) = workspace_root {
        if root.trim().is_empty() {
            return Err("Workspace folder cannot be empty".to_string());
        }
        settings.workspace_root = root;
    }
    if let Some(url) = server_url {
        settings.server_url = url.trim_end_matches('/').to_string();
    }
    if let Some(seconds) = autosave_seconds {
        settings.autosave_seconds = seconds;
    }
    if let Some(minutes) = sync_minutes {
        settings.sync_minutes = minutes;
    }
    save_settings(&store, &settings)?;
    Ok(settings)
}

#[tauri::command]
fn browse_workspace(app: AppHandle, store: State<'_, Store>, path: String) -> Result<Vec<BrowseEntry>, String> {
    browse(&app, &store, &path)
}

fn parent_of(path: &str) -> String {
    match path.rsplit_once('/') {
        Some((parent, _)) => parent.to_string(),
        None => String::new(),
    }
}

fn join_path(parent: &str, name: &str) -> String {
    if parent.is_empty() {
        name.to_string()
    } else {
        format!("{}/{}", parent.trim_end_matches('/'), name)
    }
}

#[tauri::command]
fn create_folder_entry(app: AppHandle, store: State<'_, Store>, parent: String, name: String) -> Result<String, String> {
    let path = join_path(&parent, name.trim());
    let full = workspace_path(&app, &store, &path)?;
    if full.exists() {
        return Err(format!("'{}' already exists", name));
    }
    std::fs::create_dir_all(&full).map_err(|e| e.to_string())?;
    Ok(path)
}

#[tauri::command]
fn create_document_entry(app: AppHandle, store: State<'_, Store>, parent: String, name: String) -> Result<String, String> {
    let mut name = name.trim().to_string();
    if name.is_empty() {
        return Err("Document name cannot be empty".to_string());
    }
    if !name.to_lowercase().ends_with(".typ") {
        name.push_str(".typ");
    }

    let path = join_path(&parent, &name);
    let full = workspace_path(&app, &store, &path)?;
    if full.exists() {
        return Err(format!("'{}' already exists", name));
    }
    if let Some(dir) = full.parent() {
        std::fs::create_dir_all(dir).map_err(|e| e.to_string())?;
    }

    let title = name.trim_end_matches(".typ").trim_end_matches(".TYP");
    std::fs::write(&full, format!("= {}\n\nStart writing here.\n", title))
        .map_err(|e| e.to_string())?;

    Ok(path)
}

#[tauri::command]
fn create_project_entry(app: AppHandle, store: State<'_, Store>, parent: String, name: String) -> Result<String, String> {
    let name = name.trim().to_string();
    if name.is_empty() {
        return Err("Project name cannot be empty".to_string());
    }

    let path = join_path(&parent, &name);
    let full = workspace_path(&app, &store, &path)?;
    if full.exists() {
        return Err(format!("'{}' already exists", name));
    }

    std::fs::create_dir_all(&full).map_err(|e| e.to_string())?;
    std::fs::write(full.join("main.typ"), NEW_PROJECT_MAIN).map_err(|e| e.to_string())?;
    std::fs::write(full.join("typst.toml"), workspace::manifest_for(&name))
        .map_err(|e| e.to_string())?;

    store.save_meta(
        &path,
        &ProjectMeta {
            entrypoint: "main.typ".to_string(),
            ..Default::default()
        },
    )?;

    Ok(path)
}

#[tauri::command]
fn rename_entry(app: AppHandle, store: State<'_, Store>, path: String, new_name: String) -> Result<String, String> {
    let new_name = new_name.trim();
    if new_name.is_empty() || new_name.contains('/') || new_name.contains('\\') {
        return Err("Invalid name".to_string());
    }

    let from = workspace_path(&app, &store, &path)?;
    let target_path = join_path(&parent_of(&path), new_name);
    let to = workspace_path(&app, &store, &target_path)?;

    if to.exists() {
        return Err(format!("'{}' already exists", new_name));
    }
    std::fs::rename(&from, &to).map_err(|e| e.to_string())?;
    store.rename_project(&path, &target_path)?;
    Ok(target_path)
}

#[tauri::command]
fn delete_entry(app: AppHandle, store: State<'_, Store>, path: String) -> Result<(), String> {
    let full = workspace_path(&app, &store, &path)?;
    if full.is_dir() {
        std::fs::remove_dir_all(&full).map_err(|e| e.to_string())?;
        store.forget_project(&path)?;
        Ok(())
    } else {
        std::fs::remove_file(&full).map_err(|e| e.to_string())
    }
}

/// Moves an entry into another folder, keeping its name.
#[tauri::command]
fn move_entry(
    app: AppHandle,
    store: State<'_, Store>,
    path: String,
    destination: String,
) -> Result<String, String> {
    let name = path
        .rsplit('/')
        .next()
        .ok_or("Invalid path")?
        .to_string();

    let target_path = join_path(&destination, &name);
    if target_path == path {
        return Ok(path);
    }
    if destination == path || destination.starts_with(&format!("{}/", path)) {
        return Err("A folder cannot be moved into itself".to_string());
    }

    let from = workspace_path(&app, &store, &path)?;
    let to = workspace_path(&app, &store, &target_path)?;

    if to.exists() {
        return Err(format!("'{}' already exists there", name));
    }
    if let Some(parent) = to.parent() {
        std::fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    }

    std::fs::rename(&from, &to).map_err(|e| e.to_string())?;
    store.rename_project(&path, &target_path)?;
    Ok(target_path)
}

#[tauri::command]
fn duplicate_entry(
    app: AppHandle,
    store: State<'_, Store>,
    path: String,
) -> Result<String, String> {
    let full = workspace_path(&app, &store, &path)?;
    let parent = parent_of(&path);

    let name = path.rsplit('/').next().ok_or("Invalid path")?;
    let (stem, extension) = match name.rsplit_once('.') {
        Some((stem, ext)) if !stem.is_empty() => (stem.to_string(), format!(".{}", ext)),
        _ => (name.to_string(), String::new()),
    };

    let mut candidate = String::new();
    for index in 1..1000 {
        let suffix = if index == 1 {
            " copy".to_string()
        } else {
            format!(" copy {}", index)
        };
        let attempt = join_path(&parent, &format!("{}{}{}", stem, suffix, extension));
        if !workspace_path(&app, &store, &attempt)?.exists() {
            candidate = attempt;
            break;
        }
    }

    if candidate.is_empty() {
        return Err("Could not find a free name".to_string());
    }

    let to = workspace_path(&app, &store, &candidate)?;
    if full.is_dir() {
        assets::import_paths(&[full.to_string_lossy().to_string()], &to)?;
    } else {
        std::fs::copy(&full, &to).map_err(|e| e.to_string())?;
    }

    Ok(candidate)
}

/// Absolute path on disk, used to reveal an entry in the system file manager.
#[tauri::command]
fn absolute_path(
    app: AppHandle,
    store: State<'_, Store>,
    path: String,
) -> Result<String, String> {
    Ok(workspace_path(&app, &store, &path)?
        .to_string_lossy()
        .to_string())
}

#[tauri::command]
fn upload_entry(
    app: AppHandle,
    store: State<'_, Store>,
    parent: String,
    name: String,
    base64_content: String,
) -> Result<String, String> {
    let path = join_path(&parent, &name);
    let full = workspace_path(&app, &store, &path)?;
    let bytes = BASE64
        .decode(base64_content.as_bytes())
        .map_err(|e| format!("Invalid file data: {}", e))?;
    if let Some(dir) = full.parent() {
        std::fs::create_dir_all(dir).map_err(|e| e.to_string())?;
    }
    std::fs::write(&full, bytes).map_err(|e| e.to_string())?;
    Ok(path)
}

#[derive(Serialize)]
pub struct FilePayload {
    pub path: String,
    pub is_text: bool,
    pub content: String,
}

#[derive(Serialize)]
pub struct TargetInfo {
    pub path: String,
    pub entrypoint: String,
    pub standalone: bool,
    pub is_project: bool,
    pub space_id: Option<String>,
    pub files: Vec<FileEntry>,
}

#[tauri::command]
fn target_info(app: AppHandle, store: State<'_, Store>, path: String) -> Result<TargetInfo, String> {
    let target = resolve_target(&app, &store, &path)?;

    if target.standalone {
        let size = std::fs::metadata(target.root.join(&target.entrypoint))
            .map(|m| m.len())
            .unwrap_or(0);
        return Ok(TargetInfo {
            path,
            entrypoint: target.entrypoint.clone(),
            standalone: true,
            is_project: false,
            space_id: None,
            files: vec![FileEntry {
                path: target.entrypoint.clone(),
                name: target.entrypoint,
                is_dir: false,
                is_text: true,
                size,
            }],
        });
    }

    let meta = store.meta(&path)?;
    Ok(TargetInfo {
        path,
        entrypoint: target.entrypoint,
        standalone: false,
        is_project: is_project_dir(&target.root),
        space_id: meta.space_id,
        files: list_files(&target.root)?,
    })
}

#[tauri::command]
fn read_target_file(
    app: AppHandle,
    store: State<'_, Store>,
    path: String,
    file: String,
) -> Result<FilePayload, String> {
    let target = resolve_target(&app, &store, &path)?;
    let full = project_file_path(&target.root, &file)?;
    let bytes = std::fs::read(&full).map_err(|e| e.to_string())?;

    let is_text = is_text_file(&file);
    Ok(FilePayload {
        path: file,
        is_text,
        content: if is_text {
            String::from_utf8_lossy(&bytes).to_string()
        } else {
            BASE64.encode(&bytes)
        },
    })
}

#[tauri::command]
fn write_target_file(
    app: AppHandle,
    store: State<'_, Store>,
    path: String,
    file: String,
    content: String,
) -> Result<(), String> {
    let target = resolve_target(&app, &store, &path)?;
    let full = project_file_path(&target.root, &file)?;
    if let Some(dir) = full.parent() {
        std::fs::create_dir_all(dir).map_err(|e| e.to_string())?;
    }
    std::fs::write(&full, content).map_err(|e| e.to_string())
}

#[tauri::command]
fn set_target_entrypoint(
    app: AppHandle,
    store: State<'_, Store>,
    path: String,
    entrypoint: String,
) -> Result<(), String> {
    let target = resolve_target(&app, &store, &path)?;
    if target.standalone {
        return Err("A standalone document is its own entrypoint".to_string());
    }
    let mut meta = store.meta(&path)?;
    meta.entrypoint = entrypoint;
    store.save_meta(&path, &meta)
}

#[tauri::command]
fn compile_target(
    app: AppHandle,
    store: State<'_, Store>,
    path: String,
    entrypoint: Option<String>,
    overrides: Option<std::collections::HashMap<String, String>>,
) -> Result<CompileResult, CompileFailure> {
    let target = resolve_target(&app, &store, &path).map_err(failure)?;
    let mut files = read_target_files(&app, &store, &target).map_err(failure)?;

    for (file, content) in overrides.unwrap_or_default() {
        files.insert(file, content.into_bytes());
    }

    let entrypoint = match entrypoint {
        Some(file) if files.contains_key(&file) => file,
        _ => target.entrypoint,
    };

    compiler::compile_to_svg(entrypoint, files)
        .map_err(|diagnostics| CompileFailure { diagnostics })
}

#[tauri::command]
fn export_target(
    app: AppHandle,
    store: State<'_, Store>,
    path: String,
    format: String,
    destination: String,
) -> Result<String, String> {
    let target = resolve_target(&app, &store, &path)?;
    let files = read_target_files(&app, &store, &target)?;

    let bytes = match format.as_str() {
        "pdf" => compiler::export_pdf(target.entrypoint, files),
        "png" => compiler::export_png(target.entrypoint, files),
        "html" => compiler::export_html(target.entrypoint, files),
        other => return Err(format!("Unsupported export format '{}'", other)),
    }
    .map_err(|diagnostics| {
        diagnostics
            .into_iter()
            .map(|d| d.message)
            .collect::<Vec<_>>()
            .join("; ")
    })?;

    std::fs::write(&destination, bytes).map_err(|e| e.to_string())?;
    Ok(destination)
}

#[tauri::command]
fn thumbnail(app: AppHandle, store: State<'_, Store>, path: String) -> Result<thumbnails::Thumbnail, String> {
    thumbnails::thumbnail(&app, &store, &path)
}

#[tauri::command]
fn read_image(
    app: AppHandle,
    store: State<'_, Store>,
    path: String,
) -> Result<thumbnails::ImageData, String> {
    thumbnails::read_image(&app, &store, &path)
}

#[tauri::command]
fn clear_thumbnails(store: State<'_, Store>) -> Result<(), String> {
    store.clear_thumbnails()
}

#[tauri::command]
fn list_assets(app: AppHandle, store: State<'_, Store>) -> Result<Vec<Asset>, String> {
    assets::list_assets(&app, &store)
}

#[derive(Serialize)]
pub struct Resource {
    pub name: String,
    pub reference: String,
    pub path: String,
    pub scope: String,
    pub kind: String,
    pub size: u64,
    pub font_families: Vec<String>,
}

fn resource_kind(name: &str) -> String {
    if assets::is_font(name) {
        "font"
    } else if assets::is_image(name) {
        "image"
    } else {
        "file"
    }
    .to_string()
}

#[tauri::command]
fn list_resources(
    app: AppHandle,
    store: State<'_, Store>,
    path: String,
) -> Result<Vec<Resource>, String> {
    let mut resources = Vec::new();

    for asset in assets::list_assets(&app, &store)? {
        resources.push(Resource {
            reference: asset.name.clone(),
            path: format!("{}/{}", assets::ASSETS_DIR, asset.name),
            scope: "shared".to_string(),
            kind: asset.kind,
            size: asset.size,
            font_families: asset.font_families,
            name: asset.name,
        });
    }

    let target = resolve_target(&app, &store, &path)?;
    let prefix = if target.standalone {
        path.rsplit_once('/').map(|(dir, _)| dir).unwrap_or("")
    } else {
        path.as_str()
    };

    for file in list_files(&target.root)? {
        if file.is_dir || file.path.to_lowercase().ends_with(".typ") {
            continue;
        }

        let full = target.root.join(&file.path);
        let font_families = if assets::is_font(&file.name) {
            std::fs::read(&full)
                .map(|data| assets::families_in(&data))
                .unwrap_or_default()
        } else {
            Vec::new()
        };

        resources.push(Resource {
            name: file.name,
            reference: file.path.clone(),
            path: if prefix.is_empty() {
                file.path.clone()
            } else {
                format!("{}/{}", prefix, file.path)
            },
            scope: "project".to_string(),
            kind: resource_kind(&file.path),
            size: file.size,
            font_families,
        });
    }

    Ok(resources)
}

#[tauri::command]
fn list_font_families(app: AppHandle, store: State<'_, Store>, path: Option<String>) -> Result<Vec<String>, String> {
    let files = match path {
        Some(path) if !path.is_empty() => {
            let target = resolve_target(&app, &store, &path)?;
            read_target_files(&app, &store, &target)?
        }
        _ => assets::asset_files(&app, &store),
    };

    Ok(assets::font_families(&files))
}

#[tauri::command]
fn import_assets(app: AppHandle, store: State<'_, Store>, sources: Vec<String>) -> Result<Vec<String>, String> {
    let destination = assets::assets_dir(&app, &store)?;
    assets::import_files(&sources, &destination)
}

#[tauri::command]
fn delete_asset(app: AppHandle, store: State<'_, Store>, name: String) -> Result<(), String> {
    assets::delete_asset(&app, &store, &name)
}

#[tauri::command]
fn import_into_target(
    app: AppHandle,
    store: State<'_, Store>,
    path: String,
    sources: Vec<String>,
) -> Result<Vec<String>, String> {
    let target = resolve_target(&app, &store, &path)?;
    assets::import_paths(&sources, &target.root)
}

#[tauri::command]
fn import_into_folder(
    app: AppHandle,
    store: State<'_, Store>,
    parent: String,
    sources: Vec<String>,
) -> Result<Vec<String>, String> {
    let destination = workspace_path(&app, &store, &parent)?;
    assets::import_paths(&sources, &destination)
}

#[tauri::command]
fn lsp_start(
    app: AppHandle,
    store: State<'_, Store>,
    state: State<'_, LspState>,
    path: String,
) -> Result<LspHandle, String> {
    let target = resolve_target(&app, &store, &path)?;
    state.start(&app, &target.root, &target.entrypoint)
}

#[tauri::command]
fn lsp_send(state: State<'_, LspState>, message: String) -> Result<(), String> {
    state.send(&message)
}

#[tauri::command]
fn lsp_stop(state: State<'_, LspState>) {
    state.stop();
}

#[tauri::command]
fn lsp_running(state: State<'_, LspState>) -> bool {
    state.is_running()
}

#[tauri::command]
fn cloud_login(
    app: AppHandle,
    store: State<'_, Store>,
    server_url: String,
    email: String,
    password: String,
) -> Result<Account, String> {
    let server_url = server_url.trim_end_matches('/').to_string();
    let device_name = format!("Typst Desktop ({})", std::env::consts::OS);
    let response = sync::login(&server_url, &email, &password, &device_name)?;

    let mut settings = load_settings(&app, &store)?;
    settings.server_url = server_url;
    settings.device_token = Some(response.token);
    settings.account_email = Some(response.email.clone());
    settings.account_username = Some(response.username.clone());
    save_settings(&store, &settings)?;

    Ok(Account {
        user_id: response.user_id,
        username: response.username,
        email: response.email,
    })
}

#[tauri::command]
fn cloud_logout(app: AppHandle, store: State<'_, Store>) -> Result<(), String> {
    let mut settings = load_settings(&app, &store)?;
    if let Some(token) = &settings.device_token {
        let _ = sync::logout(&settings.server_url, token);
    }

    settings.device_token = None;
    settings.account_email = None;
    settings.account_username = None;
    save_settings(&store, &settings)
}

#[tauri::command]
fn cloud_account(app: AppHandle, store: State<'_, Store>) -> Result<Option<Account>, String> {
    let settings = load_settings(&app, &store)?;
    let Some(token) = settings.device_token else {
        return Ok(None);
    };

    Ok(sync::me(&settings.server_url, &token).ok())
}

#[tauri::command]
fn cloud_list_spaces(app: AppHandle, store: State<'_, Store>) -> Result<Vec<SpaceSummary>, String> {
    let (server_url, token) = cloud_credentials(&app, &store)?;
    sync::list_spaces(&server_url, &token)
}

#[tauri::command]
fn cloud_list_folders(
    app: AppHandle,
    store: State<'_, Store>,
) -> Result<Vec<sync::CloudFolder>, String> {
    let (server_url, token) = cloud_credentials(&app, &store)?;
    sync::list_folders(&server_url, &token)
}

#[tauri::command]
fn cloud_list_documents(
    app: AppHandle,
    store: State<'_, Store>,
    folder_id: Option<String>,
) -> Result<Vec<sync::CloudDocument>, String> {
    let (server_url, token) = cloud_credentials(&app, &store)?;
    sync::list_documents(&server_url, &token, folder_id.as_deref())
}

#[tauri::command]
fn cloud_list_files(
    app: AppHandle,
    store: State<'_, Store>,
    folder_id: Option<String>,
) -> Result<Vec<sync::CloudFile>, String> {
    let (server_url, token) = cloud_credentials(&app, &store)?;
    sync::list_account_files(&server_url, &token, folder_id.as_deref())
}

/// Downloads an account file into the shared asset library, where every
/// project can reference it by name.
#[tauri::command]
fn cloud_download_file(
    app: AppHandle,
    store: State<'_, Store>,
    file_id: String,
) -> Result<String, String> {
    let (server_url, token) = cloud_credentials(&app, &store)?;
    let file = sync::pull_account_file(&server_url, &token, &file_id)?;

    let bytes = BASE64
        .decode(file.content.as_bytes())
        .map_err(|e| format!("Invalid file data: {}", e))?;

    let destination = assets::assets_dir(&app, &store)?.join(&file.name);
    std::fs::write(&destination, bytes).map_err(|e| e.to_string())?;

    Ok(file.name)
}

#[tauri::command]
fn cloud_list_shared(
    app: AppHandle,
    store: State<'_, Store>,
) -> Result<sync::SharedItems, String> {
    let (server_url, token) = cloud_credentials(&app, &store)?;
    sync::list_shared(&server_url, &token)
}

/// Downloads a cloud document into the workspace and remembers where it came
/// from so it can be synced back.
#[tauri::command]
fn cloud_download_document(
    app: AppHandle,
    store: State<'_, Store>,
    document_id: String,
    parent: String,
) -> Result<String, String> {
    let (server_url, token) = cloud_credentials(&app, &store)?;
    let document = sync::pull_document(&server_url, &token, &document_id)?;

    let mut name = document.title.replace('/', "-").trim().to_string();
    if name.is_empty() {
        name = "document".to_string();
    }
    if !name.to_lowercase().ends_with(".typ") {
        name.push_str(".typ");
    }

    let path = join_path(&parent, &name);
    let full = workspace_path(&app, &store, &path)?;
    if let Some(dir) = full.parent() {
        std::fs::create_dir_all(dir).map_err(|e| e.to_string())?;
    }
    std::fs::write(&full, &document.content).map_err(|e| e.to_string())?;

    store.save_document_link(
        &path,
        &document_id,
        &document.hash,
        &document.role,
        &document.content,
    )?;

    Ok(path)
}

#[tauri::command]
fn cloud_sync_document(
    app: AppHandle,
    store: State<'_, Store>,
    path: String,
) -> Result<SyncReport, String> {
    let (server_url, token) = cloud_credentials(&app, &store)?;
    sync::sync_document(&server_url, &token, &app, &store, &path)
}

#[tauri::command]
fn cloud_resolve_document(
    app: AppHandle,
    store: State<'_, Store>,
    path: String,
    content: String,
    server_hash: String,
) -> Result<(), String> {
    let (server_url, token) = cloud_credentials(&app, &store)?;
    sync::resolve_document_conflict(
        &server_url,
        &token,
        &app,
        &store,
        &path,
        &content,
        &server_hash,
    )
}

#[tauri::command]
fn cloud_unlink_document(store: State<'_, Store>, path: String) -> Result<(), String> {
    store.forget_document_link(&path)
}

#[derive(Serialize)]
pub struct LinkedDocument {
    pub path: String,
    pub document_id: String,
    pub synced_at: Option<String>,
    pub sync_state: Option<String>,
}

/// Every cloud document that has been downloaded, so the cloud view can show
/// which ones live on this device and whether they are up to date.
#[tauri::command]
fn cloud_linked_documents(
    app: AppHandle,
    store: State<'_, Store>,
) -> Result<Vec<LinkedDocument>, String> {
    let mut linked = Vec::new();

    for (path, document_id, synced_at) in store.all_document_links()? {
        let Ok(full) = workspace_path(&app, &store, &path) else {
            continue;
        };
        if !full.is_file() {
            continue;
        }

        linked.push(LinkedDocument {
            sync_state: workspace::sync_state_of(&full, synced_at.as_deref()),
            path,
            document_id,
            synced_at,
        });
    }

    Ok(linked)
}

#[derive(Serialize)]
pub struct LinkedSpace {
    pub path: String,
    pub space_id: String,
    pub synced_at: Option<String>,
    pub sync_state: Option<String>,
}

/// Cloud spaces that have been downloaded, wherever they sit in the workspace.
#[tauri::command]
fn cloud_linked_spaces(
    app: AppHandle,
    store: State<'_, Store>,
) -> Result<Vec<LinkedSpace>, String> {
    let mut linked = Vec::new();

    for (path, space_id, synced_at) in store.all_space_links()? {
        let Ok(full) = workspace_path(&app, &store, &path) else {
            continue;
        };
        if !full.is_dir() {
            continue;
        }

        linked.push(LinkedSpace {
            sync_state: workspace::project_sync_state(&full, synced_at.as_deref()),
            path,
            space_id,
            synced_at,
        });
    }

    Ok(linked)
}

#[tauri::command]
fn cloud_document_link(
    store: State<'_, Store>,
    path: String,
) -> Result<Option<db::DocumentLink>, String> {
    store.document_link(&path)
}

#[tauri::command]
fn cloud_create_space(app: AppHandle, store: State<'_, Store>, name: String) -> Result<SpaceSummary, String> {
    let (server_url, token) = cloud_credentials(&app, &store)?;
    sync::create_space(&server_url, &token, name.trim())
}

#[tauri::command]
fn cloud_delete_space(app: AppHandle, store: State<'_, Store>, space_id: String) -> Result<(), String> {
    let (server_url, token) = cloud_credentials(&app, &store)?;
    sync::delete_space(&server_url, &token, &space_id)
}

#[tauri::command]
fn cloud_clone_space(
    app: AppHandle,
    store: State<'_, Store>,
    space_id: String,
    project_name: String,
) -> Result<SyncReport, String> {
    let (server_url, token) = cloud_credentials(&app, &store)?;
    let project = project_name.trim().to_string();
    let dir = workspace_path(&app, &store, &project)?;
    if dir.exists() {
        return Err(format!("A project named '{}' already exists", project_name));
    }
    sync::clone_space(&server_url, &token, &store, &project, &dir, &space_id)
}

#[tauri::command]
fn cloud_link_project(
    app: AppHandle,
    store: State<'_, Store>,
    project: String,
    space_id: Option<String>,
) -> Result<SyncReport, String> {
    let (server_url, token) = cloud_credentials(&app, &store)?;
    let (dir, mut meta) = load_project(&app, &store, &project)?;

    let space_id = match space_id {
        Some(id) if !id.trim().is_empty() => id,
        _ => sync::create_space(&server_url, &token, &project)?.id,
    };

    meta.space_id = Some(space_id);
    meta.base_hashes.clear();
    store.save_meta(&project, &meta)?;

    sync::push_project(&server_url, &token, &store, &project, &dir, &mut meta)
}

#[tauri::command]
fn cloud_unlink_project(app: AppHandle, store: State<'_, Store>, project: String) -> Result<(), String> {
    let (dir, mut meta) = load_project(&app, &store, &project)?;
    meta.space_id = None;
    meta.base_hashes.clear();
    meta.last_synced_at = None;
    let _ = dir;
    store.forget_project(&project)
}

#[tauri::command]
fn cloud_push(app: AppHandle, store: State<'_, Store>, project: String) -> Result<SyncReport, String> {
    let (server_url, token) = cloud_credentials(&app, &store)?;
    let (dir, mut meta) = load_project(&app, &store, &project)?;
    sync::push_project(&server_url, &token, &store, &project, &dir, &mut meta)
}

#[tauri::command]
fn cloud_pull(app: AppHandle, store: State<'_, Store>, project: String) -> Result<SyncReport, String> {
    let (server_url, token) = cloud_credentials(&app, &store)?;
    let (dir, mut meta) = load_project(&app, &store, &project)?;
    sync::pull_project(&server_url, &token, &store, &project, &dir, &mut meta)
}

#[tauri::command]
fn cloud_sync(app: AppHandle, store: State<'_, Store>, project: String) -> Result<SyncReport, String> {
    let (server_url, token) = cloud_credentials(&app, &store)?;
    let (dir, mut meta) = load_project(&app, &store, &project)?;

    let mut report = sync::pull_project(&server_url, &token, &store, &project, &dir, &mut meta)?;
    if !report.conflicts.is_empty() {
        return Ok(report);
    }

    let pushed = sync::push_project(&server_url, &token, &store, &project, &dir, &mut meta)?;
    report.pushed = pushed.pushed;
    report.deleted_remote = pushed.deleted_remote;
    report.conflicts = pushed.conflicts;

    Ok(report)
}

#[derive(Deserialize)]
pub struct ResolutionRequest {
    pub path: String,
    pub content: String,
    pub server_hash: String,
}

#[tauri::command]
fn cloud_resolve_conflicts(
    app: AppHandle,
    store: State<'_, Store>,
    project: String,
    resolutions: Vec<ResolutionRequest>,
) -> Result<SyncReport, String> {
    let (server_url, token) = cloud_credentials(&app, &store)?;
    let (dir, mut meta) = load_project(&app, &store, &project)?;

    for resolution in &resolutions {
        sync::resolve_conflict(
            &store,
            &project,
            &dir,
            &mut meta,
            &resolution.path,
            &resolution.content,
            &resolution.server_hash,
        )?;
    }

    sync::push_project(&server_url, &token, &store, &project, &dir, &mut meta)
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .setup(|app| {
            let store = Store::open(&app.handle())?;
            app.manage(store);
            Ok(())
        })
        .manage(LspState::default())
        .on_window_event(|window, event| {
            if matches!(event, tauri::WindowEvent::Destroyed) {
                if let Some(state) = window.app_handle().try_state::<LspState>() {
                    state.stop();
                }
            }
        })
        .invoke_handler(tauri::generate_handler![
            app_info,
            get_settings,
            update_settings,
            browse_workspace,
            create_folder_entry,
            create_document_entry,
            create_project_entry,
            rename_entry,
            move_entry,
            duplicate_entry,
            absolute_path,
            delete_entry,
            upload_entry,
            target_info,
            read_target_file,
            write_target_file,
            set_target_entrypoint,
            compile_target,
            export_target,
            thumbnail,
            read_image,
            clear_thumbnails,
            list_assets,
            list_resources,
            list_font_families,
            import_assets,
            delete_asset,
            import_into_target,
            import_into_folder,
            lsp_start,
            lsp_send,
            lsp_stop,
            lsp_running,
            cloud_login,
            cloud_logout,
            cloud_account,
            cloud_list_spaces,
            cloud_list_folders,
            cloud_list_documents,
            cloud_list_shared,
            cloud_list_files,
            cloud_download_file,
            cloud_download_document,
            cloud_sync_document,
            cloud_resolve_document,
            cloud_document_link,
            cloud_linked_documents,
            cloud_linked_spaces,
            cloud_unlink_document,
            cloud_create_space,
            cloud_delete_space,
            cloud_clone_space,
            cloud_link_project,
            cloud_unlink_project,
            cloud_push,
            cloud_pull,
            cloud_sync,
            cloud_resolve_conflicts,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
