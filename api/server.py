#!/usr/bin/env python3
"""
QuantumEnergyOS - API Server
Versión: 0.2
"""

from flask import Flask, jsonify, request
import os

app = Flask(__name__)

@app.route('/')
def home():
    return jsonify({
        "status": "online",
        "system": "QuantumEnergyOS V.02",
        "message": "API corriendo correctamente - Mexicali sin apagones"
    })

@app.route('/energy/status')
def energy_status():
    return jsonify({
        "grid_status": "stable",
        "blackout_prevention": "active",
        "temperature": "45°C",
        "location": "Mexicali, Baja California"
    })

if __name__ == '__main__':
    port = int(os.environ.get('PORT', 5000))
    app.run(host='0.0.0.0', port=port, debug=True)
