use rusqlite::{params, OptionalExtension};
use serde::{Deserialize, Serialize};

use crate::db::Store;

const ZOTERO_SETTING: &str = "zotero";

#[derive(Serialize, Deserialize, Clone)]
pub struct ZoteroAccount {
    pub user_id: String,
    pub username: String,
    pub api_key: String,
}

#[derive(Serialize, Clone)]
pub struct CitationSync {
    pub id: String,
    pub target_path: String,
    pub library_id: String,
    pub library_name: String,
    pub collection_key: Option<String>,
    pub collection_name: Option<String>,
    pub file_name: String,
    pub item_count: i64,
    pub last_synced_at: Option<String>,
}

const SYNC_COLUMNS: &str = "id, target_path, library_id, library_name, collection_key, collection_name, file_name, item_count, last_synced_at";

fn sync_from_row(row: &rusqlite::Row) -> rusqlite::Result<CitationSync> {
    Ok(CitationSync {
        id: row.get(0)?,
        target_path: row.get(1)?,
        library_id: row.get(2)?,
        library_name: row.get(3)?,
        collection_key: row.get(4)?,
        collection_name: row.get(5)?,
        file_name: row.get(6)?,
        item_count: row.get(7)?,
        last_synced_at: row.get(8)?,
    })
}

impl Store {
    pub fn zotero_account(&self) -> Result<Option<ZoteroAccount>, String> {
        let raw: Option<String> = self.with(|connection| {
            connection
                .query_row("SELECT value FROM settings WHERE key = ?1", params![ZOTERO_SETTING], |row| row.get(0))
                .optional()
        })?;
        Ok(raw.and_then(|raw| serde_json::from_str(&raw).ok()))
    }

    pub fn save_zotero_account(&self, account: &ZoteroAccount) -> Result<(), String> {
        let raw = serde_json::to_string(account).map_err(|e| e.to_string())?;
        self.with(|connection| {
            connection.execute(
                "INSERT INTO settings (key, value) VALUES (?1, ?2) ON CONFLICT(key) DO UPDATE SET value = excluded.value",
                params![ZOTERO_SETTING, raw],
            )?;
            Ok(())
        })
    }

    pub fn forget_zotero_account(&self) -> Result<(), String> {
        self.with(|connection| {
            connection.execute("DELETE FROM settings WHERE key = ?1", params![ZOTERO_SETTING])?;
            Ok(())
        })
    }

    pub fn citation_syncs(&self, target_path: &str) -> Result<Vec<CitationSync>, String> {
        self.with(|connection| {
            let mut statement = connection.prepare(&format!(
                "SELECT {SYNC_COLUMNS} FROM citation_syncs WHERE target_path = ?1 ORDER BY file_name ASC"
            ))?;
            let rows = statement.query_map(params![target_path], sync_from_row)?;
            rows.collect()
        })
    }

    pub fn citation_sync(&self, id: &str) -> Result<Option<CitationSync>, String> {
        self.with(|connection| {
            connection
                .query_row(&format!("SELECT {SYNC_COLUMNS} FROM citation_syncs WHERE id = ?1"), params![id], sync_from_row)
                .optional()
        })
    }

    pub fn insert_citation_sync(&self, sync: &CitationSync) -> Result<(), String> {
        self.with(|connection| {
            connection.execute(
                "INSERT INTO citation_syncs (id, target_path, library_id, library_name, collection_key, collection_name, file_name) \
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
                params![sync.id, sync.target_path, sync.library_id, sync.library_name, sync.collection_key, sync.collection_name, sync.file_name],
            )?;
            Ok(())
        })
        .map_err(|_| format!("{} is already synced from Zotero", sync.file_name))
    }

    pub fn record_citation_sync(&self, id: &str, item_count: i64, synced_at: &str) -> Result<(), String> {
        self.with(|connection| {
            connection.execute(
                "UPDATE citation_syncs SET item_count = ?2, last_synced_at = ?3 WHERE id = ?1",
                params![id, item_count, synced_at],
            )?;
            Ok(())
        })
    }

    pub fn delete_citation_sync(&self, id: &str) -> Result<(), String> {
        self.with(|connection| {
            connection.execute("DELETE FROM citation_syncs WHERE id = ?1", params![id])?;
            Ok(())
        })
    }
}
