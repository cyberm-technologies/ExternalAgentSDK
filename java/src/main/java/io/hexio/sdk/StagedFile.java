package io.hexio.sdk;

import com.fasterxml.jackson.annotation.JsonIgnoreProperties;

@JsonIgnoreProperties(ignoreUnknown = true)
public class StagedFile {
    public String filename;
    public String filetype;
    public String alias;
    public String filedata;
}
