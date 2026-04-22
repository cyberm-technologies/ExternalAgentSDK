package io.hexio.sdk;

import com.fasterxml.jackson.annotation.JsonInclude;
import com.fasterxml.jackson.annotation.JsonProperty;

import java.util.HashMap;
import java.util.List;

/**
 * Full batched payload for POST /agent/sync. All fields are optional; only set
 * the ones you need. For socks_open / socks_close, assign {@code new HashMap<>()}
 * (or call {@link #triggerSocksOpen()} / {@link #triggerSocksClose()}) to include
 * the key as an empty object.
 */
@JsonInclude(JsonInclude.Include.NON_NULL)
public class SyncRequest {
    public SleepUpdate sleep;

    public String impersonation;

    public List<CommandResult> commands;

    @JsonProperty("side_channel_responses")
    public List<SideChannelResponse> sideChannelResponses;

    @JsonProperty("download_init")
    public List<DownloadInitRequest> downloadInit;

    @JsonProperty("download_chunk")
    public List<DownloadChunkUpload> downloadChunk;

    @JsonProperty("download_cancel")
    public List<String> downloadCancel;

    public List<ScreenshotUpload> screenshots;

    public KeylogUpload keylog;

    @JsonProperty("bof_files")
    public List<String> bofFiles;

    @JsonProperty("pe_files")
    public List<String> peFiles;

    @JsonProperty("dll_files")
    public List<String> dllFiles;

    @JsonProperty("elf_files")
    public List<String> elfFiles;

    @JsonProperty("macho_files")
    public List<String> machoFiles;

    @JsonProperty("shellcode_files")
    public List<String> shellcodeFiles;

    public List<String> hexlang;

    /** Presence flag: set to {@code new HashMap<>()} to trigger opening the SOCKS proxy. */
    @JsonProperty("socks_open")
    public Object socksOpen;

    @JsonProperty("socks_open_port")
    public Long socksOpenPort;

    /** Presence flag: set to {@code new HashMap<>()} to trigger closing the SOCKS proxy. */
    @JsonProperty("socks_close")
    public Object socksClose;

    @JsonProperty("socks_sync")
    public SocksSyncRequest socksSync;

    @JsonProperty("portfwd_open")
    public List<PortFwdOpenRequest> portFwdOpen;

    @JsonProperty("portfwd_close")
    public List<Long> portFwdClose;

    @JsonProperty("portfwd_sync")
    public List<PortFwdSyncRequestEntry> portFwdSync;

    /** Convenience helper: mark {@code socks_open} as present. */
    public void triggerSocksOpen() {
        this.socksOpen = new HashMap<String, Object>();
    }

    /** Convenience helper: mark {@code socks_close} as present. */
    public void triggerSocksClose() {
        this.socksClose = new HashMap<String, Object>();
    }
}
