package io.hexio.sdk;

import com.fasterxml.jackson.annotation.JsonIgnoreProperties;
import com.fasterxml.jackson.annotation.JsonProperty;

@JsonIgnoreProperties(ignoreUnknown = true)
public class DownloadInitResponse {
    @JsonProperty("download_id")
    public String downloadId;

    @JsonProperty("agent_path")
    public String agentPath;
}
