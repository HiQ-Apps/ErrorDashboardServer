FROM rust:latest AS builder

WORKDIR /app


COPY Cargo.toml Cargo.lock ./

COPY server/Cargo.toml server/
COPY shared_types/Cargo.toml shared_types/
COPY server/migration/Cargo.toml server/migration/

RUN cargo fetch

COPY server/ server/
COPY shared_types/ shared_types/
COPY server/migration/ server/migration/

ARG BUILD_MODE=release

RUN if [ "$BUILD_MODE" = "release" ]; then cargo build --release --workspace; else cargo build --workspace; fi
FROM debian:testing

RUN apt-get update && apt-get install -y \
    openssl \
    libssl-dev \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

ARG BUILD_MODE=release

COPY --from=builder /app/target/${BUILD_MODE}/server /usr/local/bin/server
WORKDIR /usr/local/bin

EXPOSE 8000
CMD ["server"]
