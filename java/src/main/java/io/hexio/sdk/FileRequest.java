package io.hexio.sdk;

import com.fasterxml.jackson.annotation.JsonInclude;
import com.fasterxml.jackson.annotation.JsonProperty;

import java.util.List;

@JsonInclude(JsonInclude.Include.NON_NULL)
public class FileRequest {
    @JsonProperty("bof_files")
    public List<String> bofFiles;

    @JsonProperty("pe_files")
    public List<String> peFiles;

    @JsonProperty("dll_files")
    public List<String> dllFiles;

    @JsonProperty("elf_files")
    public List<String> elfFiles;

    @JsonProperty("macho_files")
    public List<String> machoFiles;

    @JsonProperty("shellcode_files")
    public List<String> shellcodeFiles;

    public List<String> hexlang;
}
