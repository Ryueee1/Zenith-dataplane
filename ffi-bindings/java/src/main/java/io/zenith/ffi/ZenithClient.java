package io.zenith.ffi;

/**
 * Zenith Data Plane Client for Java
 * Uses JNI to interface with Zenith core library
 */
public class ZenithClient implements AutoCloseable {
    
    static {
        // Load native library
        System.loadLibrary("zenith_jni");
    }
    
    private long enginePtr;
    private boolean closed = false;
    
    /**
     * Create a new Zenith client
     * @param bufferSize Ring buffer size
     * @throws ZenithException if initialization fails
     */
    public ZenithClient(int bufferSize) throws ZenithException {
        this.enginePtr = nativeInit(bufferSize);
        if (this.enginePtr == 0) {
            throw new ZenithException(-1, "Failed to initialize Zenith engine");
        }
    }
    
    /**
     * Load a WASM plugin from file
     * @param filepath Path to .wasm file
     * @throws ZenithException if loading fails
     */
    public void loadPlugin(String filepath) throws ZenithException {
        checkClosed();
        
        byte[] wasmBytes = readFile(filepath);
        int ret = nativeLoadPlugin(enginePtr, wasmBytes);
        
        if (ret != 0) {
            throw new ZenithException(ret, "Failed to load plugin: " + filepath);
        }
    }
    
    /**
     * Get engine statistics
     * @return Stats object
     * @throws ZenithException if operation fails
     */
    public Stats getStats() throws ZenithException {
        checkClosed();
        return nativeGetStats(enginePtr);
    }
    
    /**
     * Close and free engine resources
     */
    @Override
    public void close() {
        if (!closed && enginePtr != 0) {
            nativeFree(enginePtr);
            closed = true;
            enginePtr = 0;
        }
    }
    
    private void checkClosed() throws ZenithException {
        if (closed) {
            throw new ZenithException(-1, "Client is closed");
        }
    }
    
    private byte[] readFile(String filepath) throws ZenithException {
        try {
            return java.nio.file.Files.readAllBytes(
                java.nio.file.Paths.get(filepath)
            );
        } catch (java.io.IOException e) {
            throw new ZenithException(-4, "Failed to read file: " + e.getMessage());
        }
    }
    
    // Native methods
    private static native long nativeInit(int bufferSize);
    private static native void nativeFree(long enginePtr);
    private static native int nativeLoadPlugin(long enginePtr, byte[] wasmBytes);
    private static native Stats nativeGetStats(long enginePtr);
    
    /**
     * Statistics container
     */
    public static class Stats {
        public final long bufferLen;
        public final long pluginCount;
        public final long eventsProcessed;
        
        public Stats(long bufferLen, long pluginCount, long eventsProcessed) {
            this.bufferLen = bufferLen;
            this.pluginCount = pluginCount;
            this.eventsProcessed = eventsProcessed;
        }
        
        @Override
        public String toString() {
            return String.format("Stats{bufferLen=%d, pluginCount=%d, eventsProcessed=%d}",
                bufferLen, pluginCount, eventsProcessed);
        }
    }
}
