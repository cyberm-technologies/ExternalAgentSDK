package io.hexio.sdk;

import com.fasterxml.jackson.annotation.JsonInclude;
import com.fasterxml.jackson.annotation.JsonProperty;

@JsonInclude(JsonInclude.Include.NON_NULL)
public class PortFwdSend {
    @JsonProperty("sockid")
    public String sockId;

    public String data;
    public long size;
}
