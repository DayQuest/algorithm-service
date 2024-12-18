FROM rust:1.70 as builder

WORKDIR /usr/src/myapp

COPY Cargo.toml Cargo.lock ./

RUN cargo build --release && cargo install --path .

COPY . .

RUN cargo build --release

FROM debian:bullseye-slim

RUN apt-get update && apt-get install -y \
    libc6 \
    libssl-dev \
    && rm -rf /var/lib/apt/lists/*

COPY --from=builder /usr/src/myapp/target/release/myapp /usr/local/bin/myapp

EXPOSE 8020

CMD ["myapp"]
