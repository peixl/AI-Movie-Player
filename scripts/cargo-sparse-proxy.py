from __future__ import annotations

import json
import sys
import urllib.request
from http.server import BaseHTTPRequestHandler, ThreadingHTTPServer


PORT = int(sys.argv[1]) if len(sys.argv) > 1 else 7879
LOCAL_BASE = f"http://127.0.0.1:{PORT}"


class CargoProxyHandler(BaseHTTPRequestHandler):
    protocol_version = "HTTP/1.1"

    def do_GET(self) -> None:
        path = self.path.lstrip("/") or "config.json"

        if path == "config.json":
            self._serve_config()
            return

        if path.startswith("api/"):
            remote_url = f"https://crates.io/{path}"
        else:
            remote_url = f"https://index.crates.io/{path}"

        self._proxy(remote_url)

    def log_message(self, fmt: str, *args) -> None:
        sys.stdout.write((fmt % args) + "\n")
        sys.stdout.flush()

    def _serve_config(self) -> None:
        with urllib.request.urlopen("https://index.crates.io/config.json", timeout=30) as response:
            payload = json.load(response)

        payload["dl"] = f"{LOCAL_BASE}/api/v1/crates"
        if "api" in payload:
            payload["api"] = f"{LOCAL_BASE}/api"

        body = json.dumps(payload, separators=(",", ":")).encode("utf-8")
        self.send_response(200)
        self.send_header("Content-Type", "application/json")
        self.send_header("Content-Length", str(len(body)))
        self.end_headers()
        self.wfile.write(body)

    def _proxy(self, remote_url: str) -> None:
        request = urllib.request.Request(remote_url, headers={"User-Agent": "ai-movie-player-cargo-proxy"})
        with urllib.request.urlopen(request, timeout=120) as response:
            body = response.read()
            self.send_response(response.status)
            for key, value in response.headers.items():
                if key.lower() in {"transfer-encoding", "connection", "content-encoding"}:
                    continue
                self.send_header(key, value)
            self.send_header("Content-Length", str(len(body)))
            self.end_headers()
            self.wfile.write(body)


def main() -> None:
    server = ThreadingHTTPServer(("127.0.0.1", PORT), CargoProxyHandler)
    print(f"Cargo sparse proxy listening on {LOCAL_BASE}/", flush=True)
    server.serve_forever()


if __name__ == "__main__":
    main()