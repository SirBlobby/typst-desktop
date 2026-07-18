use serde::Serialize;
use std::io::{BufReader, Read, Write};
use std::path::Path;
use std::process::{Child, ChildStdin, Command, Stdio};
use std::sync::Mutex;
use tauri::{AppHandle, Emitter};

pub const MESSAGE_EVENT: &str = "lsp://message";
pub const CLOSED_EVENT: &str = "lsp://closed";

#[derive(Default)]
pub struct LspState {
    inner: Mutex<Option<Session>>,
}

struct Session {
    child: Child,
    stdin: ChildStdin,
}

#[derive(Serialize, Clone)]
pub struct LspHandle {
    pub root_uri: String,
    pub document_uri: String,
}

fn file_uri(path: &Path) -> String {
    let text = path.to_string_lossy().replace('\\', "/");
    if text.starts_with('/') {
        format!("file://{}", text)
    } else {
        format!("file:///{}", text)
    }
}

impl LspState {
    pub fn is_running(&self) -> bool {
        self.inner.lock().map(|slot| slot.is_some()).unwrap_or(false)
    }

    pub fn start(
        &self,
        app: &AppHandle,
        root: &Path,
        entrypoint: &str,
    ) -> Result<LspHandle, String> {
        self.stop();

        let mut child = Command::new("tinymist")
            .arg("lsp")
            .current_dir(root)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::null())
            .spawn()
            .map_err(|e| {
                format!(
                    "Could not start the Typst language server (tinymist): {}. \
                     Install tinymist and make sure it is on your PATH.",
                    e
                )
            })?;

        let stdin = child.stdin.take().ok_or("Language server has no stdin")?;
        let stdout = child.stdout.take().ok_or("Language server has no stdout")?;

        let emitter = app.clone();
        std::thread::spawn(move || {
            let mut reader = BufReader::new(stdout);
            loop {
                match read_message(&mut reader) {
                    Some(message) => {
                        let _ = emitter.emit(MESSAGE_EVENT, message);
                    }
                    None => {
                        let _ = emitter.emit(CLOSED_EVENT, ());
                        break;
                    }
                }
            }
        });

        let handle = LspHandle {
            root_uri: file_uri(root),
            document_uri: file_uri(&root.join(entrypoint)),
        };

        let mut slot = self.inner.lock().map_err(|_| "Language server lock poisoned")?;
        *slot = Some(Session { child, stdin });

        Ok(handle)
    }

    pub fn send(&self, message: &str) -> Result<(), String> {
        let mut slot = self.inner.lock().map_err(|_| "Language server lock poisoned")?;
        let session = slot.as_mut().ok_or("Language server is not running")?;

        session
            .stdin
            .write_all(format!("Content-Length: {}\r\n\r\n", message.len()).as_bytes())
            .map_err(|e| e.to_string())?;
        session
            .stdin
            .write_all(message.as_bytes())
            .map_err(|e| e.to_string())?;
        session.stdin.flush().map_err(|e| e.to_string())
    }

    pub fn stop(&self) {
        if let Ok(mut slot) = self.inner.lock() {
            if let Some(mut session) = slot.take() {
                let _ = session.child.kill();
                let _ = session.child.wait();
            }
        }
    }
}

fn read_message(reader: &mut BufReader<std::process::ChildStdout>) -> Option<String> {
    let mut header = String::new();

    loop {
        let mut byte = [0u8; 1];
        if reader.read_exact(&mut byte).is_err() {
            return None;
        }
        header.push(byte[0] as char);
        if header.ends_with("\r\n\r\n") {
            break;
        }
        if header.len() > 8192 {
            return None;
        }
    }

    let mut content_length = 0usize;
    for line in header.split("\r\n") {
        if let Some(value) = line.strip_prefix("Content-Length: ") {
            content_length = value.trim().parse().unwrap_or(0);
        }
    }

    if content_length == 0 {
        return Some(String::new());
    }

    let mut body = vec![0u8; content_length];
    if reader.read_exact(&mut body).is_err() {
        return None;
    }

    String::from_utf8(body).ok()
}
