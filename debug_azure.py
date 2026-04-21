import urllib.request
import json

req = urllib.request.Request(
    'http://127.0.0.1:8000/api/v1/azure/run',
    data=b'{"provider":"ionq"}',
    headers={"Content-Type": "application/json"},
    method="POST"
)

try:
    r = urllib.request.urlopen(req, timeout=10)
    print("Status:", r.status)
    print("Body:", r.read().decode())
except urllib.error.HTTPError as e:
    print("HTTP Error:", e.code)
    print("Body:", e.read().decode()[:500])
except Exception as e:
    print("Error:", type(e).__name__, str(e)[:100])