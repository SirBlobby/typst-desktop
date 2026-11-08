pub mod listing;

use std::path::{Path, PathBuf};

use serde::Serialize;
use tauri::{AppHandle, State};
use typst::syntax::package::{PackageManifest, PackageSpec};
use typst_kit::packages::FsPackages;

use crate::db::Store;
use crate::workspace;

const NAMESPACE: &str = "local";

#[derive(Serialize)]
pub struct LocalPackageInfo {
    pub name: String,
    pub version: String,
    pub import_line: String,
    pub installed_versions: Vec<String>,
}

fn packages_root() -> Result<PathBuf, String> {
    FsPackages::system_data()
        .map(|packages| packages.path().join(NAMESPACE))
        .ok_or_else(|| "Could not find the Typst package folder on this system".to_string())
}

fn read_manifest(project_dir: &Path) -> Result<PackageManifest, String> {
    let manifest_path = project_dir.join("typst.toml");
    let text = std::fs::read_to_string(&manifest_path)
        .map_err(|_| "This project has no typst.toml, so it cannot be installed as a package".to_string())?;
    toml::from_str(&text).map_err(|error| format!("typst.toml is invalid: {}", error.message()))
}

fn package_spec(manifest: &PackageManifest) -> Result<PackageSpec, String> {
    let spec_text = format!("@{NAMESPACE}/{}:{}", manifest.package.name, manifest.package.version);
    let spec: PackageSpec = spec_text
        .parse()
        .map_err(|error| format!("typst.toml has an invalid package name or version: {error}"))?;
    manifest.validate(&spec).map_err(|error| error.to_string())?;
    Ok(spec)
}

fn import_line(spec: &PackageSpec) -> String {
    format!("#import \"@{}/{}:{}\": *", spec.namespace, spec.name, spec.version)
}

fn installed_versions(name: &str) -> Vec<String> {
    let Ok(root) = packages_root() else {
        return Vec::new();
    };
    let Ok(entries) = std::fs::read_dir(root.join(name)) else {
        return Vec::new();
    };
    let mut versions: Vec<String> = entries
        .filter_map(Result::ok)
        .filter(|entry| entry.path().is_dir())
        .map(|entry| entry.file_name().to_string_lossy().to_string())
        .filter(|version| !version.starts_with('.'))
        .collect();
    versions.sort_by_key(|version| {
        version
            .split('.')
            .map(|part| part.parse::<u64>().unwrap_or(0))
            .collect::<Vec<_>>()
    });
    versions.reverse();
    versions
}

fn project_dir(app: &AppHandle, store: &Store, path: &str) -> Result<PathBuf, String> {
    let directory = workspace::workspace_path(app, store, path)?;
    if !directory.is_dir() {
        return Err("Only projects can be installed as packages".to_string());
    }
    Ok(directory)
}

fn copy_project(source: &Path, destination: &Path) -> Result<(), String> {
    for relative in workspace::collect_files(source)? {
        let from = workspace::project_file_path(source, &relative)?;
        let to = workspace::project_file_path(destination, &relative)?;
        if let Some(directory) = to.parent() {
            std::fs::create_dir_all(directory).map_err(|error| error.to_string())?;
        }
        std::fs::copy(&from, &to).map_err(|error| error.to_string())?;
    }
    Ok(())
}

fn info_for(spec: &PackageSpec) -> LocalPackageInfo {
    LocalPackageInfo {
        name: spec.name.to_string(),
        version: spec.version.to_string(),
        import_line: import_line(spec),
        installed_versions: installed_versions(&spec.name),
    }
}

#[tauri::command]
pub fn local_package_info(app: AppHandle, store: State<'_, Store>, path: String) -> Result<LocalPackageInfo, String> {
    let directory = project_dir(&app, &store, &path)?;
    let spec = package_spec(&read_manifest(&directory)?)?;
    Ok(info_for(&spec))
}

#[tauri::command(async)]
pub fn install_local_package(app: AppHandle, store: State<'_, Store>, path: String) -> Result<LocalPackageInfo, String> {
    let directory = project_dir(&app, &store, &path)?;
    let manifest = read_manifest(&directory)?;
    let spec = package_spec(&manifest)?;

    let entrypoint = workspace::project_file_path(&directory, &manifest.package.entrypoint)?;
    if !entrypoint.is_file() {
        return Err(format!("The package entrypoint '{}' does not exist", manifest.package.entrypoint));
    }

    let package_dir = packages_root()?.join(spec.name.as_str());
    let version_dir = package_dir.join(spec.version.to_string());
    let staging_dir = package_dir.join(format!(".{}-installing", spec.version));

    if staging_dir.exists() {
        std::fs::remove_dir_all(&staging_dir).map_err(|error| error.to_string())?;
    }
    std::fs::create_dir_all(&staging_dir).map_err(|error| error.to_string())?;

    if let Err(error) = copy_project(&directory, &staging_dir) {
        let _ = std::fs::remove_dir_all(&staging_dir);
        return Err(error);
    }
    if version_dir.exists() {
        std::fs::remove_dir_all(&version_dir).map_err(|error| error.to_string())?;
    }
    std::fs::rename(&staging_dir, &version_dir).map_err(|error| error.to_string())?;

    Ok(info_for(&spec))
}

#[tauri::command]
pub fn remove_local_package(name: String, version: String) -> Result<Vec<String>, String> {
    let spec: PackageSpec = format!("@{NAMESPACE}/{name}:{version}")
        .parse()
        .map_err(|_| "Invalid package name or version".to_string())?;
    let package_dir = packages_root()?.join(spec.name.as_str());
    let version_dir = package_dir.join(spec.version.to_string());
    if version_dir.is_dir() {
        std::fs::remove_dir_all(&version_dir).map_err(|error| error.to_string())?;
    }
    let remaining = installed_versions(&spec.name);
    if remaining.is_empty() {
        let _ = std::fs::remove_dir(&package_dir);
    }
    Ok(remaining)
}
