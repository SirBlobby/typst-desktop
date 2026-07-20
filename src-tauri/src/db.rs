use rusqlite::{params, Connection, OptionalExtension};
use serde::Serialize;
use std::collections::HashMap;
use std::sync::Mutex;
use tauri::{AppHandle, Manager};

use crate::workspace::{ProjectMeta, Settings};

pub struct Store {
    connection: Mutex<Connection>,
}

#[derive(Serialize, Clone)]
pub struct DocumentLink {
    pub document_id: String,
    pub base_hash: String,
    pub role: String,
    pub base_content: String,
    pub synced_at: Option<String>,
}

const SCHEMA: [&str; 5] = [
    "CREATE TABLE IF NOT EXISTS settings (
        key TEXT PRIMARY KEY,
        value TEXT NOT NULL
    )",
    "CREATE TABLE IF NOT EXISTS projects (
        path TEXT PRIMARY KEY,
        entrypoint TEXT NOT NULL DEFAULT 'main.typ',
        cloud_project_id TEXT,
        last_synced_at TEXT
    )",
    "CREATE TABLE IF NOT EXISTS base_files (
        project_path TEXT NOT NULL,
        file_path TEXT NOT NULL,
        hash TEXT NOT NULL,
        content BLOB,
        PRIMARY KEY (project_path, file_path)
    )",
    "CREATE TABLE IF NOT EXISTS document_links (
        path TEXT PRIMARY KEY,
        document_id TEXT NOT NULL,
        base_hash TEXT NOT NULL,
        role TEXT NOT NULL,
        base_content TEXT,
        synced_at TEXT
    )",
    "CREATE TABLE IF NOT EXISTS thumbnails (
        path TEXT PRIMARY KEY,
        kind TEXT NOT NULL,
        data TEXT NOT NULL,
        source_modified INTEGER NOT NULL
    )",
];

const MIGRATIONS: [&str; 2] = [
    "ALTER TABLE document_links ADD COLUMN synced_at TEXT",
    "ALTER TABLE projects RENAME COLUMN space_id TO cloud_project_id",
];

impl Store {
    pub fn open(app: &AppHandle) -> Result<Self, String> {
        let dir = app
            .path()
            .app_data_dir()
            .map_err(|e| format!("Cannot resolve data directory: {}", e))?;
        std::fs::create_dir_all(&dir).map_err(|e| e.to_string())?;

        let connection =
            Connection::open(dir.join("typst-desktop.db")).map_err(|e| e.to_string())?;

        connection
            .pragma_update(None, "journal_mode", "WAL")
            .map_err(|e| e.to_string())?;
        connection
            .pragma_update(None, "foreign_keys", "ON")
            .map_err(|e| e.to_string())?;

        for statement in SCHEMA {
            connection.execute(statement, []).map_err(|e| e.to_string())?;
        }

        for statement in MIGRATIONS {
            let _ = connection.execute(statement, []);
        }

        Ok(Store {
            connection: Mutex::new(connection),
        })
    }

    fn with<T>(&self, run: impl FnOnce(&Connection) -> rusqlite::Result<T>) -> Result<T, String> {
        let connection = self
            .connection
            .lock()
            .map_err(|_| "Local database lock poisoned".to_string())?;
        run(&connection).map_err(|e| e.to_string())
    }

    pub fn settings(&self) -> Result<Option<Settings>, String> {
        let raw: Option<String> = self.with(|connection| {
            connection
                .query_row(
                    "SELECT value FROM settings WHERE key = 'settings'",
                    [],
                    |row| row.get(0),
                )
                .optional()
        })?;

        match raw {
            Some(raw) => serde_json::from_str(&raw)
                .map(Some)
                .map_err(|e| format!("Stored settings are invalid: {}", e)),
            None => Ok(None),
        }
    }

    pub fn save_settings(&self, settings: &Settings) -> Result<(), String> {
        let raw = serde_json::to_string(settings).map_err(|e| e.to_string())?;
        self.with(|connection| {
            connection.execute(
                "INSERT INTO settings (key, value) VALUES ('settings', ?1)
                 ON CONFLICT(key) DO UPDATE SET value = excluded.value",
                params![raw],
            )
        })?;
        Ok(())
    }

