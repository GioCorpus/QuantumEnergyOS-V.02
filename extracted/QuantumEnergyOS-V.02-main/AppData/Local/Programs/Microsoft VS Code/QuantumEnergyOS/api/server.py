#!/usr/bin/env python3
"""
QuantumEnergyOS V.02 - Energy API Server
API Server for real-time energy monitoring and quantum simulation
Author: Giovanny Corpus Bernal - Mexicali, BC
"""

from flask import Flask, jsonify, render_template
from flask_cors import CORS
import time
import random
import threading
from datetime import datetime
from dataclasses import dataclass, asdict
from typing import List, Dict, Optional
import math

app = Flask(__name__, template_folder='../web-dashboard/public', static_folder='../web-dashboard/public/static')
CORS(app)

SYSTEM_STATE = {
    'version': 'V.02',
    'uptime': time.time(),
    'status': 'optimal',
    'city': 'Mexicali',
    'mission': 'Never more blackouts'
}

energy_data = {
    'current_load': 450.0,
    'peak_load': 800.0,
    'available_capacity': 600.0,
    'efficiency': 0.92,
    'quantum_boost': 1.15,
    'photonic_efficiency': 0.94,
    'nodes_online': 12,
    'total_nodes': 15,
    'predictions': [],
    'alerts': []
}

quantum_state = {
    'active_qubits': 8,
    'coherence': 0.95,
    'entanglement_pairs': 4,
    'simulation_results': [],
    'energy_prediction': 0.0
}

historical_load = []

def update_energy_metrics():
    global energy_data, historical_load, quantum_state
    
    while True:
        base_load = 400 + (math.sin(time.time() / 300) * 100)
        noise = random.gauss(0, 20)
        current_load = max(200, min(900, base_load + noise))
        
        energy_data['current_load'] = current_load
        energy_data['peak_load'] = max(energy_data['peak_load'], current_load)
        
        utilization = current_load / energy_data['available_capacity']
        energy_data['efficiency'] = 0.92 - (utilization * 0.1) if utilization > 0.8 else 0.92
        
        if utilization > 0.85:
            energy_data['alerts'].append({
                'level': 'warning' if utilization < 0.9 else 'critical',
                'message': f'High load detected: {utilization*100:.1f}%',
                'timestamp': datetime.now().isoformat()
            })
            if len(energy_data['alerts']) > 10:
                energy_data['alerts'].pop(0)
        
        historical_load.append(utilization)
        if len(historical_load) > 100:
            historical_load.pop(0)
        
        if len(historical_load) >= 10:
            avg = sum(historical_load[-10:]) / 10
            trend = historical_load[-1] - historical_load[-10]
            prediction = avg + (trend * 0.3)
            quantum_state['energy_prediction'] = max(0, min(1, prediction)) * energy_data['available_capacity']
        
        quantum_state['coherence'] = 0.95 + (random.gauss(0, 0.02))
        quantum_state['coherence'] = max(0.8, min(1.0, quantum_state['coherence']))
        
        time.sleep(2)

metrics_thread = threading.Thread(target=update_energy_metrics, daemon=True)
metrics_thread.start()

@app.route('/')
def index():
    return render_template('index.html')

@app.route('/api/status')
def get_status():
    uptime = time.time() - SYSTEM_STATE['uptime']
    hours = int(uptime // 3600)
    minutes = int((uptime % 3600) // 60)
    
    return jsonify({
        **SYSTEM_STATE,
        'uptime_seconds': int(uptime),
        'uptime_formatted': f'{hours}h {minutes}m',
        'timestamp': datetime.now().isoformat()
    })

@app.route('/api/energy')
def get_energy():
    return jsonify({
        **energy_data,
        'timestamp': datetime.now().isoformat(),
        'utilization_percent': (energy_data['current_load'] / energy_data['available_capacity']) * 100
    })

@app.route('/api/quantum/simulation')
def quantum_simulation():
    results = []
    
    for i in range(5):
        result = {
            'id': i,
            'qubits': random.randint(4, 16),
            'depth': random.randint(10, 50),
            'fidelity': random.uniform(0.85, 0.99),
            'energy_impact': random.uniform(-50, 100),
            'timestamp': datetime.now().isoformat()
        }
        results.append(result)
    
    quantum_state['simulation_results'] = results
    
    return jsonify({
        'simulations': results,
        'quantum_state': {
            'active_qubits': quantum_state['active_qubits'],
            'coherence': quantum_state['coherence'],
            'entanglement_pairs': quantum_state['entanglement_pairs'],
            'energy_prediction': quantum_state['energy_prediction']
        },
        'timestamp': datetime.now().isoformat()
    })

@app.route('/api/optimize', methods=['POST'])
def optimize():
    optimization = {
        'actions': [],
        'savings_kw': 0.0,
        'risk_reduction': 0.0
    }
    
    utilization = energy_data['current_load'] / energy_data['available_capacity']
    
    if utilization > 0.8:
        optimization['actions'].append({
            'type': 'reduce_load',
            'target_nodes': random.randint(2, 5),
            'expected_savings': random.uniform(20, 50)
        })
        optimization['risk_reduction'] = 0.3
    
    optimization['savings_kw'] = energy_data['current_load'] * 0.05
    optimization['risk_reduction'] += 0.1
    
    return jsonify({
        **optimization,
        'timestamp': datetime.now().isoformat()
    })

@app.route('/api/alerts')
def get_alerts():
    return jsonify({
        'alerts': energy_data['alerts'][-10:],
        'count': len(energy_data['alerts']),
        'timestamp': datetime.now().isoformat()
    })

@app.route('/api/history')
def get_history():
    return jsonify({
        'load_history': historical_load[-50:],
        'sample_interval': 2,
        'timestamp': datetime.now().isoformat()
    })

@app.route('/health')
def health():
    return jsonify({'status': 'healthy', 'service': 'quantum-energy-api'})

if __name__ == '__main__':
    print("╔═══════════════════════════════════════════════════════════╗")
    print("║        QuantumEnergyOS V.02 - Energy API Server            ║")
    print("║        Made in Mexicali with 22 years of grind            ║")
    print("╚═══════════════════════════════════════════════════════════╝")
    print()
    print("Starting API Server on http://localhost:5000")
    print("Endpoints:")
    print("  • GET /api/status     - System status")
    print("  • GET /api/energy    - Energy metrics")
    print("  • GET /api/quantum/simulation - Quantum simulation")
    print("  • GET /api/alerts    - System alerts")
    print("  • GET /api/history   - Historical load data")
    print("  • GET /health        - Health check")
    print()
    
    app.run(host='0.0.0.0', port=5000, debug=False, threaded=True)