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

-- 1. Configuración y sistema
CREATE TABLE system_config (
    key TEXT PRIMARY KEY,
    value TEXT NOT NULL,
    updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);

-- 2. Nodos de la red eléctrica
CREATE TABLE grid_nodes (
    id INTEGER PRIMARY KEY,
    name TEXT NOT NULL,
    location TEXT,
    capacity_kw REAL NOT NULL,
    latitude REAL,
    longitude REAL,
    is_active BOOLEAN DEFAULT TRUE,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);

-- 3. Mediciones históricas de energía (time-series)
CREATE TABLE energy_measurements (
    id INTEGER PRIMARY KEY,
    timestamp TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    node_id INTEGER,
    load_kw REAL NOT NULL,
    capacity_kw REAL,
    utilization_pct REAL,
    solar_kp_index REAL,
    temperature_c REAL,
    FOREIGN KEY (node_id) REFERENCES grid_nodes(id)
);

CREATE INDEX idx_energy_ts ON energy_measurements(timestamp);
CREATE INDEX idx_energy_node ON energy_measurements(node_id, timestamp);

-- 4. Alertas y eventos
CREATE TABLE alerts (
    id INTEGER PRIMARY KEY,
    timestamp TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    level TEXT CHECK(level IN ('info', 'warning', 'critical', 'extreme')),
    message TEXT NOT NULL,
    node_id INTEGER,
    resolved BOOLEAN DEFAULT FALSE,
    resolved_at TIMESTAMP,
    FOREIGN KEY (node_id) REFERENCES grid_nodes(id)
);

-- 5. Simulaciones cuánticas (historial)
CREATE TABLE quantum_simulations (
    id INTEGER PRIMARY KEY,
    timestamp TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    simulation_type TEXT NOT NULL,   -- 'qaoa_grid', 'vqe_molecular', 'cooling', 'braiding', 'quartz4d'
    parameters JSON NOT NULL,
    result JSON NOT NULL,
    execution_time_ms REAL,
    backend_used TEXT                -- 'qiskit_aer', 'ibm_quantum', 'qsharp', 'photonic'
);

-- 6. Usuarios (para dashboard multiusuario futuro)
CREATE TABLE users (
    id INTEGER PRIMARY KEY,
    username TEXT UNIQUE NOT NULL,
    password_hash TEXT NOT NULL,
    role TEXT CHECK(role IN ('admin', 'operator', 'viewer')),
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);

-- 7. Optimizaciones QAOA aplicadas
CREATE TABLE qaoa_optimizations (
    id INTEGER PRIMARY KEY,
    timestamp TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    grid_state_before JSON,
    grid_state_after JSON,
    ahorro_kw REAL,
    improvement_pct REAL,
    parameters JSON
);

pip install sqlalchemy alembic psycopg2-binary  # o solo sqlite
