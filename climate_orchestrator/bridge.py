"""
climate_orchestrator/bridge.py — Puente de ejecución de acciones
Ejecuta comandos vía Rust bridge (photonic-bridge extendido) o fallback nativo.
"""
from __future__ import annotations

import json
import logging
import os
import subprocess
from dataclasses import asdict
from typing import Any, Dict, Optional

from .actions import Action

log = logging.getLogger("qeos.climate.bridge")

class ClimateBridge:
    """
    Ejecuta acciones de optimización energética.
    En producción usa el photonic-bridge Rust; en simulación solo log.
    """

    def __init__(self, bridge_path: Optional[str] = None, dry_run: bool = True):
        self.bridge_path = bridge_path or os.getenv(
            "PHOTONIC_BRIDGE_PATH",
            os.path.join(os.path.dirname(__file__), "..", "photonic-bridge", "target", "release", "photonic-bridge.exe" if os.name == "nt" else "photonic-bridge")
        )
        self.dry_run = dry_run
        self._available = self._check_bridge()

    def _check_bridge(self) -> bool:
        """Verifica si el bridge Rust está compilado y disponible."""
        if self.dry_run:
            return False
        try:
            result = subprocess.run([self.bridge_path, "--version"], capture_output=True, text=True, timeout=5)
            return result.returncode == 0
        except (FileNotFoundError, subprocess.TimeoutExpired):
            log.warning(f"Photonic bridge no disponible en {self.bridge_path}")
            return False

    def execute(self, action: Action, dry_run: Optional[bool] = None) -> Dict[str, Any]:
        """Ejecuta una acción y devuelve el resultado."""
        dry = dry_run if dry_run is not None else self.dry_run

        if dry:
            msg = f"[DRY RUN] {action.name}: {action.description}"
            log.info(msg)
            return {"success": True, "output": msg, "dry_run": True}

        log.info(f"Ejecutando acción: {action.id}")

        # Intentar por bridge Rust primero
        if self._available:
            try:
                payload = json.dumps(asdict(action))
                result = subprocess.run(
                    [self.bridge_path, "execute", "--json"],
                    input=payload,
                    capture_output=True,
                    text=True,
                    timeout=30,
                )
                if result.returncode == 0:
                    output = json.loads(result.stdout)
                    return {"success": output.get("success", True), "output": output.get("output", result.stdout)}
                else:
                    log.error(f"Bridge error: {result.stderr}")
            except Exception as e:
                log.warning(f"Bridge falló, cayendo a fallback: {e}")

        # Fallback nativo Python
        return self._execute_native(action)

    def _execute_native(self, action: Action) -> Dict[str, Any]:
        """Ejecución nativa (sin Rust). Implementaciones seguras."""
        try:
            if action.id == "limit_cpu_frequency":
                return self._limit_cpu_frequency()
            elif action.id == "shutdown_non_essential":
                return self._shutdown_non_essential()
            elif action.id == "enable_power_saving":
                return self._enable_power_saving()
            elif action.id == "shed_load":
                return self._shed_load()
            elif action.id == "enter_emergency_mode":
                return self._enter_emergency_mode()
            else:
                return {"success": False, "output": f"Acción no implementada en fallback: {action.id}"}
        except Exception as e:
            log.exception(f"Error ejecutando {action.id}: {e}")
            return {"success": False, "output": str(e)}

    # ── Implementaciones nativas de acciones ─────────────────────────────────

    def _limit_cpu_frequency(self) -> Dict[str, Any]:
        """Limita la frecuencia del CPU a un máximo conservador."""
        try:
            if os.name == "nt":
                # Windows: usar powercfg
                subprocess.run(["powercfg", "-setactive", "a1841308-3541-4fab-bc81-f71556f20b4a"], check=True, timeout=10)
                return {"success": True, "output": "Power plan cambiado a 'Ahorro de energía' (Windows)"}
            else:
                # Linux: requiere cpufrequtils o similar
                for governor in ["conservative", "powersave"]:
                    cmd = f"echo {governor} | sudo tee /sys/devices/system/cpu/cpu*/cpufreq/scaling_governor 2>/dev/null"
                    result = subprocess.run(cmd, shell=True, timeout=5, capture_output=True, text=True)
                    if result.returncode == 0:
                        return {"success": True, "output": f"Governor CPU establecido a {governor}"}
                return {"success": False, "output": "No se pudo cambiar governor CPU (requiere sudo)"}
        except subprocess.CalledProcessError as e:
            return {"success": False, "output": f"Error al limitar CPU: {e}"}

    def _shutdown_non_essential(self) -> Dict[str, Any]:
        """Apaga servicios no esenciales (ej. logs, desarrollo)."""
        services = ["nginx", "apache2", "docker", "redis-server"]  # lista común
        stopped = []
        for svc in services:
            try:
                subprocess.run(["sudo", "systemctl", "stop", svc], timeout=10, capture_output=True)
                stopped.append(svc)
            except (FileNotFoundError, subprocess.CalledProcessError):
                continue  # servicio no existe, no es crítico
        return {"success": True, "output": f"Servicios detenidos: {', '.join(stopped) if stopped else 'ninguno (ya estaban inactivos)'}"}

    def _enable_power_saving(self) -> Dict[str, Any]:
        """Activa herramientas de ahorro de energía."""
        try:
            # Intentar powertop (Linux)
            subprocess.run(["sudo", "powertop", "--auto-tune"], timeout=15, capture_output=True)
            return {"success": True, "output": "PowerTOP auto-tune aplicado"}
        except (FileNotFoundError, subprocess.CalledProcessError):
            return {"success": False, "output": "powertop no disponible o failed"}

    def _shed_load(self) -> Dict[str, Any]:
        """Desconecta cargas no críticas (MQTT, GPIO, etc)."""
        # En un sistema real: publicar mensaje MQTT a topic "grid/shed"
        # o invocar script que desconecta relays
        return {"success": True, "output": "Cargas no críticas desconectadas (simulado)"}

    def _enter_emergency_mode(self) -> Dict[str, Any]:
        """Modo emergencia: solo procesos críticos."""
        try:
            subprocess.run(["sudo", "systemctl", "isolate", "emergency.target"], timeout=10)
            return {"success": True, "output": "Sistema en modo emergencia (emergency.target)"}
        except Exception as e:
            return {"success": False, "output": f"Fallo modo emergencia: {e}"}
