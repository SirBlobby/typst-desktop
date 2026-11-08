use std::path::{Component, Path};

use typst::syntax::package::PackageSpec;
use typst_kit::downloader::SystemDownloader;
use typst_kit::packages::SystemPackages;

use super::index::Template;

const MAX_FILES: usize = 500;
const MAX_TOTAL_BYTES: usize = 100 * 1024 * 1024;

pub struct TemplateFile {
    pub path: String,
    pub data: Vec<u8>,
}

pub fn is_safe_relative(path: &str) -> bool {
    Path::new(path)
        .components()
        .all(|component| matches!(component, Component::Normal(_) | Component::CurDir))
}

pub fn clean_path(path: &str) -> &str {
    path.trim_start_matches("./")
}

fn too_large() -> String {
    format!("This template has more than {MAX_FILES} files or 100 MB of files")
}

fn collect(directory: &Path, prefix: &str, files: &mut Vec<TemplateFile>, total_bytes: &mut usize) -> Result<(), String> {
    let mut entries: Vec<_> = std::fs::read_dir(directory)
        .map_err(|error| error.to_string())?
        .filter_map(Result::ok)
        .collect();
    entries.sort_by_key(|entry| entry.file_name());

    for entry in entries {
        let file_type = entry.file_type().map_err(|error| error.to_string())?;
        let name = entry.file_name().to_string_lossy().to_string();
        let path = if prefix.is_empty() { name } else { format!("{prefix}/{name}") };

        if file_type.is_dir() {
            collect(&entry.path(), &path, files, total_bytes)?;
        } else if file_type.is_file() {
            let data = std::fs::read(entry.path()).map_err(|error| error.to_string())?;
            *total_bytes += data.len();
            if files.len() >= MAX_FILES || *total_bytes > MAX_TOTAL_BYTES {
                return Err(too_large());
            }
            files.push(TemplateFile { path, data });
        }
    }
    Ok(())
}

pub fn template_files(template: &Template) -> Result<Vec<TemplateFile>, String> {
    if !is_safe_relative(&template.template_path) || !is_safe_relative(&template.entrypoint) {
        return Err("This template has an invalid layout".to_string());
    }
    let spec: PackageSpec = format!("@preview/{}:{}", template.name, template.version)
        .parse()
        .map_err(|_| "Invalid template name".to_string())?;
    let packages = SystemPackages::new(SystemDownloader::new("TypstDesktop (typst-kit)"));
    let root = packages
        .obtain(&spec)
        .map_err(|error| format!("Could not download the template: {error}"))?;

    let mut files = Vec::new();
    let mut total_bytes = 0;
    collect(&root.path().join(&template.template_path), "", &mut files, &mut total_bytes)?;
    if files.is_empty() {
        return Err("This template has no files".to_string());
    }
    Ok(files)
}
