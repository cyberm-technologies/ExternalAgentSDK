package io.hexio.sdk;

import com.fasterxml.jackson.annotation.JsonIgnoreProperties;

@JsonIgnoreProperties(ignoreUnknown = true)
public class SocksOpenEntry {
    public String id;
    public String addr;
    public long port;
    public String proto;
}
