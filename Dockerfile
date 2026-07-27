FROM rust:1.97 AS build

WORKDIR /builder

COPY . .

RUN cargo build --release

FROM debian:trixie-slim

WORKDIR /app

RUN apt update && apt install -y \
    libsqlite3-0 \
    libssl3 \
    ca-certificates \
    && update-ca-certificates

ENV SSL_CERT_FILE=/etc/ssl/certs/ca-certificates.crt
ENV SSL_CERT_DIR=/etc/ssl/certs
ENV REQUESTS_CA_BUNDLE=/etc/ssl/certs/ca-certificates.crt
ENV RUST_BACKTRACE=0
ENV RUST_LOG=info

COPY --from=build /builder/target/release/nenechi-cli .
RUN chmod +x /app/nenechi-cli

ENTRYPOINT ["/app/nenechi-cli"]

CMD ["-h"]
