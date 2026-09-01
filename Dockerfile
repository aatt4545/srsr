
# Dockerfile
FROM rust:1.75 AS rust-builder

WORKDIR /app
COPY Cargo.toml .
COPY src ./src
RUN cargo build --release

FROM golang:1.21 AS go-builder

WORKDIR /app
COPY go.mod ./
COPY main.go ./

COPY --from=rust-builder /app/target/release/libdeobfuscator.so ./target/release/

RUN go mod tidy
RUN go mod download
RUN go build -o deobfuscator-server

FROM debian:bookworm-slim

RUN apt-get update && apt-get install -y \
    ca-certificates \
    git \
    build-essential \
    cmake \
    nodejs \
    npm \
    lua5.4 \
    lua5.1 \
    python3 \
    && rm -rf /var/lib/apt/lists/*

RUN git clone https://github.com/luau-lang/luau /tmp/luau && \
    cd /tmp/luau && \
    make luau && \
    cp luau /usr/local/bin/luau && \
    rm -rf /tmp/luau

WORKDIR /app
COPY --from=go-builder /app/deobfuscator-server .
COPY --from=rust-builder /app/target/release/libdeobfuscator.so /usr/lib/libdeobfuscator.so
COPY index.html .

EXPOSE 8080

CMD ["./deobfuscator-server"]
