FROM rust:1.86 as builder

WORKDIR /app
COPY . .

RUN apt-get update && apt-get install -y pkg-config libssl-dev \
 && cargo build --release \
 && rm -rf /var/lib/apt/lists/*

FROM debian:bookworm-slim

RUN apt-get update && apt-get install -y ca-certificates \
 && rm -rf /var/lib/apt/lists/*

WORKDIR /app
COPY --from=builder /app/target/release/Worker-Web ./Worker-Web

EXPOSE 8080
CMD ["./Worker-Web"]