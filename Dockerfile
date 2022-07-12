FROM rust:latest as builder
WORKDIR /usr/app
COPY . .
ENV SQLX_OFFLINE true
RUN cargo build --release

FROM gcr.io/distroless/cc
WORKDIR /usr/

COPY --from=builder /usr/app/target/release/app /usr/app

# When `docker run` is executed, launch the binary!
ENTRYPOINT ["./usr/app"]
