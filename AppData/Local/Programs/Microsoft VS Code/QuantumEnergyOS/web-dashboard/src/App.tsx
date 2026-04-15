import React, { useState, useEffect } from 'react';
import './App.css';

interface EnergyData {
  current_load: number;
  peak_load: number;
  available_capacity: number;
  efficiency: number;
  nodes_online: number;
  total_nodes: number;
  utilization_percent: number;
}

interface SystemStatus {
  version: string;
  uptime_formatted: string;
  status: string;
  city: string;
  mission: string;
}

interface QuantumState {
  active_qubits: number;
  coherence: number;
  entanglement_pairs: number;
  energy_prediction: number;
}

function App() {
  const [energyData, setEnergyData] = useState<EnergyData | null>(null);
  const [status, setStatus] = useState<SystemStatus | null>(null);
  const [quantumState, setQuantumState] = useState<QuantumState | null>(null);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    fetchData();
    const interval = setInterval(fetchData, 3000);
    return () => clearInterval(interval);
  }, []);

  const fetchData = async () => {
    try {
      const [energyRes, statusRes] = await Promise.all([
        fetch('http://localhost:5000/api/energy'),
        fetch('http://localhost:5000/api/status')
      ]);

      if (!energyRes.ok || !statusRes.ok) {
        throw new Error('API not available');
      }

      const energy = await energyRes.json();
      const sysStatus = await statusRes.json();

      setEnergyData(energy);
      setStatus(sysStatus);
      setLoading(false);
    } catch (err) {
      setError('API Server not running. Start with: python api/server.py');
      setLoading(false);
    }
  };

  if (loading) {
    return (
      <div className="loading-screen">
        <div className="quantum-loader"></div>
        <p>Initializing QuantumEnergyOS...</p>
      </div>
    );
  }

  return (
    <div className="dashboard">
      <header className="dashboard-header">
        <div className="logo-section">
          <h1 className="logo">QuantumEnergyOS</h1>
          <span className="version">V.02</span>
        </div>
        <div className="mission-banner">
          <span>🌵 {status?.city || 'Mexicali'} - {status?.mission || 'Never more blackouts'}</span>
        </div>
      </header>

      {error && (
        <div className="error-banner">
          <span>⚠️ {error}</span>
        </div>
      )}

      <main className="dashboard-content">
        <section className="metrics-grid">
          <div className="metric-card primary">
            <h3>Current Load</h3>
            <div className="metric-value">
              {energyData?.current_load.toFixed(1) || '0'}
              <span className="unit">MW</span>
            </div>
            <div className="metric-bar">
              <div 
                className="bar-fill" 
                style={{ 
                  width: `${energyData?.utilization_percent || 0}%`,
                  backgroundColor: energyData && energyData.utilization_percent > 80 
                    ? '#ef4444' 
                    : '#00d4ff'
                }}
              ></div>
            </div>
            <p className="metric-sub">{energyData?.utilization_percent?.toFixed(1)}% utilization</p>
          </div>

          <div className="metric-card">
            <h3>Peak Load</h3>
            <div className="metric-value">
              {energyData?.peak_load.toFixed(1) || '0'}
              <span className="unit">MW</span>
            </div>
          </div>

          <div className="metric-card">
            <h3>Available Capacity</h3>
            <div className="metric-value">
              {energyData?.available_capacity || 0}
              <span className="unit">MW</span>
            </div>
          </div>

          <div className="metric-card">
            <h3>System Efficiency</h3>
            <div className="metric-value efficiency">
              {((energyData?.efficiency || 0) * 100).toFixed(1)}
              <span className="unit">%</span>
            </div>
          </div>
        </section>

        <section className="quantum-section">
          <h2>⚛️ Quantum State</h2>
          <div className="quantum-grid">
            <div className="quantum-card">
              <h4>Active Qubits</h4>
              <span className="quantum-value">{quantumState?.active_qubits || 8}</span>
            </div>
            <div className="quantum-card">
              <h4>Coherence</h4>
              <span className="quantum-value">{(quantumState?.coherence || 0.95) * 100}%</span>
            </div>
            <div className="quantum-card">
              <h4>Entanglements</h4>
              <span className="quantum-value">{quantumState?.entanglement_pairs || 4}</span>
            </div>
            <div className="quantum-card">
              <h4>Energy Prediction</h4>
              <span className="quantum-value">{quantumState?.energy_prediction?.toFixed(1) || 0} MW</span>
            </div>
          </div>
        </section>

        <section className="status-section">
          <div className="status-card">
            <h3>System Status</h3>
            <div className="status-item">
              <span className="label">Status:</span>
              <span className="value success">{(status?.status || 'optimal').toUpperCase()}</span>
            </div>
            <div className="status-item">
              <span className="label">Uptime:</span>
              <span className="value">{status?.uptime_formatted || '0h 0m'}</span>
            </div>
            <div className="status-item">
              <span className="label">Nodes Online:</span>
              <span className="value">{energyData?.nodes_online || 0} / {energyData?.total_nodes || 0}</span>
            </div>
          </div>
        </section>
      </main>

      <footer className="dashboard-footer">
        <p>Made in Mexicali with 22 years of grind | "El quantum fluye, la energía permanece"</p>
      </footer>
    </div>
  );
}

export default App;