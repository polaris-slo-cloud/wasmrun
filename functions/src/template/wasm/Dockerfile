FROM docker.io/library/rust:1.80.0-slim-bookworm as builder
ARG FUNCTION_PATH
RUN apt-get update && \
    apt-get install -y clang netbase && \
    apt-get clean
RUN rustup target add wasm32-wasi
RUN mkdir /build
COPY ${FUNCTION_PATH}/. /build/
COPY functions/src/shared_lib/. /build/shared_lib
RUN ls -lah /build
WORKDIR /build
RUN cargo build --target wasm32-wasi --release
RUN ls -lah
RUN chmod +x target/wasm32-wasi/release/<replace>-wasm.wasm


FROM scratch
COPY --from=builder /build/target/wasm32-wasi/release/<replace>-wasm.wasm /
COPY --from=builder /etc/services /etc/
ENTRYPOINT ["/<replace>-wasm.wasm"]
