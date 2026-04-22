package io.hexio.sdk;

import com.fasterxml.jackson.annotation.JsonInclude;

@JsonInclude(JsonInclude.Include.NON_NULL)
public class SocksReceive {
    public String id;
    public String data;
}
