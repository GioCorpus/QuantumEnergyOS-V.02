#!/usr/bin/env python3
"""
═══════════════════════════════════════════════════════════════════════
 QuantumEnergyOS — Energy Predictor Daemon
 Provides real-time energy status via Unix socket
 
 Socket: /var/run/quantum-energy.sock
 Protocol: Simple text protocol (percentage as string)
 
 Desde Mexicali, BC — para el mundo. Kardashev 0→1
═══════════════════════════════════════════════════════════════════════
"""

import socket
import os
import sys
import time
import json
import logging
from pathlib import Path
from datetime import datetime

# ── Configuration ────────────────────────────────────────────────────
SOCKET_PATH = "/var/run/quantum-energy.sock"
PID_FILE = "/var/run/quantum-energy.pid"
LOG_FILE = "/var/log/quantum-energy.log"
UPDATE_INTERVAL = 5  # seconds

# ── Logging Setup ────────────────────────────────────────────────────
logging.basicConfig(
    level=logging.INFO,
    format='%(asctime)s [%(levelname)s] %(message)s',
    handlers=[
        logging.FileHandler(LOG_FILE),
        logging.StreamHandler(sys.stdout)
    ]
)
logger = logging.getLogger("quantum-energy")

# ── Energy Predictor Class ──────────────────────────────────────────
class EnergyPredictor:
    """
    Simulates energy prediction for QuantumEnergyOS.
    
    In production, this would integrate with:
    - CFE (Comisión Federal de Electricidad) API
    - Local solar panel sensors
    - Battery storage systems
    - Grid load data
    """
    
    def __init__(self):
        self.base_energy = 75  # Base energy percentage
        self.variation = 10    # Random variation
        self.blackout_risk = 0 # Blackout risk factor
        self.last_update = time.time()
        
    def get_energy_status(self):
        """Get current energy status as percentage."""
        # Simulate energy fluctuations
        current_time = time.time()
        elapsed = current_time - self.last_update
        
        # Time-based variation (simulates daily cycles)
        hour = datetime.now().hour
        if 6 <= hour <= 18:
            # Daytime: higher energy (solar)
            time_factor = 1.1
        else:
            # Nighttime: lower energy
            time_factor = 0.9
        
        # Random variation
        import random
        random_factor = 1.0 + (random.random() - 0.5) * 0.1
        
        # Calculate energy percentage
        energy = self.base_energy * time_factor * random_factor
        
        # Apply blackout risk
        if self.blackout_risk > 0:
            energy *= (1.0 - self.blackout_risk * 0.5)
            self.blackout_risk = max(0, self.blackout_risk - 0.1)
        
        # Clamp to 0-100
        energy = max(0, min(100, energy))
        
        self.last_update = current_time
        return int(energy)
    
    def simulate_blackout(self, risk_level=0.5):
        """Simulate a blackout risk event."""
        self.blackout_risk = risk_level
        logger.warning(f"Blackout risk simulated: {risk_level * 100}%")
    
    def get_detailed_status(self):
        """Get detailed energy status as JSON."""
        energy = self.get_energy_status()
        return {
            "energy_percent": energy,
            "status": "critical" if energy < 30 else "warning" if energy < 70 else "normal",
            "timestamp": datetime.now().isoformat(),
            "blackout_risk": self.blackout_risk,
            "solar_available": 6 <= datetime.now().hour <= 18,
            "grid_load": "high" if energy < 50 else "normal"
        }

# ── Socket Server ────────────────────────────────────────────────────
class EnergySocketServer:
    """Unix socket server for energy status queries."""
    
    def __init__(self, socket_path, predictor):
        self.socket_path = socket_path
        self.predictor = predictor
        self.server_socket = None
        self.running = False
        
    def start(self):
        """Start the socket server."""
        # Remove existing socket
        if os.path.exists(self.socket_path):
            os.unlink(self.socket_path)
        
        # Create socket directory
        socket_dir = os.path.dirname(self.socket_path)
        os.makedirs(socket_dir, exist_ok=True)
        
        # Create Unix socket
        self.server_socket = socket.socket(socket.AF_UNIX, socket.SOCK_STREAM)
        self.server_socket.bind(self.socket_path)
        self.server_socket.listen(5)
        
        # Set permissions
        os.chmod(self.socket_path, 0o666)
        
        logger.info(f"Energy socket server started: {self.socket_path}")
        self.running = True
        
        # Write PID file
        with open(PID_FILE, 'w') as f:
            f.write(str(os.getpid()))
        
        try:
            while self.running:
                # Accept connections
                self.server_socket.settimeout(1.0)
                try:
                    client_socket, _ = self.server_socket.accept()
                    self.handle_client(client_socket)
                except socket.timeout:
                    continue
        except KeyboardInterrupt:
            logger.info("Shutting down...")
        finally:
            self.stop()
    
    def handle_client(self, client_socket):
        """Handle client connection."""
        try:
            # Read request
            data = client_socket.recv(1024).decode('utf-8').strip()
            
            if data == "GET_ENERGY":
                # Send energy percentage
                energy = self.predictor.get_energy_status()
                response = str(energy)
                client_socket.send(response.encode('utf-8'))
                logger.debug(f"Sent energy status: {energy}%")
                
            elif data == "GET_STATUS":
                # Send detailed status as JSON
                status = self.predictor.get_detailed_status()
                response = json.dumps(status)
                client_socket.send(response.encode('utf-8'))
                logger.debug(f"Sent detailed status: {status}")
                
            elif data.startswith("SIMULATE_BLACKOUT"):
                # Parse risk level
                parts = data.split()
                risk = float(parts[1]) if len(parts) > 1 else 0.5
                self.predictor.simulate_blackout(risk)
                client_socket.send(b"OK")
                logger.info(f"Blackout simulation triggered: {risk * 100}%")
                
            else:
                client_socket.send(b"ERROR: Unknown command")
                
        except Exception as e:
            logger.error(f"Error handling client: {e}")
            try:
                client_socket.send(b"ERROR")
            except:
                pass
        finally:
            client_socket.close()
    
    def stop(self):
        """Stop the socket server."""
        self.running = False
        if self.server_socket:
            self.server_socket.close()
        if os.path.exists(self.socket_path):
            os.unlink(self.socket_path)
        if os.path.exists(PID_FILE):
            os.unlink(PID_FILE)
        logger.info("Energy socket server stopped")

# ── Main Entry Point ────────────────────────────────────────────────
def main():
    """Main entry point for energy daemon."""
    logger.info("═══════════════════════════════════════════════════════════════")
    logger.info("  QuantumEnergyOS — Energy Predictor Daemon")
    logger.info("  Socket: /var/run/quantum-energy.sock")
    logger.info("═══════════════════════════════════════════════════════════════")
    
    # Check if already running
    if os.path.exists(PID_FILE):
        with open(PID_FILE, 'r') as f:
            pid = int(f.read().strip())
        try:
            os.kill(pid, 0)
            logger.error(f"Daemon already running (PID: {pid})")
            sys.exit(1)
        except OSError:
            # PID file exists but process is dead
            os.unlink(PID_FILE)
    
    # Create predictor and server
    predictor = EnergyPredictor()
    server = EnergySocketServer(SOCKET_PATH, predictor)
    
    # Start server
    try:
        server.start()
    except Exception as e:
        logger.error(f"Failed to start server: {e}")
        sys.exit(1)

if __name__ == "__main__":
    main()
