# ------------------------------------------------------------------------------
# Cargo Build Stage
# ------------------------------------------------------------------------------
FROM rust:1.70-alpine as cargo-build

WORKDIR /usr/src/email
RUN apk update && \
    apk upgrade
RUN apk add protoc protobuf-dev
RUN apk add build-base
RUN apk add libressl-dev
RUN apk add pkgconfig openssl openssl-dev musl-dev
RUN rustup target add x86_64-unknown-linux-musl
# RUN rustup target add aarch64-unknown-linux-musl
RUN rustup toolchain install stable-aarch64-unknown-linux-musl

COPY . .

RUN cargo build --release --target=x86_64-unknown-linux-musl
RUN cargo install --path .

# ------------------------------------------------------------------------------
# Final Stage
# ------------------------------------------------------------------------------

FROM alpine:latest

COPY --from=cargo-build /usr/local/cargo/bin/email /usr/local/bin/email

CMD ["email"]
