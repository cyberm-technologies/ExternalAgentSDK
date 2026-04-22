package io.hexio.sdk;

import com.fasterxml.jackson.annotation.JsonProperty;

public class DownloadInitRequest {
    @JsonProperty("file_name")
    public String fileName;

    @JsonProperty("agent_path")
    public String agentPath;

    @JsonProperty("file_size")
    public long fileSize;

    @JsonProperty("chunk_size")
    public long chunkSize;

    @JsonProperty("total_chunks")
    public long totalChunks;
}
