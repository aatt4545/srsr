
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
    curl \
    wget \
    nodejs \
    npm \
    lua5.4 \
    lua5.1 \
    python3 \
    python3-pip \
    php \
    php-cli \
    ruby \
    perl \
    gcc \
    g++ \
    default-jdk \
    default-jre \
    sqlite3 \
    bash \
    && rm -rf /var/lib/apt/lists/*

# Luauをビルド
RUN git clone https://github.com/luau-lang/luau /tmp/luau && \
    cd /tmp/luau && \
    make luau && \
    cp luau /usr/local/bin/luau && \
    rm -rf /tmp/luau

# Goをインストール
RUN curl -fsSL https://go.dev/dl/go1.21.0.linux-amd64.tar.gz | tar -xz -C /usr/local
ENV PATH="/usr/local/go/bin:${PATH}"

# Rustをインストール
RUN curl -fsSL https://sh.rustup.rs | sh -s -- -y
ENV PATH="/root/.cargo/bin:${PATH}"

# PowerShellをインストール
RUN curl -fsSL https://packages.microsoft.com/config/debian/12/packages-microsoft-prod.deb -o /tmp/packages-microsoft-prod.deb && \
    dpkg -i /tmp/packages-microsoft-prod.deb && \
    apt-get update && \
    apt-get install -y powershell && \
    rm -rf /var/lib/apt/lists/* /tmp/packages-microsoft-prod.deb

# Kotlinをインストール
RUN curl -fsSL https://github.com/JetBrains/kotlin/releases/download/v1.9.0/kotlin-compiler-1.9.0.zip -o /tmp/kotlin.zip && \
    unzip /tmp/kotlin.zip -d /usr/local/kotlin && \
    rm /tmp/kotlin.zip
ENV PATH="/usr/local/kotlin/kotlinc/bin:${PATH}"

# SwiftはLinuxでは制限があるためスキップ
# C# (.NET)をインストール
RUN curl -fsSL https://dot.net/v1/dotnet-install.sh | bash -s -- -c 8.0 --install-dir /usr/local/dotnet
ENV PATH="/usr/local/dotnet:${PATH}"

WORKDIR /app
COPY --from=go-builder /app/deobfuscator-server .
COPY --from=rust-builder /app/target/release/libdeobfuscator.so /usr/lib/libdeobfuscator.so
COPY index.html .

EXPOSE 8080

CMD ["./deobfuscator-server"]
