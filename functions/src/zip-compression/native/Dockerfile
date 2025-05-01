FROM docker.io/library/rust:1.80.0-slim-bookworm as builder
ARG TARGETARCH=x86_64
ARG FUNCTION_PATH
RUN apt-get update && apt-get clean
RUN apt-get install -y musl-tools
RUN rustup target add ${TARGETARCH}-unknown-linux-musl
RUN mkdir /build
COPY ${FUNCTION_PATH}/. /build/
COPY functions/src/shared_lib/. /build/shared_lib
RUN ls -lah /build
WORKDIR /build
RUN cargo build --release --target ${TARGETARCH}-unknown-linux-musl
RUN ls -lah
RUN chmod +x target/${TARGETARCH}-unknown-linux-musl/release/zip-compression-native

FROM docker.io/library/rust:1.80.0-bullseye
ARG TARGETARCH=x86_64
COPY --from=builder /build/target/${TARGETARCH}-unknown-linux-musl/release/zip-compression-native /
ENTRYPOINT ["/zip-compression-native"]
