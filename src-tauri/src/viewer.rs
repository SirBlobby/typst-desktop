use tauri::ipc::Response;
use tauri::{AppHandle, State};

use crate::db::Store;
use crate::workspace::workspace_path;

const MAX_PDF_BYTES: u64 = 256 * 1024 * 1024;

fn is_pdf(name: &str) -> bool {
    name.to_lowercase().ends_with(".pdf")
}

#[tauri::command(async)]
pub fn read_pdf(app: AppHandle, store: State<'_, Store>, path: String) -> Result<Response, String> {
    if !is_pdf(&path) {
        return Err("Not a PDF file".to_string());
    }

    let full = workspace_path(&app, &store, &path)?;
    let size = std::fs::metadata(&full).map_err(|e| e.to_string())?.len();
    if size > MAX_PDF_BYTES {
        return Err("PDF is too large to open".to_string());
    }

    let bytes = std::fs::read(&full).map_err(|e| e.to_string())?;
    Ok(Response::new(bytes))
}
