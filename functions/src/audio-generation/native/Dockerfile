FROM docker.io/library/rust:1.80.0-bullseye as builder
ARG TARGETARCH=x86_64
ARG FUNCTION_PATH
RUN apt-get update && \
    apt-get install -y clang netbase ca-certificates &&  \
    apt-get clean
RUN rustup target add ${TARGETARCH}-unknown-linux-musl
RUN mkdir /build
COPY ${FUNCTION_PATH}/. /build/
COPY functions/src/shared_lib/. /build/shared_lib
RUN ls -lah /build
WORKDIR /build
RUN cargo build --release --target ${TARGETARCH}-unknown-linux-musl
RUN ls -lah
RUN chmod +x target/${TARGETARCH}-unknown-linux-musl/release/audio-generation-native

FROM docker.io/library/rust:1.80.0-bullseye
ARG TARGETARCH=x86_64
COPY --from=builder /build/target/${TARGETARCH}-unknown-linux-musl/release/audio-generation-native /
COPY --from=builder /etc/services /etc/
COPY --from=builder /etc/ssl/certs/ca-certificates.crt /etc/ssl/certs/
ENTRYPOINT ["/audio-generation-native"]