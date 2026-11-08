use std::io::Read;
use std::sync::OnceLock;
use std::time::Duration;

use serde_json::Value;

use super::file_rules::MAX_FILE_BYTES;

const REQUEST_TIMEOUT_SECONDS: u64 = 120;
const MAX_ERROR_BODY_BYTES: u64 = 64 * 1024;
const MAX_MESSAGE_CHARACTERS: usize = 300;
const USER_AGENT: &str = "TypstDesktop (GitHub sync)";
const SERVICE: &str = "GitHub";

pub struct Client {
    agent: ureq::Agent,
    base: String,
    headers: Vec<(&'static str, String)>,
}

fn remote_message(response: ureq::Response) -> String {
    let mut body = String::new();
    let _ = response.into_reader().take(MAX_ERROR_BODY_BYTES).read_to_string(&mut body);
    let parsed: Option<Value> = serde_json::from_str(&body).ok();
    let message = parsed.as_ref().and_then(|value| value.get("message").or_else(|| value.get("error")));
    let text = match message {
        Some(Value::String(text)) => text.clone(),
        Some(other) => other.to_string(),
        None => String::new(),
    };
    text.chars().filter(|character| !character.is_control()).take(MAX_MESSAGE_CHARACTERS).collect()
}

fn shared_agent() -> ureq::Agent {
    static AGENT: OnceLock<ureq::Agent> = OnceLock::new();
    AGENT
        .get_or_init(|| {
            ureq::AgentBuilder::new()
                .timeout(Duration::from_secs(REQUEST_TIMEOUT_SECONDS))
                .user_agent(USER_AGENT)
                .redirects(0)
                .build()
        })
        .clone()
}

fn status_error(code: u16, message: String) -> String {
    let detail = if message.is_empty() { String::new() } else { format!(" ({message})") };
    match code {
        401 => format!("{SERVICE} did not accept the access token. Update the token in the link settings."),
        403 => format!("{SERVICE} refused access{detail}. The token needs read and write access to this repository's contents."),
        404 => format!("{SERVICE} could not find the repository, branch or folder. Private repositories need an access token."),
        409 | 422 => format!("{SERVICE} rejected the change{detail}. Pull the latest changes and try again."),
        429 => format!("{SERVICE} is rate limiting requests. Try again in a minute."),
        _ => format!("{SERVICE} answered with status {code}{detail}"),
    }
}

pub fn unreadable() -> String {
    format!("{SERVICE} sent an unreadable answer")
}

impl Client {
    pub fn new(base: String, token: Option<&str>) -> Self {
        let mut headers = vec![
            ("Accept", "application/vnd.github+json".to_string()),
            ("X-GitHub-Api-Version", "2022-11-28".to_string()),
        ];
        if let Some(token) = token {
            headers.push(("Authorization", format!("Bearer {token}")));
        }
        Self { agent: shared_agent(), base, headers }
    }

    fn request(&self, method: &str, path: &str) -> ureq::Request {
        self.headers
            .iter()
            .fold(self.agent.request(method, &format!("{}{path}", self.base)), |request, (name, value)| request.set(name, value))
    }

    fn finish(&self, result: Result<ureq::Response, ureq::Error>) -> Result<ureq::Response, String> {
        match result {
            Ok(response) if (300..400).contains(&response.status()) => {
                Err(format!("{SERVICE} redirected the request. Use the current address of the repository."))
            }
            Ok(response) => Ok(response),
            Err(ureq::Error::Status(code, response)) => Err(status_error(code, remote_message(response))),
            Err(ureq::Error::Transport(error)) => Err(format!("Could not reach {SERVICE}: {error}")),
        }
    }

    pub fn get(&self, path: &str, extra_headers: &[(&str, &str)]) -> Result<ureq::Response, String> {
        let request = extra_headers
            .iter()
            .fold(self.request("GET", path), |request, (name, value)| request.set(name, value));
        self.finish(request.call())
    }

    pub fn get_json(&self, path: &str) -> Result<Value, String> {
        self.get(path, &[])?.into_json::<Value>().map_err(|_| unreadable())
    }

    pub fn send_json(&self, method: &str, path: &str, body: &Value) -> Result<Value, String> {
        self.finish(self.request(method, path).send_json(body))?
            .into_json::<Value>()
            .map_err(|_| unreadable())
    }

    pub fn read_limited(&self, response: ureq::Response) -> Result<Option<Vec<u8>>, String> {
        let mut bytes = Vec::new();
        response
            .into_reader()
            .take(MAX_FILE_BYTES + 1)
            .read_to_end(&mut bytes)
            .map_err(|_| unreadable())?;
        Ok((bytes.len() as u64 <= MAX_FILE_BYTES).then_some(bytes))
    }
}

pub fn text_field(value: &Value, pointer: &str) -> Option<String> {
    value.pointer(pointer).and_then(Value::as_str).map(str::to_string)
}

pub fn encode_segment(value: &str) -> String {
    value
        .bytes()
        .map(|byte| match byte {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => (byte as char).to_string(),
            _ => format!("%{byte:02X}"),
        })
        .collect()
}
