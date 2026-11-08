use std::collections::HashMap;

use rusqlite::{params, OptionalExtension};
use serde::{Deserialize, Serialize};

use super::file_rules::FileRules;
use super::remote::RemoteRepo;
use super::repo_url::RepoUrl;
use crate::db::Store;

const ACCOUNT_SETTING: &str = "github";

#[derive(Serialize, Deserialize, Clone)]
pub struct GithubAccount {
    pub username: String,
    pub token: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SnapshotEntry {
    pub blob_sha: String,
    pub content_hash: String,
}

pub type Snapshot = HashMap<String, SnapshotEntry>;

#[derive(Default)]
pub struct SnapshotUpdate {
    pub saved: Vec<(String, SnapshotEntry)>,
    pub removed: Vec<String>,
}

pub struct GitLink {
    pub project_path: String,
    pub url: RepoUrl,
    pub branch: String,
    pub folder: String,
    pub token: Option<String>,
    pub uses_account: bool,
    pub excluded_paths: Vec<String>,
    pub base_commit: Option<String>,
    pub last_synced_at: Option<String>,
}

#[derive(Serialize)]
pub struct LinkView {
    pub repo_url: String,
    pub branch: String,
    pub folder: String,
    pub has_token: bool,
    pub account_username: Option<String>,
    pub excluded_paths: Vec<String>,
    pub base_commit: Option<String>,
    pub last_synced_at: Option<String>,
}

type LinkRow = (String, String, String, String, Option<String>, bool, Option<String>, Option<String>, Option<String>);

const LINK_COLUMNS: &str = "project_path, repo_url, branch, folder, token, uses_account, excluded_paths, base_commit, last_synced_at";

fn split_paths(stored: Option<String>) -> Vec<String> {
    stored
        .unwrap_or_default()
        .lines()
        .map(str::to_string)
        .filter(|line| !line.is_empty())
        .collect()
}

fn link_from_row(row: LinkRow) -> Result<GitLink, String> {
    let (project_path, repo_url, branch, folder, token, uses_account, excluded_paths, base_commit, last_synced_at) = row;
    let url = RepoUrl::parse(&repo_url).map_err(|_| "The saved GitHub link is damaged. Unlink and link the repository again.".to_string())?;
    Ok(GitLink {
        project_path,
        url,
        branch,
        folder,
        token,
        uses_account,
        excluded_paths: split_paths(excluded_paths),
        base_commit,
        last_synced_at,
    })
}

impl GitLink {
    pub fn rules(&self) -> FileRules {
        FileRules::new(self.excluded_paths.clone())
    }

    fn effective_token(&self, account: Option<&GithubAccount>) -> Option<String> {
        if self.uses_account {
            account.filter(|_| self.url.is_github_com()).map(|account| account.token.clone())
        } else {
            self.token.clone()
        }
    }

    pub fn remote(&self, account: Option<&GithubAccount>) -> RemoteRepo {
        RemoteRepo {
            url: self.url.clone(),
            branch: self.branch.clone(),
            folder: self.folder.clone(),
            token: self.effective_token(account),
        }
    }

    pub fn view(&self, account: Option<&GithubAccount>) -> LinkView {
        LinkView {
            repo_url: self.url.text.clone(),
            branch: self.branch.clone(),
            folder: self.folder.clone(),
            has_token: self.effective_token(account).is_some(),
            account_username: account.filter(|_| self.uses_account).map(|account| account.username.clone()),
            excluded_paths: self.excluded_paths.clone(),
            base_commit: self.base_commit.clone(),
            last_synced_at: self.last_synced_at.clone(),
        }
    }
}

impl Store {
    pub fn github_account(&self) -> Result<Option<GithubAccount>, String> {
        let raw: Option<String> = self.with(|connection| {
            connection
                .query_row("SELECT value FROM settings WHERE key = ?1", params![ACCOUNT_SETTING], |row| row.get(0))
                .optional()
        })?;
        Ok(raw.and_then(|raw| serde_json::from_str(&raw).ok()))
    }

