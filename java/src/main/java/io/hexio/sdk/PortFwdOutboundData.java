package io.hexio.sdk;

import com.fasterxml.jackson.annotation.JsonIgnoreProperties;

import java.util.List;

@JsonIgnoreProperties(ignoreUnknown = true)
public class PortFwdOutboundData {
    public List<PortFwdRecv> recvs;
    public List<String> closes;
}
