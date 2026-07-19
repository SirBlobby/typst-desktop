use serde::Serialize;
use std::collections::HashMap;
use typst::diag::Warned;
use typst::layout::{Frame, FrameItem};
use typst::utils::Scalar;
use typst::WorldExt;
use typst_html::{HtmlDocument, HtmlOptions};
use typst_layout::PagedDocument;
use typst_pdf::{pdf, PdfOptions};
use typst_render::{render, RenderOptions};
use typst_svg::SvgOptions;

use crate::world::ProjectWorld;

#[derive(Serialize, Clone)]
pub struct DocumentStats {
    pub pages: usize,
    pub words: usize,
    pub characters: usize,
}

#[derive(Serialize, Clone)]
pub struct Diagnostic {
    pub message: String,
    pub severity: String,
    pub line: Option<usize>,
    pub column: Option<usize>,
}

#[derive(Serialize)]
pub struct CompileResult {
    pub pages: Vec<String>,
    pub stats: DocumentStats,
    pub diagnostics: Vec<Diagnostic>,
}

fn extract_frame_text(frame: &Frame, text: &mut String) {
    for (_, item) in frame.items() {
        match item {
            FrameItem::Text(text_item) => {
                text.push_str(&text_item.text);
                text.push(' ');
            }
            FrameItem::Group(group) => extract_frame_text(&group.frame, text),
            _ => {}
        }
    }
}

fn extract_stats(document: &PagedDocument) -> DocumentStats {
    let mut text = String::new();
    for page in document.pages() {
        extract_frame_text(&page.frame, &mut text);
    }

    DocumentStats {
        pages: document.pages().len(),
        words: text.split_whitespace().count(),
        characters: text.chars().filter(|c| !c.is_whitespace()).count(),
    }
}

fn line_and_column(source: &str, offset: usize) -> (usize, usize) {
    let mut line = 1;
    let mut column = 1;
    for (index, character) in source.char_indices() {
        if index >= offset {
            break;
        }
        if character == '\n' {
            line += 1;
            column = 1;
        } else {
            column += 1;
        }
    }
    (line, column)
}

fn collect_diagnostics(
    world: &ProjectWorld,
    entrypoint_source: &str,
    errors: impl IntoIterator<Item = typst::diag::SourceDiagnostic>,
) -> Vec<Diagnostic> {
    errors
        .into_iter()
        .map(|diagnostic| {
            let (line, column) = match world.range(diagnostic.span) {
                Some(range) => {
                    let (line, column) = line_and_column(entrypoint_source, range.start);
                    (Some(line), Some(column))
                }
                None => (None, None),
            };
            Diagnostic {
                message: diagnostic.message.to_string(),
                severity: format!("{:?}", diagnostic.severity).to_lowercase(),
                line,
                column,
            }
        })
        .collect()
}

fn entrypoint_text(files: &HashMap<String, Vec<u8>>, entrypoint: &str) -> String {
    files
        .get(entrypoint)
        .map(|bytes| String::from_utf8_lossy(bytes).to_string())
        .unwrap_or_default()
}

pub fn compile_to_svg(
    entrypoint: String,
    files: HashMap<String, Vec<u8>>,
) -> Result<CompileResult, Vec<Diagnostic>> {
    let source_text = entrypoint_text(&files, &entrypoint);
    let world = ProjectWorld::new(entrypoint, files, false);

    match typst::compile::<PagedDocument>(&world) {
        Warned {
            output: Ok(document),
            warnings,
        } => {
            let options = SvgOptions::default();
            let pages = document
                .pages()
                .iter()
                .map(|page| typst_svg::svg(page, &options))
                .collect();

            Ok(CompileResult {
                pages,
                stats: extract_stats(&document),
                diagnostics: collect_diagnostics(&world, &source_text, warnings),
            })
        }
        Warned {
            output: Err(errors),
            warnings: _,
        } => Err(collect_diagnostics(&world, &source_text, errors)),
    }
}

pub fn export_pdf(
    entrypoint: String,
    files: HashMap<String, Vec<u8>>,
) -> Result<Vec<u8>, Vec<Diagnostic>> {
    let source_text = entrypoint_text(&files, &entrypoint);
    let world = ProjectWorld::new(entrypoint, files, false);

    match typst::compile::<PagedDocument>(&world) {
        Warned {
            output: Ok(document),
            warnings: _,
        } => pdf(&document, &PdfOptions::default()).map_err(|errors| {
            collect_diagnostics(&world, &source_text, errors)
        }),
        Warned {
            output: Err(errors),
            warnings: _,
        } => Err(collect_diagnostics(&world, &source_text, errors)),
    }
}

pub fn export_png(
    entrypoint: String,
    files: HashMap<String, Vec<u8>>,
) -> Result<Vec<u8>, Vec<Diagnostic>> {
    let source_text = entrypoint_text(&files, &entrypoint);
    let world = ProjectWorld::new(entrypoint, files, false);

    match typst::compile::<PagedDocument>(&world) {
        Warned {
            output: Ok(document),
            warnings: _,
        } => {
            let options = RenderOptions {
                pixel_per_pt: Scalar::new(2.0),
                ..RenderOptions::default()
            };
            match document.pages().first() {
                Some(page) => Ok(render(page, &options).encode_png().unwrap_or_default()),
                None => Ok(Vec::new()),
            }
        }
        Warned {
            output: Err(errors),
            warnings: _,
        } => Err(collect_diagnostics(&world, &source_text, errors)),
    }
}

pub fn export_html(
    entrypoint: String,
    files: HashMap<String, Vec<u8>>,
) -> Result<Vec<u8>, Vec<Diagnostic>> {
    let source_text = entrypoint_text(&files, &entrypoint);
    let world = ProjectWorld::new(entrypoint, files, true);

    let document = match typst::compile::<HtmlDocument>(&world) {
        Warned {
            output: Ok(document),
            warnings: _,
        } => document,
        Warned {
            output: Err(errors),
            warnings: _,
        } => return Err(collect_diagnostics(&world, &source_text, errors)),
    };

    typst_html::html(&document, &HtmlOptions::default())
        .map(|html| html.into_bytes())
        .map_err(|errors| collect_diagnostics(&world, &source_text, errors))
}
