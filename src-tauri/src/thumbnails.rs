use base64::{engine::general_purpose::STANDARD as BASE64, Engine as _};
use serde::Serialize;
use std::path::Path;
use std::time::UNIX_EPOCH;
use tauri::AppHandle;

use crate::assets::is_image;
use crate::compiler;
use crate::db::Store;
use crate::workspace::{read_target_files, resolve_target, workspace_path};

const MAX_IMAGE_BYTES: u64 = 8 * 1024 * 1024;
const MAX_VIEWER_BYTES: u64 = 64 * 1024 * 1024;

#[derive(Serialize)]
pub struct Thumbnail {
    pub kind: String,
    pub data: String,
}

fn modified_seconds(path: &Path) -> i64 {
    std::fs::metadata(path)
        .and_then(|meta| meta.modified())
        .ok()
        .and_then(|time| time.duration_since(UNIX_EPOCH).ok())
        .map(|elapsed| elapsed.as_secs() as i64)
        .unwrap_or(0)
}

fn newest_change_seconds(dir: &Path) -> i64 {
    let mut newest = 0;
    for entry in walkdir::WalkDir::new(dir).into_iter().filter_map(|e| e.ok()) {
        if !entry.file_type().is_file() {
            continue;
        }
        let seconds = entry
            .metadata()
            .ok()
            .and_then(|meta| meta.modified().ok())
            .and_then(|time| time.duration_since(UNIX_EPOCH).ok())
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or(0);
        if seconds > newest {
            newest = seconds;
        }
    }
    newest
}

fn mime_for(name: &str) -> &'static str {
    let lower = name.to_lowercase();
    if lower.ends_with(".png") {
        "image/png"
    } else if lower.ends_with(".gif") {
        "image/gif"
    } else if lower.ends_with(".svg") {
        "image/svg+xml"
    } else if lower.ends_with(".webp") {
        "image/webp"
    } else {
        "image/jpeg"
    }
}

#[derive(Serialize)]
pub struct ImageData {
    pub name: String,
    pub data: String,
    pub size: u64,
    pub width: Option<u32>,
    pub height: Option<u32>,
}

pub fn read_image(app: &AppHandle, store: &Store, path: &str) -> Result<ImageData, String> {
    let full = workspace_path(app, store, path)?;
    let name = full
        .file_name()
        .map(|n| n.to_string_lossy().to_string())
        .unwrap_or_default();

    if !is_image(&name) {
        return Err("Not an image file".to_string());
    }

    let size = std::fs::metadata(&full).map(|m| m.len()).unwrap_or(0);
    if size > MAX_VIEWER_BYTES {
        return Err("Image is too large to open".to_string());
    }

    let bytes = std::fs::read(&full).map_err(|e| e.to_string())?;
    let (width, height) = image_dimensions(&bytes);

    Ok(ImageData {
        data: format!("data:{};base64,{}", mime_for(&name), BASE64.encode(&bytes)),
        name,
        size,
        width,
        height,
    })
}

fn image_dimensions(bytes: &[u8]) -> (Option<u32>, Option<u32>) {
    if bytes.len() > 24 && bytes.starts_with(&[0x89, b'P', b'N', b'G']) {
        let width = u32::from_be_bytes([bytes[16], bytes[17], bytes[18], bytes[19]]);
        let height = u32::from_be_bytes([bytes[20], bytes[21], bytes[22], bytes[23]]);
        return (Some(width), Some(height));
    }

    if bytes.len() > 10 && bytes.starts_with(&[0xFF, 0xD8]) {
        let mut index = 2usize;
        while index + 9 < bytes.len() {
            if bytes[index] != 0xFF {
                index += 1;
                continue;
            }
            let marker = bytes[index + 1];
            if (0xC0..=0xCF).contains(&marker) && marker != 0xC4 && marker != 0xC8 && marker != 0xCC
            {
                let height = u16::from_be_bytes([bytes[index + 5], bytes[index + 6]]) as u32;
                let width = u16::from_be_bytes([bytes[index + 7], bytes[index + 8]]) as u32;
                return (Some(width), Some(height));
            }
            let length = u16::from_be_bytes([bytes[index + 2], bytes[index + 3]]) as usize;
            index += 2 + length;
        }
    }

    (None, None)
}

pub fn thumbnail(app: &AppHandle, store: &Store, path: &str) -> Result<Thumbnail, String> {
    let full = workspace_path(app, store, path)?;
    let name = full
        .file_name()
        .map(|n| n.to_string_lossy().to_string())
        .unwrap_or_default();

    let is_project = full.is_dir();
    let image = is_image(&name);
    if !image && !is_project && !name.to_lowercase().ends_with(".typ") {
        return Err("No preview available".to_string());
    }

    let modified = if is_project {
        newest_change_seconds(&full)
    } else {
        modified_seconds(&full)
    };

    if let Some((kind, data)) = store.thumbnail(path, modified)? {
        return Ok(Thumbnail { kind, data });
    }

    let thumbnail = if image {
        let size = std::fs::metadata(&full).map(|m| m.len()).unwrap_or(0);
        if size > MAX_IMAGE_BYTES {
            return Err("Image is too large to preview".to_string());
        }
        let bytes = std::fs::read(&full).map_err(|e| e.to_string())?;
        Thumbnail {
            kind: "image".to_string(),
            data: format!("data:{};base64,{}", mime_for(&name), BASE64.encode(&bytes)),
        }
    } else {
        let target = resolve_target(app, store, path)?;
        let files = read_target_files(app, store, &target)?;

        let result = compiler::compile_to_svg(target.entrypoint, files)
            .map_err(|_| "Document does not compile".to_string())?;

        let svg = result
            .pages
            .into_iter()
            .next()
            .ok_or("Document has no pages")?;

        Thumbnail {
            kind: "svg".to_string(),
            data: svg,
        }
    };

    store.save_thumbnail(path, &thumbnail.kind, &thumbnail.data, modified)?;
    Ok(thumbnail)
}
