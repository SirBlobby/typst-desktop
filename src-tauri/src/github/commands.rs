use std::collections::{BTreeMap, HashMap};
use std::path::PathBuf;

use serde::{Deserialize, Serialize};
use tauri::{AppHandle, State};

use super::import::{preview_import, ImportPreview};
use super::local_files;
use super::locks::lock_project;
use super::pull::{pull_into_project, PullInput, PullOutcome, Resolution};
use super::push::push_project;
use super::remote;
use super::repo_url::{clean_branch, clean_excluded_paths, clean_folder, clean_token, RepoUrl};
use super::storage::{GitLink, GithubAccount, LinkView};
use super::sync_plan::{as_list, local_changes, remote_changes, Change};
use crate::db::Store;
use crate::join_path;
use crate::workspace::{self, ProjectMeta};

const MAX_NAME_LENGTH: usize = 200;

#[derive(Serialize)]
pub struct AccountView {
    pub username: String,
}

#[derive(Deserialize)]
pub struct LinkRequest {
    pub repo_url: String,
    pub branch: String,
    pub folder: Option<String>,
    pub token: Option<String>,
    pub excluded_paths: Option<Vec<String>>,
    pub replace_project_files: Option<bool>,
}

#[derive(Deserialize)]
pub struct LinkUpdate {
    pub token: Option<String>,
    pub use_account: Option<bool>,
    pub excluded_paths: Option<Vec<String>>,
}

#[derive(Serialize)]
pub struct ChangesView {
    pub link: LinkView,
    pub head: String,
    pub behind: bool,
    pub local: Vec<Change>,
    pub remote: Vec<Change>,
    pub conflicts: Vec<String>,
}

#[derive(Serialize)]
pub struct PullResponse {
    pub link: LinkView,
    #[serde(flatten)]
    pub outcome: PullOutcome,
}

#[derive(Serialize)]
pub struct PushResponse {
    pub link: LinkView,
    pub commit: String,
    pub pushed: Vec<String>,
}

fn project_dir(app: &AppHandle, store: &Store, path: &str) -> Result<PathBuf, String> {
    let directory = workspace::workspace_path(app, store, path)?;
    if !directory.is_dir() || !workspace::is_project_dir(&directory) {
        return Err("Only projects can be linked to GitHub".to_string());
    }
    Ok(directory)
}

fn require_link(store: &Store, path: &str) -> Result<GitLink, String> {
    store
        .git_link(path)?
        .ok_or_else(|| "This project is not linked to a GitHub repository".to_string())
}

fn link_from_request(project_path: &str, request: &LinkRequest, account: Option<&GithubAccount>) -> Result<GitLink, String> {
    let url = RepoUrl::parse(&request.repo_url)?;
    let token = clean_token(request.token.as_deref());
    let uses_account = token.is_none() && account.is_some() && url.is_github_com();
    Ok(GitLink {
        project_path: project_path.to_string(),
        url,
        branch: clean_branch(&request.branch)?,
        folder: clean_folder(request.folder.as_deref().unwrap_or_default())?,
        token,
        uses_account,
        excluded_paths: clean_excluded_paths(request.excluded_paths.as_deref().unwrap_or_default())?,
        base_commit: None,
        last_synced_at: None,
    })
}

fn clean_name(name: &str) -> Result<String, String> {
    let name = name.trim();
    let has_bad_character = name.chars().any(|character| matches!(character, '/' | '\\' | ':' | '*' | '?' | '"' | '<' | '>' | '|'));
    if name.is_empty() || name.chars().count() > MAX_NAME_LENGTH || has_bad_character {
        return Err("Give the project a name of up to 200 characters without / \\ : * ? \" < > or |".to_string());
    }
    Ok(name.to_string())
}

fn link_and_import(store: &Store, project_dir: &std::path::Path, link: &GitLink, account: Option<&GithubAccount>, replace_project_files: bool) -> Result<(), String> {
    let repo = link.remote(account);
    let local = local_files::load(project_dir, &link.rules())?;
    let (preview, entries) = preview_import(&local, link, &repo, replace_project_files)?;
    store.save_git_link(link)?;
    let result = pull_into_project(PullInput {
        store,
        project_dir,
        link,
        repo: &repo,
        head: &preview.commit,
        entries,
        local,
        snapshot: HashMap::new(),
        resolution: Resolution::KeepRepository,
        remove_local_only: replace_project_files,
    });
    if let Err(error) = result {
        let _ = store.delete_git_link(&link.project_path);
        return Err(error);
    }
    Ok(())
}

