package io.hexio.sdk;

import com.fasterxml.jackson.annotation.JsonInclude;

@JsonInclude(JsonInclude.Include.NON_NULL)
public class PortFwdSyncRequestEntry {
    public long port;
    public PortFwdInboundData data;
}
