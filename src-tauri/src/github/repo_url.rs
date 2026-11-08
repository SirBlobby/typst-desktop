const MAX_BRANCH_LENGTH: usize = 200;
const MAX_EXCLUDED_PATHS: usize = 100;
const MAX_EXCLUDED_PATH_LENGTH: usize = 512;
const FORBIDDEN_BRANCH_CHARACTERS: &[char] = &[' ', '~', '^', ':', '?', '*', '[', '\\'];

#[derive(Clone)]
pub struct RepoUrl {
    pub authority: String,
    pub path: String,
    pub text: String,
}

fn is_safe_segment(segment: &str) -> bool {
    !segment.is_empty()
        && segment != "."
        && segment != ".."
        && segment.chars().all(|character| character.is_ascii_alphanumeric() || matches!(character, '-' | '_' | '.'))
}

fn is_safe_authority(authority: &str) -> bool {
    !authority.is_empty()
        && !authority.starts_with(':')
        && authority.chars().all(|character| character.is_ascii_alphanumeric() || matches!(character, '-' | '.' | ':'))
}

impl RepoUrl {
    pub fn parse(input: &str) -> Result<Self, String> {
        let trimmed = input.trim().trim_end_matches('/');
        let rest = trimmed
            .strip_prefix("https://")
            .ok_or("Use the https:// address of the repository, like https://github.com/owner/repository")?;

        let (authority, full_path) = rest.split_once('/').ok_or("The address needs the owner and repository name")?;
        if authority.contains('@') {
            return Err("Put the access token in the token field instead of the address".to_string());
        }
        if !is_safe_authority(authority) {
            return Err("The repository address has an unsupported host name".to_string());
        }

        let path = full_path.strip_suffix(".git").unwrap_or(full_path);
        let segments: Vec<&str> = path.split('/').collect();
        if segments.len() != 2 || !segments.iter().all(|segment| is_safe_segment(segment)) {
            return Err("The address needs the owner and repository name, like https://github.com/owner/repository".to_string());
        }

        let authority = authority.to_ascii_lowercase();
        Ok(Self {
            text: format!("https://{authority}/{path}"),
            authority,
            path: path.to_string(),
        })
    }

    pub fn github_com() -> Self {
        Self {
            authority: "github.com".to_string(),
            path: String::new(),
            text: "https://github.com".to_string(),
        }
    }

    pub fn is_github_com(&self) -> bool {
        matches!(self.authority.as_str(), "github.com" | "www.github.com")
    }

    pub fn api_base(&self) -> String {
        if self.is_github_com() {
            "https://api.github.com".to_string()
        } else {
            format!("https://{}/api/v3", self.authority)
        }
    }
}

pub fn is_clean_path(path: &str) -> bool {
    !path.is_empty()
        && !path.starts_with('/')
        && !path.contains('\\')
        && path.split('/').all(|segment| !segment.is_empty() && segment != "." && segment != "..")
}

pub fn clean_branch(branch: &str) -> Result<String, String> {
    let branch = branch.trim();
    let is_valid = !branch.is_empty()
        && branch.len() <= MAX_BRANCH_LENGTH
        && !branch.starts_with(['-', '/', '.'])
        && !branch.ends_with(['/', '.'])
        && !branch.ends_with(".lock")
        && !branch.contains("..")
        && !branch.contains("//")
        && !branch.contains("@{")
        && !branch.chars().any(|character| character.is_control() || FORBIDDEN_BRANCH_CHARACTERS.contains(&character));
    if is_valid {
        Ok(branch.to_string())
    } else {
        Err("That is not a valid branch name".to_string())
    }
}

pub fn clean_folder(folder: &str) -> Result<String, String> {
    let folder = folder.trim().trim_matches('/');
    if folder.is_empty() {
        return Ok(String::new());
    }
    if is_clean_path(folder) {
        Ok(folder.to_string())
    } else {
        Err("That is not a valid folder in the repository".to_string())
    }
}

pub fn clean_excluded_paths(paths: &[String]) -> Result<Vec<String>, String> {
    let cleaned: Vec<String> = paths
        .iter()
        .map(|path| path.trim().trim_start_matches("./").trim_start_matches('/').to_string())
        .filter(|path| !path.is_empty())
        .collect();
    if cleaned.len() > MAX_EXCLUDED_PATHS {
        return Err("Too many excluded paths".to_string());
    }
    if cleaned.iter().any(|path| path.len() > MAX_EXCLUDED_PATH_LENGTH || path.chars().any(char::is_control)) {
        return Err("Excluded paths must be short and on one line each".to_string());
    }
    Ok(cleaned)
}

pub fn clean_token(token: Option<&str>) -> Option<String> {
    token.map(str::trim).filter(|token| !token.is_empty()).map(str::to_string)
}
