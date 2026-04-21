import subprocess
import sys
import time

proc = subprocess.Popen(
    [sys.executable, r"C:\Users\HP\AppData\Local\Programs\Microsoft VS Code\QuantumEnergyOS-V.02\server.py"],
    stdout=subprocess.PIPE,
    stderr=subprocess.STDOUT,
    creationflags=subprocess.CREATE_NEW_PROCESS_GROUP if sys.platform == "win32" else 0
)

time.sleep(4)

print("="*60)
print("QuantumEnergyOS V.02 - WebSocket Test")
print("="*60)

try:
    import socketio
    
    sio = socketio.Client()
    received_events = []
    
    @sio.on('connected')
    def on_connected(data):
        received_events.append('connected')
    
    @sio.on('subscribed')
    def on_subscribed(data):
        received_events.append('subscribed')
    
    @sio.on('grid_update')
    def on_grid_update(data):
        received_events.append('grid_update')
    
    @sio.on('dashboard_update')
    def on_dashboard_update(data):
        received_events.append('dashboard_update')
    
    sio.connect('http://127.0.0.1:8000')
    sio.emit('subscribe_grid')
    time.sleep(0.5)
    sio.emit('request_dashboard')
    time.sleep(2.5)
    sio.disconnect()
    
    print(f"Events received: {received_events}")
    print("WebSocket test PASSED!" if len(received_events) >= 3 else "Some events missing")

except Exception as e:
    print(f"WebSocket test error: {e}")

proc.terminate()
proc.wait()