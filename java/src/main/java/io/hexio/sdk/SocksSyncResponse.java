package io.hexio.sdk;

import com.fasterxml.jackson.annotation.JsonIgnoreProperties;

import java.util.List;

@JsonIgnoreProperties(ignoreUnknown = true)
public class SocksSyncResponse {
    public List<SocksOpenEntry> opens;
    public List<String> closes;
    /** Note: this field is singular `send` in the wire protocol. */
    public List<SocksSend> send;
}
