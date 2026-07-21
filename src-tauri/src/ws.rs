use serde::{Deserialize, Serialize};
use std::net::TcpStream;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Mutex;
use std::time::Duration;
use tauri::{AppHandle, Emitter, Manager};
use tungstenite::client::IntoClientRequest;
use tungstenite::stream::MaybeTlsStream;
use tungstenite::Message;

pub const STATUS_EVENT: &str = "cloud://ws-status";
pub const SYNC_EVENT: &str = "cloud://sync-event";

const READ_TIMEOUT: Duration = Duration::from_secs(10);
const RECONNECT_DELAY: Duration = Duration::from_secs(5);

#[derive(Deserialize, Serialize, Clone)]
pub struct DeviceEvent {
    pub kind: String,
    pub project_id: Option<String>,
    pub document_id: Option<String>,
}

pub struct WsState {
    generation: AtomicU64,
    status: Mutex<String>,
}

impl Default for WsState {
    fn default() -> Self {
        Self {
            generation: AtomicU64::new(0),
            status: Mutex::new("offline".to_string()),
        }
    }
}

impl WsState {
    pub fn status(&self) -> String {
        self.status
            .lock()
            .map(|slot| slot.clone())
            .unwrap_or_else(|_| "offline".to_string())
    }

    fn set_status(&self, app: &AppHandle, status: &str) {
        if let Ok(mut slot) = self.status.lock() {
            *slot = status.to_string();
        }
        let _ = app.emit(STATUS_EVENT, status);
    }

    pub fn start(&self, app: AppHandle, server_url: String, token: String) {
        let generation = self.generation.fetch_add(1, Ordering::SeqCst) + 1;
        self.set_status(&app, "connecting");

        std::thread::spawn(move || run_loop(app, server_url, token, generation));
    }

    pub fn stop(&self, app: &AppHandle) {
        self.generation.fetch_add(1, Ordering::SeqCst);
        self.set_status(app, "offline");
    }
}

fn still_current(app: &AppHandle, generation: u64) -> bool {
    app.state::<WsState>().generation.load(Ordering::SeqCst) == generation
}

fn ws_url(server_url: &str) -> String {
    let trimmed = server_url.trim_end_matches('/');
    if let Some(rest) = trimmed.strip_prefix("https://") {
        format!("wss://{}/api/desktop/ws", rest)
    } else if let Some(rest) = trimmed.strip_prefix("http://") {
        format!("ws://{}/api/desktop/ws", rest)
    } else {
        format!("ws://{}/api/desktop/ws", trimmed)
    }
}

fn configure_read_timeout(stream: &MaybeTlsStream<TcpStream>) {
    let tcp = match stream {
        MaybeTlsStream::Plain(stream) => Some(stream),
        MaybeTlsStream::NativeTls(stream) => Some(stream.get_ref()),
        _ => None,
    };

    if let Some(tcp) = tcp {
        let _ = tcp.set_read_timeout(Some(READ_TIMEOUT));
    }
}

fn is_timeout(error: &tungstenite::Error) -> bool {
    matches!(
        error,
        tungstenite::Error::Io(io_error)
            if io_error.kind() == std::io::ErrorKind::WouldBlock
                || io_error.kind() == std::io::ErrorKind::TimedOut
    )
}

fn connect_and_listen(app: &AppHandle, url: &str, token: &str, generation: u64) -> Result<(), String> {
    let mut request = url
        .into_client_request()
        .map_err(|e| e.to_string())?;
    let header_value = format!("Bearer {}", token)
        .parse()
        .map_err(|_| "Invalid device token".to_string())?;
    request.headers_mut().insert("Authorization", header_value);

    let (mut socket, _response) = tungstenite::connect(request).map_err(|e| e.to_string())?;
    configure_read_timeout(socket.get_ref());

    app.state::<WsState>().set_status(app, "connected");

    loop {
        if !still_current(app, generation) {
            let _ = socket.close(None);
            return Ok(());
        }

        match socket.read() {
            Ok(Message::Text(text)) => {
                if let Ok(event) = serde_json::from_str::<DeviceEvent>(text.as_ref()) {
                    let _ = app.emit(SYNC_EVENT, event);
                }
            }
            Ok(Message::Ping(_)) => {
                let _ = socket.flush();
            }
            Ok(Message::Close(_)) => return Ok(()),
            Ok(_) => {}
            Err(ref error) if is_timeout(error) => continue,
            Err(error) => return Err(error.to_string()),
        }
    }
}

fn run_loop(app: AppHandle, server_url: String, token: String, generation: u64) {
    let url = ws_url(&server_url);

    loop {
        if !still_current(&app, generation) {
            return;
        }

        let _ = connect_and_listen(&app, &url, &token, generation);

        if !still_current(&app, generation) {
            return;
        }

        app.state::<WsState>().set_status(&app, "offline");
        std::thread::sleep(RECONNECT_DELAY);
    }
}
