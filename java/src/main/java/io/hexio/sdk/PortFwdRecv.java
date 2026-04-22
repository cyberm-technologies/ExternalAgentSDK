package io.hexio.sdk;

import com.fasterxml.jackson.annotation.JsonIgnoreProperties;
import com.fasterxml.jackson.annotation.JsonProperty;

@JsonIgnoreProperties(ignoreUnknown = true)
public class PortFwdRecv {
    @JsonProperty("sockid")
    public String sockId;

    public String data;
    public long size;
}
