package io.hexio.sdk;

import com.fasterxml.jackson.annotation.JsonIgnoreProperties;

@JsonIgnoreProperties(ignoreUnknown = true)
public class PortFwdOpCloseResult {
    public long port;
    public boolean success;
}