    pub fn meta(&self, project: &str) -> Result<ProjectMeta, String> {
        let row: Option<(String, Option<String>, Option<String>)> = self.with(|connection| {
            connection
                .query_row(
                    "SELECT entrypoint, cloud_project_id, last_synced_at FROM projects WHERE path = ?1",
                    params![project],
                    |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)),
                )
                .optional()
        })?;

        let Some((entrypoint, cloud_project_id, last_synced_at)) = row else {
            return Ok(ProjectMeta::default());
        };

        let base_hashes = self.with(|connection| {
            let mut statement = connection
                .prepare("SELECT file_path, hash FROM base_files WHERE project_path = ?1")?;
            let rows = statement.query_map(params![project], |row| {
                Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?))
            })?;

            let mut map = HashMap::new();
            for row in rows {
                let (path, hash) = row?;
                map.insert(path, hash);
            }
            Ok(map)
        })?;

        Ok(ProjectMeta {
            entrypoint,
            cloud_project_id,
            last_synced_at,
            base_hashes,
        })
    }

    pub fn has_project(&self, project: &str) -> Result<bool, String> {
        let found: Option<i64> = self.with(|connection| {
            connection
                .query_row(
                    "SELECT 1 FROM projects WHERE path = ?1",
                    params![project],
                    |row| row.get(0),
                )
                .optional()
        })?;
        Ok(found.is_some())
    }

    pub fn save_meta(&self, project: &str, meta: &ProjectMeta) -> Result<(), String> {
        self.with(|connection| {
            connection.execute(
                "INSERT INTO projects (path, entrypoint, cloud_project_id, last_synced_at)
                 VALUES (?1, ?2, ?3, ?4)
                 ON CONFLICT(path) DO UPDATE SET
                   entrypoint = excluded.entrypoint,
                   cloud_project_id = excluded.cloud_project_id,
                   last_synced_at = excluded.last_synced_at",
                params![
                    project,
                    meta.entrypoint,
                    meta.cloud_project_id,
                    meta.last_synced_at
                ],
            )?;

            let mut keep: Vec<String> = Vec::new();
            for (file, hash) in &meta.base_hashes {
                connection.execute(
                    "INSERT INTO base_files (project_path, file_path, hash)
                     VALUES (?1, ?2, ?3)
                     ON CONFLICT(project_path, file_path) DO UPDATE SET hash = excluded.hash",
                    params![project, file, hash],
                )?;
                keep.push(file.clone());
            }

            let mut statement = connection
                .prepare("SELECT file_path FROM base_files WHERE project_path = ?1")?;
            let existing: Vec<String> = statement
                .query_map(params![project], |row| row.get::<_, String>(0))?
                .collect::<rusqlite::Result<Vec<String>>>()?;

            for file in existing {
                if !keep.contains(&file) {
                    connection.execute(
                        "DELETE FROM base_files WHERE project_path = ?1 AND file_path = ?2",
                        params![project, file],
                    )?;
                }
            }

            Ok(())
        })
    }

    pub fn forget_project(&self, project: &str) -> Result<(), String> {
        self.with(|connection| {
            connection.execute(
                "DELETE FROM base_files WHERE project_path = ?1",
                params![project],
            )?;
            connection.execute("DELETE FROM projects WHERE path = ?1", params![project])?;
            Ok(())
        })
    }

    pub fn rename_project(&self, from: &str, to: &str) -> Result<(), String> {
        self.with(|connection| {
            connection.execute(
                "UPDATE projects SET path = ?2 WHERE path = ?1",
                params![from, to],
            )?;
            connection.execute(
                "UPDATE base_files SET project_path = ?2 WHERE project_path = ?1",
                params![from, to],
            )?;
            Ok(())
        })
    }

    pub fn base_snapshot(&self, project: &str, file: &str) -> Result<Option<Vec<u8>>, String> {
        self.with(|connection| {
            connection
                .query_row(
                    "SELECT content FROM base_files WHERE project_path = ?1 AND file_path = ?2",
                    params![project, file],
                    |row| row.get::<_, Option<Vec<u8>>>(0),
                )
                .optional()
                .map(|value| value.flatten())
        })
    }

    pub fn save_base_snapshot(
        &self,
        project: &str,
        file: &str,
        hash: &str,
        content: &[u8],
    ) -> Result<(), String> {
        self.with(|connection| {
            connection.execute(
                "INSERT INTO base_files (project_path, file_path, hash, content)
                 VALUES (?1, ?2, ?3, ?4)
                 ON CONFLICT(project_path, file_path) DO UPDATE SET
                   hash = excluded.hash,
                   content = excluded.content",
                params![project, file, hash, content],
            )?;
            Ok(())
        })
    }

    pub fn save_document_link(
        &self,
        path: &str,
        document_id: &str,
        base_hash: &str,
        role: &str,
        base_content: &str,
    ) -> Result<(), String> {
        self.with(|connection| {
            connection.execute(
                "INSERT INTO document_links
                   (path, document_id, base_hash, role, base_content, synced_at)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6)
                 ON CONFLICT(path) DO UPDATE SET
                   document_id = excluded.document_id,
                   base_hash = excluded.base_hash,
                   role = excluded.role,
                   base_content = excluded.base_content,
                   synced_at = excluded.synced_at",
                params![
                    path,
                    document_id,
                    base_hash,
                    role,
                    base_content,
                    chrono::Utc::now().to_rfc3339()
                ],
            )?;
            Ok(())
        })
    }

    pub fn document_link(&self, path: &str) -> Result<Option<DocumentLink>, String> {
        self.with(|connection| {
            connection
                .query_row(
                    "SELECT document_id, base_hash, role, base_content, synced_at
                     FROM document_links WHERE path = ?1",
                    params![path],
                    |row| {
                        Ok(DocumentLink {
                            document_id: row.get(0)?,
                            base_hash: row.get(1)?,
                            role: row.get(2)?,
                            base_content: row.get::<_, Option<String>>(3)?.unwrap_or_default(),
                            synced_at: row.get(4)?,
                        })
                    },
                )
                .optional()
        })
    }

    pub fn all_cloud_project_links(&self) -> Result<Vec<(String, String, Option<String>)>, String> {
        self.with(|connection| {
            let mut statement = connection.prepare(
                "SELECT path, cloud_project_id, last_synced_at FROM projects
                 WHERE cloud_project_id IS NOT NULL",
            )?;
            let rows = statement.query_map([], |row| {
                Ok((
                    row.get::<_, String>(0)?,
                    row.get::<_, String>(1)?,
                    row.get::<_, Option<String>>(2)?,
                ))
            })?;
            rows.collect::<rusqlite::Result<Vec<_>>>()
        })
    }

    pub fn all_document_links(&self) -> Result<Vec<(String, String, Option<String>)>, String> {
        self.with(|connection| {
            let mut statement = connection
                .prepare("SELECT path, document_id, synced_at FROM document_links")?;
            let rows = statement.query_map([], |row| {
                Ok((
                    row.get::<_, String>(0)?,
                    row.get::<_, String>(1)?,
                    row.get::<_, Option<String>>(2)?,
                ))
            })?;
            rows.collect::<rusqlite::Result<Vec<_>>>()
        })
    }

    pub fn forget_document_link(&self, path: &str) -> Result<(), String> {
        self.with(|connection| {
            connection.execute("DELETE FROM document_links WHERE path = ?1", params![path])?;
            Ok(())
        })
    }

    pub fn thumbnail(&self, path: &str, modified: i64) -> Result<Option<(String, String)>, String> {
        self.with(|connection| {
            connection
                .query_row(
                    "SELECT kind, data FROM thumbnails
                     WHERE path = ?1 AND source_modified >= ?2",
                    params![path, modified],
                    |row| Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?)),
                )
                .optional()
        })
    }

    pub fn save_thumbnail(
        &self,
        path: &str,
        kind: &str,
        data: &str,
        modified: i64,
    ) -> Result<(), String> {
        self.with(|connection| {
            connection.execute(
                "INSERT INTO thumbnails (path, kind, data, source_modified)
                 VALUES (?1, ?2, ?3, ?4)
                 ON CONFLICT(path) DO UPDATE SET
                   kind = excluded.kind,
                   data = excluded.data,
                   source_modified = excluded.source_modified",
                params![path, kind, data, modified],
            )?;
            Ok(())
        })
    }

    pub fn clear_thumbnails(&self) -> Result<(), String> {
        self.with(|connection| {
            connection.execute("DELETE FROM thumbnails", [])?;
            Ok(())
        })
    }
}
