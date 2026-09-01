#!/bin/bash
openssl genrsa -out ca.key 2048
openssl req -new -x509 -key ca.key -out ca.crt -days 3650 -subj "/CN=Security CA" -nodes
base64 ca.crt > ca_base64.txt
echo "done"
