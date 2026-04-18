import subprocess
import sys
import time
import json
import urllib.request
import urllib.error

proc = subprocess.Popen(
    [sys.executable, r"C:\Users\HP\AppData\Local\Programs\Microsoft VS Code\QuantumEnergyOS-V.02\server.py"],
    stdout=subprocess.PIPE,
    stderr=subprocess.STDOUT,
    creationflags=subprocess.CREATE_NEW_PROCESS_GROUP if sys.platform == "win32" else 0
)

time.sleep(5)

def test(url, method, data):
    try:
        headers = {"Content-Type": "application/json"} if data else {}
        req = urllib.request.Request(url, data=data, headers=headers, method=method)
        r = urllib.request.urlopen(req, timeout=30)
        return r.status, json.loads(r.read().decode())
    except urllib.error.HTTPError as e:
        return e.code, e.read().decode()[:100]
    except Exception as e:
        return "ERR", str(e)[:60]

print("QuantumEnergyOS V.02 - Final Tests")
print("="*60)

base = "http://127.0.0.1:8000"
tests = [
    (f"{base}/", "GET", None),
    (f"{base}/health", "GET", None),
    (f"{base}/api/v1/dashboard", "GET", None),
    (f"{base}/api/v1/grid/balance", "POST", b'{"n_nodos":6}'),
    (f"{base}/api/v1/vqe/molecular", "POST", b'{"molecule":"H2"}'),
    (f"{base}/api/v1/ibm/status", "GET", None),
    (f"{base}/api/v1/azure/status", "GET", None),
    (f"{base}/api/v1/azure/run", "POST", b'{"provider":"ionq"}'),
    (f"{base}/api/v1/solar/forecast", "GET", None),
    (f"{base}/api/v1/quartz/predict", "POST", b'{"hours_ahead":24}'),
]

passed = 0
for url, method, data in tests:
    path = url.replace(base, "")
    status, result = test(url, method, data)
    if status in (200, 201):
        print(f"[PASS] {method} {path}")
        passed += 1
    else:
        print(f"[FAIL] {method} {path}")
        print(f"       {status}: {result}")

print("="*60)
print(f"Result: {passed}/{len(tests)} tests passed")

proc.terminate()
proc.wait()