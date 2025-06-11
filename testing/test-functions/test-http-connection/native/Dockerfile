FROM docker.io/library/rust:1.80.0-slim-bookworm as builder
RUN apt-get update
RUN rustup target add wasm32-wasi
RUN mkdir /build
COPY /. /build/
RUN ls -lah /build
WORKDIR /build
RUN cargo build --target wasm32-wasi --release
RUN ls -lah
RUN chmod +x target/wasm32-wasi/release/test-http-connection-wasm.wasm

FROM scratch
COPY --from=builder /build/target/wasm32-wasi/release/test-http-connection-wasm.wasm /
COPY --from=builder /etc/services /etc/
ENTRYPOINT ["/test-http-connection-wasm.wasm"]