    pub fn save_github_account(&self, account: Option<&GithubAccount>) -> Result<(), String> {
        match account {
            Some(account) => {
                let raw = serde_json::to_string(account).map_err(|error| error.to_string())?;
                self.with(|connection| {
                    connection.execute(
                        "INSERT INTO settings (key, value) VALUES (?1, ?2) ON CONFLICT(key) DO UPDATE SET value = excluded.value",
                        params![ACCOUNT_SETTING, raw],
                    )
                })?;
            }
            None => {
                self.with(|connection| connection.execute("DELETE FROM settings WHERE key = ?1", params![ACCOUNT_SETTING]))?;
            }
        }
        Ok(())
    }

    pub fn git_link(&self, project_path: &str) -> Result<Option<GitLink>, String> {
        let row: Option<LinkRow> = self.with(|connection| {
            connection
                .query_row(
                    &format!("SELECT {LINK_COLUMNS} FROM git_links WHERE project_path = ?1"),
                    params![project_path],
                    |row| {
                        Ok((
                            row.get(0)?,
                            row.get(1)?,
                            row.get(2)?,
                            row.get(3)?,
                            row.get(4)?,
                            row.get(5)?,
                            row.get(6)?,
                            row.get(7)?,
                            row.get(8)?,
                        ))
                    },
                )
                .optional()
        })?;
        row.map(link_from_row).transpose()
    }

    pub fn git_linked_paths(&self) -> Result<Vec<String>, String> {
        self.with(|connection| {
            let mut statement = connection.prepare("SELECT project_path FROM git_links")?;
            let paths = statement.query_map([], |row| row.get(0))?.collect::<rusqlite::Result<Vec<String>>>();
            paths
        })
    }

    pub fn save_git_link(&self, link: &GitLink) -> Result<(), String> {
        self.with(|connection| {
            connection.execute(
                &format!(
                    "INSERT INTO git_links ({LINK_COLUMNS}) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9) \
                     ON CONFLICT(project_path) DO UPDATE SET repo_url = excluded.repo_url, branch = excluded.branch, \
                     folder = excluded.folder, token = excluded.token, uses_account = excluded.uses_account, \
                     excluded_paths = excluded.excluded_paths, base_commit = excluded.base_commit, last_synced_at = excluded.last_synced_at"
                ),
                params![
                    link.project_path,
                    link.url.text,
                    link.branch,
                    link.folder,
                    link.token,
                    link.uses_account,
                    link.excluded_paths.join("\n"),
                    link.base_commit,
                    link.last_synced_at,
                ],
            )
        })?;
        Ok(())
    }

    pub fn delete_git_link(&self, project_path: &str) -> Result<(), String> {
        self.with(|connection| {
            connection.execute("DELETE FROM git_link_files WHERE project_path = ?1", params![project_path])?;
            connection.execute("DELETE FROM git_links WHERE project_path = ?1", params![project_path])
        })?;
        Ok(())
    }

    pub fn git_snapshot(&self, project_path: &str) -> Result<Snapshot, String> {
        self.with(|connection| {
            let mut statement =
                connection.prepare("SELECT path, blob_sha, content_hash FROM git_link_files WHERE project_path = ?1")?;
            let rows = statement
                .query_map(params![project_path], |row| {
                    Ok((row.get::<_, String>(0)?, SnapshotEntry { blob_sha: row.get(1)?, content_hash: row.get(2)? }))
                })?
                .collect::<rusqlite::Result<Snapshot>>();
            rows
        })
    }

    pub fn apply_git_snapshot(
        &self,
        project_path: &str,
        update: &SnapshotUpdate,
        base_commit: &str,
        synced_at: &str,
    ) -> Result<(), String> {
        self.with(|connection| {
            let transaction = connection.unchecked_transaction()?;
            for path in update.removed.iter().chain(update.saved.iter().map(|(path, _)| path)) {
                transaction.execute(
                    "DELETE FROM git_link_files WHERE project_path = ?1 AND path = ?2",
                    params![project_path, path],
                )?;
            }
            for (path, entry) in &update.saved {
                transaction.execute(
                    "INSERT INTO git_link_files (project_path, path, blob_sha, content_hash) VALUES (?1, ?2, ?3, ?4)",
                    params![project_path, path, entry.blob_sha, entry.content_hash],
                )?;
            }
            transaction.execute(
                "UPDATE git_links SET base_commit = ?1, last_synced_at = ?2 WHERE project_path = ?3",
                params![base_commit, synced_at, project_path],
            )?;
            transaction.commit()
        })
    }
}
