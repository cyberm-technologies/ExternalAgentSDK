package io.hexio.sdk;

import com.fasterxml.jackson.annotation.JsonInclude;

@JsonInclude(JsonInclude.Include.NON_NULL)
public class ScreenshotUpload {
    public String filename;
    public String data;
}
