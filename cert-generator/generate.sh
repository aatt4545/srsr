#!/bin/bash

# 自己署名CA証明書を生成
openssl genrsa -out ca.key 2048

openssl req -new -x509 \
    -key ca.key \
    -out ca.crt \
    -days 3650 \
    -subj "/CN=Security CA" \
    -nodes

# base64にエンコード
base64 ca.crt > ca_base64.txt

echo "=== 生成完了 ==="
echo "ca.key: 秘密鍵"
echo "ca.crt: 証明書"
echo "ca_base64.txt: mobileconfigに埋め込むbase64"
