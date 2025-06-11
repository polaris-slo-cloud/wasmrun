FROM docker.io/library/rust:1.80.0-slim-bookworm as builder
RUN apt-get update && \
    apt-get install -y netbase && apt-get clean
RUN rustup target add wasm32-wasi
ENV RUSTFLAGS="--cfg wasmedge --cfg tokio_unstable"
RUN mkdir /build
COPY . /build/
RUN ls -lah /build
WORKDIR /build
RUN cargo build --target wasm32-wasi --release
RUN ls -lah
RUN chmod +x target/wasm32-wasi/release/client.wasm


FROM scratch
COPY --from=builder /build/target/wasm32-wasi/release/client.wasm /
COPY --from=builder /etc/services /etc/
ENTRYPOINT ["/client.wasm"]
