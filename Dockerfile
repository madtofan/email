# ------------------------------------------------------------------------------
# Cargo Build Stage
# ------------------------------------------------------------------------------
FROM rust:1.70.0 as builder

WORKDIR /usr/src/email

# Installing dependencies
RUN apt update && apt upgrade -y
RUN apt install protobuf-compiler -y
RUN apt-get install pkg-config libssl-dev -y

# Copy in the rest of the sources
RUN mkdir -p /usr/src/common
COPY ./common ../common
COPY ./email/ .


# This is the actual application build.
RUN cargo build --release

# ------------------------------------------------------------------------------
# Final Stage
# ------------------------------------------------------------------------------
FROM debian:bullseye-slim AS runtime 

# Copy application binary from builder image
COPY --from=builder /usr/src/email/target/release/email /usr/local/bin

# Run the application
CMD ["/usr/local/bin/email"]
