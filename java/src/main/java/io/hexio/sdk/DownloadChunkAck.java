package io.hexio.sdk;

import com.fasterxml.jackson.annotation.JsonIgnoreProperties;
import com.fasterxml.jackson.annotation.JsonProperty;

@JsonIgnoreProperties(ignoreUnknown = true)
public class DownloadChunkAck {
    @JsonProperty("download_id")
    public String downloadId;

    @JsonProperty("chunk_received")
    public boolean chunkReceived;
}
