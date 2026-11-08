use std::path::PathBuf;

pub fn program_path(name: &str) -> PathBuf {
    let file_name = if cfg!(windows) {
        format!("{}.exe", name)
    } else {
        name.to_string()
    };
    std::env::current_exe()
        .ok()
        .and_then(|executable| executable.parent().map(|dir| dir.join(&file_name)))
        .filter(|bundled| bundled.is_file())
        .unwrap_or_else(|| PathBuf::from(name))
}
