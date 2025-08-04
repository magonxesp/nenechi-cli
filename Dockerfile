FROM rust:1.88 AS build

WORKDIR /builder

COPY . .

RUN cargo build --release

FROM debian:bookworm-slim

WORKDIR /app

RUN apt update && apt install -y libsqlite3-0 libssl3

COPY --from=build /builder/target/release/nenechi-cli .
RUN chmod +x /app/nenechi-cli

ENTRYPOINT ["/app/nenechi-cli"]

CMD ["-h"]
