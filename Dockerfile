FROM debian:bookworm-slim

RUN apt-get update && apt-get install -y ca-certificates && rm -rf /var/lib/apt/lists/*

WORKDIR /app
COPY --from=go-builder /app/deobfuscator-server .
COPY --from=rust-builder /app/target/release/libdeobfuscator.so /usr/lib/libdeobfuscator.so
COPY index.html .

EXPOSE 8080

CMD ["./deobfuscator-server"]
