use std::path::Path;

use serde::Serialize;
use typst::syntax::package::PackageManifest;

use super::{installed_versions, packages_root, NAMESPACE};

#[derive(Serialize)]
pub struct LocalPackageVersion {
    pub version: String,
    pub description: Option<String>,
    pub authors: Vec<String>,
    pub license: Option<String>,
    pub import_line: String,
    pub folder: String,
}

#[derive(Serialize)]
pub struct LocalPackage {
    pub name: String,
    pub versions: Vec<LocalPackageVersion>,
}

fn read_version(name: &str, version: &str, folder: &Path) -> LocalPackageVersion {
    let manifest = std::fs::read_to_string(folder.join("typst.toml"))
        .ok()
        .and_then(|text| toml::from_str::<PackageManifest>(&text).ok());
    let info = manifest.map(|manifest| manifest.package);

    LocalPackageVersion {
        version: version.to_string(),
        description: info.as_ref().and_then(|info| info.description.as_ref().map(|text| text.to_string())),
        authors: info
            .as_ref()
            .map(|info| info.authors.iter().map(|author| author.to_string()).collect())
            .unwrap_or_default(),
        license: info.as_ref().and_then(|info| info.license.as_ref().map(|text| text.to_string())),
        import_line: format!("#import \"@{NAMESPACE}/{name}:{version}\": *"),
        folder: folder.to_string_lossy().to_string(),
    }
}

#[tauri::command]
pub fn list_local_packages() -> Result<Vec<LocalPackage>, String> {
    let root = packages_root()?;
    let Ok(entries) = std::fs::read_dir(&root) else {
        return Ok(Vec::new());
    };

    let mut packages: Vec<LocalPackage> = entries
        .filter_map(Result::ok)
        .filter(|entry| entry.path().is_dir())
        .map(|entry| entry.file_name().to_string_lossy().to_string())
        .filter(|name| !name.starts_with('.'))
        .filter_map(|name| {
            let versions: Vec<LocalPackageVersion> = installed_versions(&name)
                .iter()
                .map(|version| read_version(&name, version, &root.join(&name).join(version)))
                .collect();
            (!versions.is_empty()).then_some(LocalPackage { name, versions })
        })
        .collect();
    packages.sort_by(|left, right| left.name.cmp(&right.name));
    Ok(packages)
}

#[tauri::command]
pub fn local_packages_folder() -> Result<String, String> {
    let root = packages_root()?;
    std::fs::create_dir_all(&root).map_err(|error| error.to_string())?;
    Ok(root.to_string_lossy().to_string())
}
