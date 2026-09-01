# proxy.py
from mitmproxy import http

def request(flow: http.HTTPFlow) -> None:
    # 通信内容をログに記録
    print(f"URL: {flow.request.pretty_url}")
    print(f"Method: {flow.request.method}")
    print(f"Headers: {flow.request.headers}")
    
    if flow.request.content:
        print(f"Body: {flow.request.content.decode('utf-8', errors='ignore')}")

def response(flow: http.HTTPFlow) -> None:
    print(f"Status: {flow.response.status_code}")
    print(f"Headers: {flow.response.headers}")
    
    if flow.response.content:
        print(f"Body: {flow.response.content.decode('utf-8', errors='ignore')}")
