use crate::workspace::is_text_file;

pub const MAX_FILE_BYTES: u64 = 50 * 1024 * 1024;
pub const MAX_SYNCED_FILES: usize = 5000;

const ASSET_EXTENSIONS: &[&str] = &[
    ".png", ".jpg", ".jpeg", ".gif", ".svg", ".webp", ".pdf", ".ttf", ".otf", ".ttc", ".otc", ".woff", ".woff2",
];

#[derive(Clone, Default)]
pub struct FileRules {
    excluded: Vec<String>,
}

impl FileRules {
    pub fn new(excluded: Vec<String>) -> Self {
        Self { excluded }
    }

    fn is_excluded(&self, path: &str) -> bool {
        let lower = path.to_ascii_lowercase();
        self.excluded.iter().any(|rule| match rule.strip_prefix('*') {
            Some(suffix) => lower.ends_with(&suffix.to_ascii_lowercase()),
            None => {
                let folder = rule.trim_end_matches('/');
                path == folder || path.starts_with(&format!("{folder}/"))
            }
        })
    }

    pub fn is_managed(&self, path: &str) -> bool {
        let lower = path.to_ascii_lowercase();
        let is_document_file = is_text_file(path) || ASSET_EXTENSIONS.iter().any(|extension| lower.ends_with(extension));
        is_document_file && !self.is_excluded(path)
    }
}

pub fn check_file_count(count: usize) -> Result<(), String> {
    if count > MAX_SYNCED_FILES {
        return Err(format!(
            "The folder has {count} document files, more than the {MAX_SYNCED_FILES} that can be synced. Link a smaller folder or exclude paths."
        ));
    }
    Ok(())
}
