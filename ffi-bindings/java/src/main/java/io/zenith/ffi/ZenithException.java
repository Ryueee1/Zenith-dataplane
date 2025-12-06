package io.zenith.ffi;

/**
 * Exception thrown by Zenith operations
 */
public class ZenithException extends Exception {
    
    private final int errorCode;
    
    public static final int OK = 0;
    public static final int NULL_PTR = -1;
    public static final int BUFFER_FULL = -2;
    public static final int PLUGIN_LOAD = -3;
    public static final int FFI_ERROR = -4;
    
    public ZenithException(int errorCode, String message) {
        super(message);
        this.errorCode = errorCode;
    }
    
    public int getErrorCode() {
        return errorCode;
    }
    
    @Override
    public String toString() {
        return String.format("ZenithException[code=%d]: %s", errorCode, getMessage());
    }
}
