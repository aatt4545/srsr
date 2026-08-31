FROM rust:1.75 AS rust-builder

WORKDIR /app
COPY Cargo.toml .
COPY src ./src
RUN cargo build --release

FROM golang:1.21 AS go-builder

WORKDIR /app
COPY go.mod ./
COPY main.go ./
RUN go mod tidy
RUN go mod download
RUN go build -o deobfuscator-server

FROM debian:bookworm-slim

WORKDIR /app
COPY --from=go-builder /app/deobfuscator-server .
COPY --from=rust-builder /app/target/release/libdeobfuscator.so /app/target/release/
COPY index.html .

EXPOSE 8080

CMD ["./deobfuscator-server"]
