use base64::{engine::general_purpose::STANDARD as BASE64, Engine as _};
use serde_json::Value;
use std::io::Read;

const REQUEST_TIMEOUT_SECS: u64 = 30;
const ALLOWED_PREFIXES: [&str; 3] = ["/documents/", "/comments/", "/suggestions/"];
const ALLOWED_METHODS: [&str; 5] = ["GET", "POST", "PUT", "PATCH", "DELETE"];
const MAX_AVATAR_BYTES: u64 = 4 * 1024 * 1024;

fn agent() -> ureq::Agent {
    ureq::AgentBuilder::new()
        .timeout(std::time::Duration::from_secs(REQUEST_TIMEOUT_SECS))
        .build()
}

fn endpoint(server_url: &str, path: &str) -> String {
    format!("{}/api/desktop{}", server_url.trim_end_matches('/'), path)
}

fn describe(error: ureq::Error) -> String {
    match error {
        ureq::Error::Status(code, response) => {
            let body = response.into_string().unwrap_or_default();
            if body.is_empty() { format!("Server returned {}", code) } else { body }
        }
        other => other.to_string(),
    }
}

fn check_request(method: &str, path: &str) -> Result<(), String> {
    if !ALLOWED_METHODS.contains(&method) {
        return Err(format!("Unsupported method {method}"));
    }
    let allowed_path = ALLOWED_PREFIXES.iter().any(|prefix| path.starts_with(prefix)) && !path.contains("..");
    if !allowed_path {
        return Err(format!("Unsupported path {path}"));
    }
    Ok(())
}

pub fn request(server_url: &str, token: &str, method: &str, path: &str, body: Option<Value>) -> Result<Value, String> {
    check_request(method, path)?;
    let pending = agent()
        .request(method, &endpoint(server_url, path))
        .set("Authorization", &format!("Bearer {}", token));
    let response = match body {
        Some(json) => pending.send_json(json),
        None => pending.call(),
    }
    .map_err(describe)?;

    let text = response.into_string().map_err(|error| error.to_string())?;
    if text.trim().is_empty() {
        return Ok(Value::Null);
    }
    serde_json::from_str(&text).map_err(|error| error.to_string())
}

pub fn user_avatar(server_url: &str, token: &str, user_id: &str) -> Result<Option<String>, String> {
    if user_id.is_empty() || !user_id.chars().all(|character| character.is_ascii_alphanumeric() || character == '-') {
        return Err("Invalid user id".to_string());
    }
    let response = match agent()
        .get(&endpoint(server_url, &format!("/users/{user_id}/avatar")))
        .set("Authorization", &format!("Bearer {}", token))
        .call()
    {
        Ok(response) => response,
        Err(ureq::Error::Status(404, _)) => return Ok(None),
        Err(error) => return Err(describe(error)),
    };

    let content_type = response.content_type().to_string();
    let mut bytes = Vec::new();
    response
        .into_reader()
        .take(MAX_AVATAR_BYTES)
        .read_to_end(&mut bytes)
        .map_err(|error| error.to_string())?;
    Ok(Some(format!("data:{};base64,{}", content_type, BASE64.encode(bytes))))
}
