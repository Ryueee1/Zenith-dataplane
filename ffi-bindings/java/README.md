# Zenith Java Bindings

Java JNI bindings for the Zenith Data Plane.

## Prerequisites

- Java 11 or higher
- Maven 3.6+
- Zenith core library built

Build core:
```bash
cd ../../core
cargo build --release
```

## Building

```bash
mvn clean install
```

## Usage

```java
import io.zenith.ffi.ZenithClient;
import io.zenith.ffi.ZenithException;

public class Example {
    public static void main(String[] args) {
        // Using try-with-resources (AutoCloseable)
        try (ZenithClient client = new ZenithClient(1024)) {
            
            // Load plugin
            client.loadPlugin("filter.wasm");
            
            // Get stats
            ZenithClient.Stats stats = client.getStats();
            System.out.println("Plugins: " + stats.pluginCount);
            
        } catch (ZenithException e) {
            System.err.println("Error: " + e);
        }
    }
}
```

## API Documentation

### `ZenithClient(int bufferSize)`

Creates a new Zenith client instance.

**Throws:** `ZenithException`

### `void loadPlugin(String filepath)`

Loads a WASM plugin from file.

**Throws:** `ZenithException`

### `Stats getStats()`

Returns engine statistics.

### `void close()`

Frees engine resources. Called automatically with try-with-resources.

## Error Handling

```java
try {
    client.loadPlugin("plugin.wasm");
} catch (ZenithException e) {
    int code = e.getErrorCode();
    // Handle error
}
```

## Error Codes

- `0`: Success
- `-1`: Null pointer / Client closed
- `-2`: Buffer full
- `-3`: Plugin load error
- `-4`: FFI conversion error

## Running Tests

```bash
mvn test
```

## JNI Library

The JNI wrapper (`libzenith_jni.so`) must be built separately and placed in `java.library.path`.

**Note:** Full JNI implementation requires additional C code to bridge Java<->Rust. This is the Java-side skeleton.

## License

Apache 2.0
