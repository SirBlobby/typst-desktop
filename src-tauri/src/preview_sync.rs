use serde::Serialize;
use std::collections::HashMap;
use tauri::{AppHandle, State};
use typst::introspection::PagedPosition;
use typst::layout::{Abs, Point};
use typst::World;
use typst_ide::Jump;
use typst_layout::PagedDocument;

use crate::latex;
use crate::world::{project_file_id, project_path, ProjectWorld};
use crate::db::Store;

#[derive(Serialize)]
pub struct PagePoint {
    pub page: usize,
    pub x: f64,
    pub y: f64,
}

#[derive(Serialize)]
#[serde(tag = "kind", rename_all = "lowercase")]
pub enum ClickTarget {
    Source { path: String, offset: usize },
    Page(PagePoint),
}

fn compiled(entrypoint: String, files: HashMap<String, Vec<u8>>) -> Option<(ProjectWorld, PagedDocument)> {
    if latex::is_latex_file(&entrypoint) {
        return None;
    }
    let world = ProjectWorld::new(entrypoint, files, false);
    let document = typst::compile::<PagedDocument>(&world).output.ok()?;
    Some((world, document))
}

fn page_point(document: &PagedDocument, position: &PagedPosition) -> Option<PagePoint> {
    let size = document.pages().get(position.page.get() - 1)?.frame.size();
    if size.x <= Abs::zero() || size.y <= Abs::zero() {
        return None;
    }
    Some(PagePoint {
        page: position.page.get(),
        x: position.point.x / size.x,
        y: position.point.y / size.y,
    })
}

fn cursor_positions(
    entrypoint: String,
    files: HashMap<String, Vec<u8>>,
    file: &str,
    utf16_offset: usize,
) -> Vec<PagePoint> {
    let Some((world, document)) = compiled(entrypoint, files) else {
        return Vec::new();
    };
    let Some(source) = project_file_id(file).and_then(|id| world.source(id).ok()) else {
        return Vec::new();
    };
    let Some(cursor) = source.lines().utf16_to_byte(utf16_offset) else {
        return Vec::new();
    };
    typst_ide::jump_from_cursor(&document, &source, cursor)
        .iter()
        .filter_map(|position| page_point(&document, position))
        .collect()
}

fn click_target(
    entrypoint: String,
    files: HashMap<String, Vec<u8>>,
    page: usize,
    x: f64,
    y: f64,
) -> Option<ClickTarget> {
    let (world, document) = compiled(entrypoint, files)?;
    let size = document.pages().get(page.checked_sub(1)?)?.frame.size();
    let position = PagedPosition {
        page: std::num::NonZeroUsize::new(page)?,
        point: Point::new(size.x * x.clamp(0.0, 1.0), size.y * y.clamp(0.0, 1.0)),
    };
    match typst_ide::jump_from_click(&world, &document, &position)? {
        Jump::File(id, byte) => {
            let path = project_path(id)?;
            let offset = world.source(id).ok()?.lines().byte_to_utf16(byte)?;
            Some(ClickTarget::Source { path, offset })
        }
        Jump::Position(target) => page_point(&document, &target).map(ClickTarget::Page),
        Jump::Url(_) => None,
    }
}

#[tauri::command(async)]
pub fn sync_cursor(
    app: AppHandle,
    store: State<'_, Store>,
    path: String,
    entrypoint: Option<String>,
    overrides: Option<HashMap<String, String>>,
    file: String,
    offset: usize,
) -> Result<Vec<PagePoint>, String> {
    let (entrypoint, files) = crate::workspace::target_compile_input(&app, &store, &path, entrypoint, overrides)?;
    Ok(cursor_positions(entrypoint, files, &file, offset))
}

#[tauri::command(async)]
#[allow(clippy::too_many_arguments)]
pub fn sync_click(
    app: AppHandle,
    store: State<'_, Store>,
    path: String,
    entrypoint: Option<String>,
    overrides: Option<HashMap<String, String>>,
    page: usize,
    x: f64,
    y: f64,
) -> Result<Option<ClickTarget>, String> {
    let (entrypoint, files) = crate::workspace::target_compile_input(&app, &store, &path, entrypoint, overrides)?;
    Ok(click_target(entrypoint, files, page, x, y))
}
