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

COPY --from=rust-builder /app/target/release/libcode_toolkit.so ./target/release/

RUN go mod tidy
RUN go mod download
RUN go build -o code-toolkit

FROM debian:bookworm-slim

RUN apt-get update && apt-get install -y \
    ca-certificates \
    nodejs \
    lua5.4 \
    lua5.1 \
    python3 \
    php \
    ruby \
    perl \
    bash \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app
COPY --from=go-builder /app/code-toolkit .
COPY --from=rust-builder /app/target/release/libcode_toolkit.so /usr/lib/libcode_toolkit.so

EXPOSE 8080

CMD ["./code-toolkit"]
