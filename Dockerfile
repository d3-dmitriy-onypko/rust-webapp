# syntax=docker/dockerfile:1.4.1
FROM rust:latest as builder

RUN --mount=type=cache,target=/usr/local/cargo/registry \
    cargo install cargo-strip
COPY . /app/
RUN ls -la
ENV SQLX_OFFLINE true

WORKDIR /app

RUN --mount=type=cache,target=/usr/local/cargo/registry \
    cargo build --release && \
    cargo strip 

FROM gcr.io/distroless/cc

COPY --from=builder /app/target/release/app /

ENTRYPOINT ["./app"]
