use serde::{Deserialize, Serialize};
use std::sync::RwLock;
use tauri::{AppHandle, State};

use crate::db::Store;

const SPACE_HEADER: &str = "x-workspace";

static ACTIVE_SPACE: RwLock<Option<String>> = RwLock::new(None);

#[derive(Serialize, Deserialize)]
pub struct CloudSpace {
    pub id: String,
    pub name: String,
    pub role: String,
    pub member_count: i64,
}

fn active_space() -> Option<String> {
    ACTIVE_SPACE.read().ok().and_then(|space| space.clone())
}

pub fn set_active_space(team_id: Option<String>) {
    if let Ok(mut space) = ACTIVE_SPACE.write() {
        *space = team_id;
    }
}

pub fn add_space_header(
    request: ureq::Request,
    next: ureq::MiddlewareNext,
) -> Result<ureq::Response, ureq::Error> {
    match active_space() {
        Some(team_id) => next.handle(request.set(SPACE_HEADER, &team_id)),
        None => next.handle(request),
    }
}

#[tauri::command(async)]
pub fn cloud_list_spaces(app: AppHandle, store: State<'_, Store>) -> Result<Vec<CloudSpace>, String> {
    let (server_url, token) = crate::cloud_credentials(&app, &store)?;
    crate::sync::list_spaces(&server_url, &token)
}

#[tauri::command]
pub fn cloud_set_space(team_id: Option<String>) {
    set_active_space(team_id);
}
