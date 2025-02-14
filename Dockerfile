FROM rust:latest as builder

WORKDIR /app

COPY Cargo.toml Cargo.lock ./

RUN mkdir -p src && echo "fn main() {}" > src/main.rs

RUN cargo build --release && rm -rf target/release/deps/dayquest_algorithm*

COPY . .

RUN cargo build --release

FROM debian:bookworm-slim

WORKDIR /app

RUN apt-get update && apt-get install -y libssl3 ca-certificates && rm -rf /var/lib/apt/lists/*

COPY --from=builder /app/target/release/dayquest_algorithm /app/dayquest_algorithm

COPY config.json /app/config.json

CMD ["/app/dayquest_algorithm"]