#[tauri::command]
pub fn github_account(store: State<'_, Store>) -> Result<Option<AccountView>, String> {
    Ok(store.github_account()?.map(|account| AccountView { username: account.username }))
}

#[tauri::command(async)]
pub fn github_connect(store: State<'_, Store>, token: String) -> Result<AccountView, String> {
    let token = clean_token(Some(&token)).ok_or("Paste a GitHub access token")?;
    let username = remote::token_owner(&RepoUrl::github_com(), &token)?;
    store.save_github_account(Some(&GithubAccount { username: username.clone(), token }))?;
    Ok(AccountView { username })
}

#[tauri::command]
pub fn github_disconnect(store: State<'_, Store>) -> Result<(), String> {
    store.save_github_account(None)
}

#[tauri::command]
pub fn git_link_status(store: State<'_, Store>, path: String) -> Result<Option<LinkView>, String> {
    let account = store.github_account()?;
    Ok(store.git_link(&path)?.map(|link| link.view(account.as_ref())))
}

#[tauri::command]
pub fn git_linked_projects(store: State<'_, Store>) -> Result<Vec<String>, String> {
    store.git_linked_paths()
}

#[tauri::command(async)]
pub fn git_preview_link(app: AppHandle, store: State<'_, Store>, path: String, request: LinkRequest) -> Result<ImportPreview, String> {
    let directory = project_dir(&app, &store, &path)?;
    let account = store.github_account()?;
    let link = link_from_request(&path, &request, account.as_ref())?;
    let _lock = lock_project(&path)?;
    let local = local_files::load(&directory, &link.rules())?;
    let (preview, _) = preview_import(&local, &link, &link.remote(account.as_ref()), request.replace_project_files.unwrap_or(true))?;
    Ok(preview)
}

#[tauri::command(async)]
pub fn git_create_link(app: AppHandle, store: State<'_, Store>, path: String, request: LinkRequest) -> Result<LinkView, String> {
    let directory = project_dir(&app, &store, &path)?;
    let account = store.github_account()?;
    let link = link_from_request(&path, &request, account.as_ref())?;
    let _lock = lock_project(&path)?;
    if store.git_link(&path)?.is_some() {
        return Err("This project is already linked to a GitHub repository".to_string());
    }
    link_and_import(&store, &directory, &link, account.as_ref(), request.replace_project_files.unwrap_or(true))?;
    Ok(require_link(&store, &path)?.view(account.as_ref()))
}

#[tauri::command]
pub fn git_update_link(store: State<'_, Store>, path: String, update: LinkUpdate) -> Result<LinkView, String> {
    let _lock = lock_project(&path)?;
    let account = store.github_account()?;
    let mut link = require_link(&store, &path)?;

    if let Some(token) = &update.token {
        link.token = clean_token(Some(token));
        link.uses_account = false;
    }
    if update.use_account == Some(true) {
        if account.is_none() || !link.url.is_github_com() {
            return Err("Connect a GitHub account in Settings first".to_string());
        }
        link.token = None;
        link.uses_account = true;
    }
    if let Some(excluded_paths) = &update.excluded_paths {
        link.excluded_paths = clean_excluded_paths(excluded_paths)?;
    }
    store.save_git_link(&link)?;
    Ok(link.view(account.as_ref()))
}

#[tauri::command]
pub fn git_delete_link(store: State<'_, Store>, path: String) -> Result<(), String> {
    let _lock = lock_project(&path)?;
    store.delete_git_link(&path)
}

