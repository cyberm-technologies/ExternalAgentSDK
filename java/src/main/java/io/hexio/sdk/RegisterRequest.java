package io.hexio.sdk;

import com.fasterxml.jackson.annotation.JsonProperty;

public class RegisterRequest {
    public String hostname;
    public String ip;
    public String user;
    public String os;
    public String process;
    public String arch;
    public long pid;

    @JsonProperty("client_type")
    public String clientType;

    @JsonProperty("sleep_time")
    public long sleepTime;
}
