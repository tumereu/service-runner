#!/usr/bin/env python3
import http.server
from http import HTTPStatus
import socketserver
import time
import random
from datetime import datetime

def human_time(seconds: float) -> str:
    return f"{seconds:.2f}s"

class Handler(http.server.SimpleHTTPRequestHandler):
    def do_GET(self):
        self.send_response(HTTPStatus.OK)
        self.end_headers()
        self.wfile.write(b'Response body')


def run_startup_sequence():
    steps = [
        "Initializing configuration...",
        "Loading modules...",
        "Establishing network interfaces...",
        "Starting services..."
    ]
    total = 0.0
    for step in steps:
        print(f"{datetime.now().isoformat()} - {step}", flush=True)
        delay = random.uniform(0, 0.5)
        time.sleep(delay)
        total += delay
    print(f"{datetime.now().isoformat()} - Startup complete (took {total:.2f}s)", flush=True)

def serve(port: int):
    socketserver.TCPServer.allow_reuse_address=True
    httpd = socketserver.TCPServer(('', 8000), Handler)
    httpd.serve_forever()

def main():
    run_startup_sequence()
    serve(8000)

if __name__ == "__main__":
    main()