#[tauri::command(async)]
pub fn git_changes(app: AppHandle, store: State<'_, Store>, path: String) -> Result<ChangesView, String> {
    let directory = project_dir(&app, &store, &path)?;
    let account = store.github_account()?;
    let link = require_link(&store, &path)?;
    let _lock = lock_project(&path)?;

    let repo = link.remote(account.as_ref());
    let rules = link.rules();
    let head = remote::head(&repo)?;
    let behind = link.base_commit.as_deref() != Some(head.as_str());
    let saved = store.git_snapshot(&path)?;
    let local_changed = local_changes(&local_files::load(&directory, &rules)?, &saved, &rules);
    let remote_changed = if behind {
        remote_changes(&remote::list_files(&repo, &head)?, &saved, &rules)
    } else {
        BTreeMap::new()
    };
    let conflicts = local_changed.keys().filter(|path| remote_changed.contains_key(*path)).cloned().collect();

    Ok(ChangesView {
        link: link.view(account.as_ref()),
        head,
        behind,
        local: as_list(&local_changed),
        remote: as_list(&remote_changed),
        conflicts,
    })
}

#[tauri::command(async)]
pub fn git_pull(app: AppHandle, store: State<'_, Store>, path: String, resolution: Option<Resolution>) -> Result<PullResponse, String> {
    let directory = project_dir(&app, &store, &path)?;
    let account = store.github_account()?;
    let link = require_link(&store, &path)?;
    let _lock = lock_project(&path)?;

    let repo = link.remote(account.as_ref());
    let head = remote::head(&repo)?;
    if link.base_commit.as_deref() == Some(head.as_str()) {
        let outcome = PullOutcome { applied: true, ..PullOutcome::default() };
        return Ok(PullResponse { link: link.view(account.as_ref()), outcome });
    }

    let rules = link.rules();
    let outcome = pull_into_project(PullInput {
        store: &store,
        project_dir: &directory,
        link: &link,
        repo: &repo,
        head: &head,
        entries: remote::list_files(&repo, &head)?,
        local: local_files::load(&directory, &rules)?,
        snapshot: store.git_snapshot(&path)?,
        resolution: resolution.unwrap_or_default(),
        remove_local_only: false,
    })?;

    Ok(PullResponse { link: require_link(&store, &path)?.view(account.as_ref()), outcome })
}

#[tauri::command(async)]
pub fn git_push(app: AppHandle, store: State<'_, Store>, path: String, message: String, paths: Option<Vec<String>>) -> Result<PushResponse, String> {
    let directory = project_dir(&app, &store, &path)?;
    let account = store.github_account()?;
    let link = require_link(&store, &path)?;
    let _lock = lock_project(&path)?;

    let outcome = push_project(&store, &directory, &link, &link.remote(account.as_ref()), &message, paths)?;
    Ok(PushResponse {
        link: require_link(&store, &path)?.view(account.as_ref()),
        commit: outcome.commit,
        pushed: outcome.pushed,
    })
}

#[tauri::command(async)]
pub fn git_preview_import(store: State<'_, Store>, request: LinkRequest) -> Result<ImportPreview, String> {
    let account = store.github_account()?;
    let link = link_from_request("", &request, account.as_ref())?;
    let (preview, _) = preview_import(&HashMap::new(), &link, &link.remote(account.as_ref()), true)?;
    Ok(preview)
}

#[tauri::command(async)]
pub fn git_import_project(app: AppHandle, store: State<'_, Store>, parent: String, name: String, request: LinkRequest) -> Result<String, String> {
    let name = clean_name(&name)?;
    let path = join_path(&parent, &name);
    let directory = workspace::workspace_path(&app, &store, &path)?;
    if directory.exists() {
        return Err(format!("'{name}' already exists"));
    }
    let account = store.github_account()?;
    let link = link_from_request(&path, &request, account.as_ref())?;
    let _lock = lock_project(&path)?;

    std::fs::create_dir_all(&directory).map_err(|error| error.to_string())?;
    let created = workspace::ensure_project_marker(&directory)
        .and_then(|_| store.save_meta(&path, &ProjectMeta::default()))
        .and_then(|_| link_and_import(&store, &directory, &link, account.as_ref(), true));
    if let Err(error) = created {
        let _ = std::fs::remove_dir_all(&directory);
        let _ = store.forget_project(&path);
        return Err(error);
    }
    Ok(path)
}
