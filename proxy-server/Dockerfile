FROM python:3.11-slim

RUN pip install mitmproxy

WORKDIR /app
COPY proxy.py .

EXPOSE 8080

CMD ["mitmdump", "-s", "proxy.py", "--listen-port", "8080"]
