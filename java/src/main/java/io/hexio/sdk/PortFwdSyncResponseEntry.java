package io.hexio.sdk;

import com.fasterxml.jackson.annotation.JsonIgnoreProperties;

@JsonIgnoreProperties(ignoreUnknown = true)
public class PortFwdSyncResponseEntry {
    public long port;
    public PortFwdOutboundData data;
}
