import requests
import time
from datetime import datetime

# ==================== CONFIGURA ESTO ====================
TELEGRAM_TOKEN = "TU_TOKEN"
TELEGRAM_CHAT_ID = "TU_CHAT_ID"
WHATSAPP_PHONE = "521XXXXXXXXXX"

# URL de tu QuantumEnergyOS (cámbiala por la tuya)
QUANTUM_API = "http://localhost:8080/api/alert"   # o la IP de tu servidor
# ======================================================

def enviar_a_quantum(data):
    try:
        requests.post(QUANTUM_API, json=data, timeout=5)
        print("Alerta enviada a QuantumEnergyOS")
    except:
        print("QuantumEnergyOS no respondió (¿está corriendo?)")

def detectar_apagon():
    try:
        r = requests.get("https://downdetector.com.mx/status/cfe/", timeout=10)
        if any(p in r.text.lower() for p in ["apagón", "sin luz", "mexicali"]):
            alerta = {
                "timestamp": datetime.now().isoformat(),
                "type": "power_outage",
                "location": "Mexicali",
                "source": "downdetector",
                "severity": "high"
            }
            
            enviar_a_quantum(alerta)
            return True
    except:
        pass
    return False

print("Detector conectado a QuantumEnergyOS - Iniciado")
while True:
    if detectar_apagon():
        print("¡Apagón detectado y enviado a tu OS!")
    time.sleep(180)
