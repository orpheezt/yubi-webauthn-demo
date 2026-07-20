# syntax=docker/dockerfile:1
ARG RUST_VERSION=1.85.0
ARG ALPINE_VERSION=3.21.0

FROM rust:${RUST_VERSION}-alpine AS build
WORKDIR /app
RUN apk add --no-cache musl-dev libssl-dev pkgconfig openssl-dev clang lld

RUN cargo install sqlx-cli --no-default-features --features rustls,postgres

FROM alpine:${ALPINE_VERSION} AS final

RUN apk add --no-cache ca-certificates

WORKDIR /app
COPY --from=build /usr/local/cargo/bin/sqlx /usr/local/bin/sqlx
COPY migrations ./migrations

CMD ["sqlx", "migrate", "run"]
