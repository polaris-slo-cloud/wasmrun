FROM docker.io/library/rust:1.80.0-bullseye as builder
ARG TARGETARCH=x86_64

RUN apt-get update && apt-get clean
RUN rustup target add ${TARGETARCH}-unknown-linux-musl
RUN mkdir /build
COPY /. /build/
RUN ls -lah /build
WORKDIR /build
RUN cargo build --release --target ${TARGETARCH}-unknown-linux-musl
RUN ls -lah
RUN chmod +x target/${TARGETARCH}-unknown-linux-musl/release/test-redis-native

FROM scratch
ARG TARGETARCH=x86_64
COPY --from=builder /build/target/${TARGETARCH}-unknown-linux-musl/release/test-redis-native /
ENTRYPOINT ["/test-redis-native"]
