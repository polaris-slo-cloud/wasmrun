FROM docker.io/library/rust:1.80.0-slim-bookworm as builder
ARG FUNCTION_PATH
RUN apt-get update && apt-get clean
RUN rustup target add wasm32-wasi
ENV RUSTFLAGS="--cfg wasmedge --cfg tokio_unstable"
RUN mkdir /build
COPY ${FUNCTION_PATH}/. /build/
COPY functions/src/shared_lib/. /build/shared_lib
RUN ls -lah /build
WORKDIR /build
RUN cargo build --target wasm32-wasi --release
RUN ls -lah
RUN chmod +x target/wasm32-wasi/release/audio-generation-wasm.wasm

FROM scratch
COPY --from=builder /build/target/wasm32-wasi/release/audio-generation-wasm.wasm /
ENTRYPOINT ["/audio-generation-wasm.wasm"]