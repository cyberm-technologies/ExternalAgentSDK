package io.hexio.sdk;

import com.fasterxml.jackson.annotation.JsonIgnoreProperties;
import com.fasterxml.jackson.annotation.JsonProperty;

import java.util.List;
import java.util.Map;

@JsonIgnoreProperties(ignoreUnknown = true)
public class SyncResponse {
    public List<Command> commands;

    public List<StagedFile> files;

    @JsonProperty("download_init")
    public List<DownloadInitResponse> downloadInit;

    @JsonProperty("download_chunk")
    public List<DownloadChunkAck> downloadChunk;

    @JsonProperty("bof_files")
    public Map<String, String> bofFiles;

    @JsonProperty("pe_files")
    public Map<String, String> peFiles;

    @JsonProperty("dll_files")
    public Map<String, String> dllFiles;

    @JsonProperty("elf_files")
    public Map<String, String> elfFiles;

    @JsonProperty("macho_files")
    public Map<String, String> machoFiles;

    @JsonProperty("shellcode_files")
    public Map<String, String> shellcodeFiles;

    public Map<String, String> hexlang;

    @JsonProperty("socks_open")
    public Boolean socksOpen;

    @JsonProperty("socks_port")
    public Long socksPort;

    @JsonProperty("socks_close")
    public Boolean socksClose;

    @JsonProperty("socks_sync")
    public SocksSyncResponse socksSync;

    @JsonProperty("portfwd_open")
    public List<PortFwdOpCloseResult> portFwdOpen;

    @JsonProperty("portfwd_close")
    public List<PortFwdOpCloseResult> portFwdClose;

    @JsonProperty("portfwd_sync")
    public List<PortFwdSyncResponseEntry> portFwdSync;
}
