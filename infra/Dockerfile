FROM rust:1.75 as builder
WORKDIR /app
COPY . .
RUN cd core && cargo build --release

FROM debian:bookworm-slim
COPY --from=builder /app/core/target/release/libzenith_core.so /usr/local/lib/
# Copy the binary if we had a main.rs entry point. 
# For now we assume this container might run a wrapper or the engine binary if we added one.
CMD ["echo", "Zenith Core built successfully"]
