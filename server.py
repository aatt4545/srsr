# server.py
import http.server
import os

class Handler(http.server.SimpleHTTPRequestHandler):
    def do_GET(self):
        if self.path == '/':
            self.send_response(200)
            self.send_header('Content-Type', 'text/html')
            self.end_headers()
            html = """
            <html>
            <body>
            <h1>Download Ransomware</h1>
            <a href="/ransomware.exe">Download ransomware.exe</a>
            </body>
            </html>
            """
            self.wfile.write(html.encode())
        elif self.path == '/ransomware.exe':
            if os.path.exists('ransomware.exe'):
                self.send_response(200)
                self.send_header('Content-Type', 'application/octet-stream')
                self.send_header('Content-Disposition', 'attachment; filename=ransomware.exe')
                self.end_headers()
                with open('ransomware.exe', 'rb') as f:
                    self.wfile.write(f.read())
            else:
                self.send_response(404)
                self.end_headers()
        else:
            super().do_GET()

if __name__ == '__main__':
    port = int(os.environ.get('PORT', 8080))
    server = http.server.HTTPServer(('0.0.0.0', port), Handler)
    print(f'Server running on port {port}')
    server.serve_forever()
