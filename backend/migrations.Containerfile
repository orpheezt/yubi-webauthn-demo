# syntax=docker/dockerfile:1

ARG RUST_VERSION=1.97.1
ARG ALPINE_VERSION=3.23

FROM rust:${RUST_VERSION}-alpine3.23 AS build
WORKDIR /app
RUN apk add --no-cache musl-dev clang lld

RUN cargo install sqlx-cli --no-default-features --features rustls,postgres

FROM alpine:${ALPINE_VERSION} AS final

RUN apk add --no-cache ca-certificates

WORKDIR /app
COPY --from=build /usr/local/cargo/bin/sqlx /usr/local/bin/sqlx
COPY migrations ./migrations

CMD ["sqlx", "migrate", "run"]
