package io.hexio.sdk;

import com.fasterxml.jackson.annotation.JsonInclude;
import com.fasterxml.jackson.annotation.JsonProperty;

@JsonInclude(JsonInclude.Include.NON_NULL)
public class DownloadChunkUpload {
    @JsonProperty("download_id")
    public String downloadId;

    @JsonProperty("chunk_data")
    public String chunkData;
}
