package io.hexio.sdk;

public class HexioApiException extends RuntimeException {
    private final int statusCode;

    public HexioApiException(int statusCode, String message) {
        super("API error (" + statusCode + "): " + message);
        this.statusCode = statusCode;
    }

    public int getStatusCode() {
        return statusCode;
    }
}
