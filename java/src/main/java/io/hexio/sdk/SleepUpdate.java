package io.hexio.sdk;

import com.fasterxml.jackson.annotation.JsonInclude;
import com.fasterxml.jackson.annotation.JsonProperty;

@JsonInclude(JsonInclude.Include.NON_NULL)
public class SleepUpdate {
    @JsonProperty("sleep_time")
    public Long sleepTime;

    @JsonProperty("sleep_jitter")
    public Long sleepJitter;
}
