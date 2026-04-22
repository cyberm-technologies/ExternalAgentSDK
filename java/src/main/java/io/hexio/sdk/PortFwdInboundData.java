package io.hexio.sdk;

import com.fasterxml.jackson.annotation.JsonInclude;

import java.util.List;

@JsonInclude(JsonInclude.Include.NON_NULL)
public class PortFwdInboundData {
    public List<String> opens;
    public List<PortFwdSend> sends;
    public List<String> closes;
}
