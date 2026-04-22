package io.hexio.sdk;

import com.fasterxml.jackson.annotation.JsonIgnoreProperties;
import com.fasterxml.jackson.databind.JsonNode;

import java.util.ArrayList;
import java.util.List;

@JsonIgnoreProperties(ignoreUnknown = true)
public class CheckinResponse {
    public List<Command> commands = new ArrayList<>();
    public List<JsonNode> files;
}
