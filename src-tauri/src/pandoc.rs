use std::path::{Path, PathBuf};
use std::process::Command;

pub const EXPORT_FORMATS: [&str; 4] = ["docx", "latex", "markdown", "html"];

fn import_format(source: &Path) -> Result<&'static str, String> {
    let extension = source
        .extension()
        .map(|ext| ext.to_string_lossy().to_lowercase())
        .unwrap_or_default();

    match extension.as_str() {
        "docx" => Ok("docx"),
        "tex" => Ok("latex"),
        "md" | "markdown" => Ok("markdown"),
        "html" | "htm" => Ok("html"),
        other => Err(format!("Pandoc cannot import '.{}' files", other)),
    }
}

fn pandoc_program() -> PathBuf {
    let file_name = if cfg!(windows) { "pandoc.exe" } else { "pandoc" };
    std::env::current_exe()
        .ok()
        .and_then(|executable| executable.parent().map(|dir| dir.join(file_name)))
        .filter(|bundled| bundled.is_file())
        .unwrap_or_else(|| PathBuf::from("pandoc"))
}

fn run_pandoc(working_dir: &Path, arguments: &[&str]) -> Result<(), String> {
    let output = Command::new(pandoc_program())
        .current_dir(working_dir)
        .args(arguments)
        .output()
        .map_err(|error| {
            if error.kind() == std::io::ErrorKind::NotFound {
                "Pandoc could not be found. Reinstall Typst Desktop to restore it.".to_string()
            } else {
                format!("Could not start pandoc: {}", error)
            }
        })?;

    if output.status.success() {
        Ok(())
    } else {
        Err(format!(
            "Pandoc error: {}",
            String::from_utf8_lossy(&output.stderr).trim()
        ))
    }
}

pub fn version() -> Option<String> {
    let output = Command::new(pandoc_program()).arg("--version").output().ok()?;
    if !output.status.success() {
        return None;
    }
    String::from_utf8_lossy(&output.stdout)
        .lines()
        .next()
        .map(|line| line.trim().to_string())
}

pub fn export(root: &Path, entrypoint: &str, format: &str, destination: &str) -> Result<(), String> {
    if !EXPORT_FORMATS.contains(&format) {
        return Err(format!("Pandoc cannot export to '{}'", format));
    }

    let mut arguments = vec!["-f", "typst", "-t", format, "-o", destination];
    if format == "html" || format == "latex" {
        arguments.push("--standalone");
    }
    arguments.push(entrypoint);

    run_pandoc(root, &arguments)
}

fn unique_typst_path(folder: &Path, stem: &str) -> PathBuf {
    let candidate = folder.join(format!("{}.typ", stem));
    if !candidate.exists() {
        return candidate;
    }
    (1..)
        .map(|index| folder.join(format!("{} ({}).typ", stem, index)))
        .find(|path| !path.exists())
        .unwrap_or(candidate)
}

pub fn import(source: &str, folder: &Path) -> Result<String, String> {
    let source_path = Path::new(source);
    let from = import_format(source_path)?;
    let stem = source_path
        .file_stem()
        .map(|stem| stem.to_string_lossy().to_string())
        .unwrap_or_else(|| "imported".to_string());

    std::fs::create_dir_all(folder).map_err(|e| e.to_string())?;
    let destination = unique_typst_path(folder, &stem);
    let destination_name = destination
        .file_name()
        .map(|name| name.to_string_lossy().to_string())
        .ok_or("Could not name the converted file")?;
    let media_folder = format!("{}-media", stem);

    let mut arguments = vec!["-f", from, "-t", "typst", "-o", destination_name.as_str(), source];
    let extract_media = format!("--extract-media={}", media_folder);
    if from == "docx" {
        arguments.push(extract_media.as_str());
    }

    run_pandoc(folder, &arguments)?;
    Ok(destination_name)
}
