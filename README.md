# Hexio External Agent SDK

SDKs and specification for building custom agents and connectors against the Hexio C2 teamserver's External Agent REST API.

The External Agent API exposes all core C2 functionality -- command queuing, file transfer, screenshots, keylogging, SOCKS5 proxying, port forwarding, and companion client integration -- through a simple JSON/HTTP interface. Bring your own tradecraft and transport; the teamserver handles the rest.

## Repository Layout

| Path | Contents |
|------|----------|
| [APISPEC.md](APISPEC.md) | Full REST API specification -- endpoints, auth, request/response schemas |
| [go/](go/) | Go SDK |
| [python/](python/) | Python SDK (`pip install hexio-sdk`) |
| [typescript/](typescript/) | TypeScript / JavaScript SDK (Node 18+, Bun, Deno, browsers) |
| [rust/](rust/) | Rust SDK |
| [java/](java/) | Java SDK |
| [csharp/](csharp/) | C# / .NET SDK |
| [cpp/](cpp/) | C++ SDK (single-header) |
| [images/](images/) | Architecture and flow diagrams referenced by the spec |

Each SDK directory has its own README with install and usage details.

## Getting Started

1. Read [APISPEC.md](APISPEC.md) for the full endpoint reference and authentication model.
2. Pick the SDK that matches your agent's language, or call the REST API directly.
3. Create a `client_type.json` describing your agent's commands and supported features (see the [Client Type Configuration](APISPEC.md#client-type-configuration) section of the spec).
4. Register your agent via `POST /register`, then beacon using either `/agent/sync` (batched, recommended) or the individual endpoints.

## Architecture

![Architecture](images/architecture.svg)

Your custom agent can talk to the ExternalAgent Listener directly, or go through a connector that translates a custom transport (WebSocket, DNS, ICMP, etc.) into HTTP/JSON calls.

## Features

- Command queuing and execution results
- File download (exfiltration) and upload
- Screenshot and keylog submission
- Interactive side-channel sessions
- SOCKS5 proxy tunneling
- TCP port forwarding
- BOF, PE, DLL, ELF, Mach-O, shellcode, and HexLang file delivery
- User impersonation tracking
- HexioScript integration via `self.agent_type` / `self.agent_external`
