package io.hexio.sdk;

import com.fasterxml.jackson.annotation.JsonInclude;

import java.util.List;

@JsonInclude(JsonInclude.Include.NON_NULL)
public class SocksSyncRequest {
    public List<String> closes;
    public List<SocksReceive> receives;
}
