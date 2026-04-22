package io.hexio.sdk;

import com.fasterxml.jackson.annotation.JsonIgnoreProperties;

@JsonIgnoreProperties(ignoreUnknown = true)
public class SocksSend {
    public String id;
    public String data;
    public long size;
}
