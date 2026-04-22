package io.hexio.sdk;

import com.fasterxml.jackson.annotation.JsonInclude;
import com.fasterxml.jackson.annotation.JsonProperty;

@JsonInclude(JsonInclude.Include.NON_NULL)
public class CommandResult {
    @JsonProperty("command_id")
    public Long commandId;

    public String command;

    public String response;
}
