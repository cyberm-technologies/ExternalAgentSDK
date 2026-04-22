package io.hexio.sdk;

import com.fasterxml.jackson.annotation.JsonInclude;
import com.fasterxml.jackson.annotation.JsonProperty;

@JsonInclude(JsonInclude.Include.NON_NULL)
public class PortFwdOpenRequest {
    public long port;

    @JsonProperty("remote_host")
    public String remoteHost;

    @JsonProperty("remote_port")
    public long remotePort;
}
