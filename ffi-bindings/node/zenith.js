/**
 * Zenith Node.js FFI Bindings
 * High-performance data plane client for Node.js
 */

const ffi = require('ffi-napi');
const ref = require('ref-napi');
const fs = require('fs');
const path = require('path');

const voidPtr = ref.refType(ref.types.void);
const uint8Ptr = ref.refType(ref.types.uint8);
const uint32 = ref.types.uint32;
const uint64 = ref.types.uint64;
const int32 = ref.types.int32;
const size_t = ref.types.size_t;

/**
 * Engine statistics structure
 */
const StatsStruct = ref.types.void; // Simplified for MVP

/**
 * Zenith error codes
 */
const ErrorCodes = {
    OK: 0,
    NULL_PTR: -1,
    BUFFER_FULL: -2,
    PLUGIN_LOAD: -3,
    FFI_ERROR: -4
};

/**
 * ZenithError class
 */
class ZenithError extends Error {
    constructor(code, message) {
        super(message || `Zenith error: ${code}`);
        this.code = code;
        this.name = 'ZenithError';
    }
}

/**
 * ZenithClient class
 */
class ZenithClient {
    /**
     * Create a new Zenith client
     * @param {number} bufferSize - Ring buffer size (default: 1024)
     * @param {string} libPath - Path to libzenith_core.so (auto-detected if not provided)
     */
    constructor(bufferSize = 1024, libPath = null) {
        if (!libPath) {
            const basePath = path.join(__dirname, '..', '..', 'core', 'target', 'release');
            libPath = path.join(basePath, 'libzenith_core.so');
        }

        this.lib = ffi.Library(libPath, {
            'zenith_init': [voidPtr, [uint32]],
            'zenith_free': ['void', [voidPtr]],
            'zenith_load_plugin': [int32, [voidPtr, uint8Ptr, size_t]],
            'zenith_get_stats': [int32, [voidPtr, voidPtr]]
        });

        this.enginePtr = this.lib.zenith_init(bufferSize);

        if (this.enginePtr.isNull()) {
            throw new ZenithError(ErrorCodes.NULL_PTR, 'Failed to initialize Zenith engine');
        }

        this.closed = false;
    }

    /**
     * Load a WASM plugin from file
     * @param {string} filepath - Path to .wasm file
     * @throws {ZenithError}
     */
    loadPlugin(filepath) {
        if (this.closed) {
            throw new ZenithError(ErrorCodes.NULL_PTR, 'Client is closed');
        }

        const wasmBytes = fs.readFileSync(filepath);
        if (!wasmBytes || wasmBytes.length === 0) {
            throw new Error('Empty WASM file');
        }

        const buffer = Buffer.from(wasmBytes);
        const ret = this.lib.zenith_load_plugin(
            this.enginePtr,
            buffer,
            buffer.length
        );

        if (ret !== ErrorCodes.OK) {
            throw new ZenithError(ret, `Failed to load plugin: ${filepath}`);
        }
    }

    /**
     * Get engine statistics
     * @returns {Object} Stats object
     */
    getStats() {
        if (this.closed) {
            throw new ZenithError(ErrorCodes.NULL_PTR, 'Client is closed');
        }

        // For MVP, return placeholder
        // In full implementation, would parse C struct
        return {
            bufferLen: 0,
            pluginCount: 0,
            eventsProcessed: 0
        };
    }

    /**
     * Close and free engine resources
     */
    close() {
        if (!this.closed && !this.enginePtr.isNull()) {
            this.lib.zenith_free(this.enginePtr);
            this.closed = true;
        }
    }

    /**
     * Auto-cleanup on garbage collection
     */
    [Symbol.dispose || Symbol.for('nodejs.dispose')]() {
        this.close();
    }
}

module.exports = {
    ZenithClient,
    ZenithError,
    ErrorCodes
};
