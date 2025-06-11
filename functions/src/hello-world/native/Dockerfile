FROM docker.io/library/rust:1.82.0-bullseye as builder
ARG TARGETARCH=x86_64
ARG FUNCTION_PATH
RUN apt-get update && apt-get clean
#    apt-get install -y \
#    libfdt-dev \
#    build-essential \
#    musl-tools \
RUN rustup target add ${TARGETARCH}-unknown-linux-musl
RUN mkdir /build
COPY ${FUNCTION_PATH}/. /build/
COPY functions/src/shared_lib/. /build/shared_lib
RUN ls -lah /build
WORKDIR /build
#ENV RUSTFLAGS="-C link_arg=-lgcc -C link_arg=-specs -C link_arg=/usr/lib/${TARGETARCH}-linux-musl/musl-gcc.specs" 
# --verbose
RUN cargo build --release --target ${TARGETARCH}-unknown-linux-musl
RUN ls -lah
RUN chmod +x target/${TARGETARCH}-unknown-linux-musl/release/hello-world-native

FROM docker.io/library/debian:bullseye-slim
ARG TARGETARCH=x86_64
COPY --from=builder /build/target/${TARGETARCH}-unknown-linux-musl/release/hello-world-native /
ENTRYPOINT ["/hello-world-native"]
