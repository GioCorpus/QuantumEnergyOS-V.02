"""
browser_integration.py — QuantumEnergyOS V.02
══════════════════════════════════════════════════════════════════════
Integración completa de navegadores con el servidor Flask.
Agrega al server.py los endpoints de monitoreo, control y QAOA.

USO — añadir en api/server.py antes del bloque if __name__ == '__main__':

    from browser_integration import integrate_browsers
    integrate_browsers(app)

Autor: Giovanny Anthony Corpus Bernal — Mexicali, BC
"""

from __future__ import annotations

import json
import logging
import math
import os
import subprocess
import sys
import threading
import time
from collections import deque
from dataclasses import asdict, dataclass, field
from datetime import datetime, timezone
from pathlib import Path
from typing import Any, Dict, List, Optional

from flask import Blueprint, Flask, jsonify, request

log = logging.getLogger("qeos.browsers")

# ── Constantes ────────────────────────────────────────────────────────
CPU_SPIKE_THRESHOLD_PCT: float = 8.0
HISTORY_MAX:             int   = 200
QAOA_EVENTS_MAX:         int   = 50
QAOA_WORKER_INTERVAL_S:  float = 5.0
API_PORT:                int   = int(os.environ.get("PORT", 8000))

# Perfiles de navegador definidos en install_browsers.sh
BROWSER_PROFILES = {
    "energy-grid": {
        "name":        "EnergyGrid",
        "browser":     "firefox",
        "description": "Dashboards de red eléctrica en tiempo real",
        "url":         f"http://localhost:{API_PORT}",
        "priority":    "high",
        "hardware_accel": True,
    },
    "quantum-dev": {
        "name":        "QuantumDev",
        "browser":     "chromium",
        "description": "Desarrollo Q# / Qiskit / WebGPU",
        "url":         f"http://localhost:{API_PORT}/api/v1/status",
        "priority":    "high",
        "hardware_accel": True,
    },
    "secure": {
        "name":        "Secure",
        "browser":     "brave",
        "description": "Navegación segura y privada",
        "url":         "about:newtab",
        "priority":    "normal",
        "hardware_accel": True,
    },
}

# ── Estado compartido ─────────────────────────────────────────────────

@dataclass
class BrowserProcess:
    pid:            int
    name:           str            # "firefox", "chromium", "brave"
    profile:        str            # "energy-grid", "quantum-dev", "secure"
    cpu_pct:        float = 0.0
    mem_mb:         float = 0.0
    priority:       str   = "normal"
    tabs_count:     int   = 0
    timestamp:      str   = field(default_factory=lambda: datetime.now(timezone.utc).isoformat())


@dataclass
class QAOABrowserEvent:
    trigger:        str            # "cpu_spike" | "memory_pressure" | "manual"
    affected_pids:  List[int]
    cpu_before:     float
    cpu_after:      float
    action_taken:   str
    energy_saved_j: float
    timestamp:      str   = field(default_factory=lambda: datetime.now(timezone.utc).isoformat())


class BrowserState:
    """Estado global de los navegadores — thread-safe."""
    def __init__(self):
        self._lock            = threading.Lock()
        self.processes:  Dict[int, BrowserProcess] = {}
        self.history:    deque = deque(maxlen=HISTORY_MAX)
        self.qaoa_events: deque = deque(maxlen=QAOA_EVENTS_MAX)
        self.total_cpu_pct:  float = 0.0
        self.total_mem_mb:   float = 0.0
        self.optimizations:  int   = 0
        self.energy_saved_j: float = 0.0
        self.config: Dict[str, Any] = {
            "cpu_spike_threshold_pct": CPU_SPIKE_THRESHOLD_PCT,
            "qaoa_worker_interval_s":  QAOA_WORKER_INTERVAL_S,
            "cgroup_enabled":          _is_cgroup_v2(),
            "hardware_accel_enabled":  True,
        }

    def update_processes(self, procs: List[BrowserProcess]) -> None:
        with self._lock:
            self.processes = {p.pid: p for p in procs}
            self.total_cpu_pct = sum(p.cpu_pct for p in procs)
            self.total_mem_mb  = sum(p.mem_mb  for p in procs)
            snapshot = {
                "timestamp":     datetime.now(timezone.utc).isoformat(),
                "total_cpu_pct": round(self.total_cpu_pct, 2),
                "total_mem_mb":  round(self.total_mem_mb, 1),
                "process_count": len(procs),
            }
            self.history.append(snapshot)

    def add_qaoa_event(self, evt: QAOABrowserEvent) -> None:
        with self._lock:
            self.qaoa_events.appendleft(asdict(evt))
            self.optimizations += 1
            self.energy_saved_j += evt.energy_saved_j

    def snapshot(self) -> Dict[str, Any]:
        with self._lock:
            return {
                "processes":     [asdict(p) for p in self.processes.values()],
                "total_cpu_pct": round(self.total_cpu_pct, 2),
                "total_mem_mb":  round(self.total_mem_mb, 1),
                "optimizations": self.optimizations,
                "energy_saved_j": round(self.energy_saved_j, 4),
                "cgroup_enabled": self.config["cgroup_enabled"],
                "history_points": len(self.history),
            }


_state = BrowserState()


# ── Helpers del sistema ────────────────────────────────────────────────

def _is_cgroup_v2() -> bool:
    return Path("/sys/fs/cgroup/cgroup.controllers").exists()


def _detect_browser_processes() -> List[BrowserProcess]:
    """Escanea /proc buscando procesos de navegadores activos."""
    browsers = []
    proc_base = Path("/proc")

    for entry in proc_base.iterdir():
        if not entry.name.isdigit():
            continue
        pid = int(entry.name)
        comm_file = entry / "comm"
        if not comm_file.exists():
            continue
        try:
            comm = comm_file.read_text().strip().lower()
        except (PermissionError, FileNotFoundError):
            continue

        browser_names = ["firefox", "chromium", "brave", "chrome", "falkon"]
        matched = next((b for b in browser_names if b in comm), None)
        if not matched:
            continue

        # Leer CPU y memoria
        try:
            stat = (entry / "stat").read_text().split()
            cpu_ticks = int(stat[13]) + int(stat[14])
        except Exception:
            cpu_ticks = 0

        try:
            mem_kb = int((entry / "status").read_text().split("VmRSS:")[1].split()[0])
            mem_mb = mem_kb / 1024.0
        except Exception:
            mem_mb = 0.0

        # Inferir perfil desde cmdline
        profile = "secure"
        try:
            cmdline = (entry / "cmdline").read_text().replace('\0', ' ').lower()
            if "energy-grid" in cmdline or f"localhost:{API_PORT}" in cmdline:
                profile = "energy-grid"
            elif "quantum-dev" in cmdline or "qiskit" in cmdline or "qsharp" in cmdline:
                profile = "quantum-dev"
        except Exception:
            pass

        browsers.append(BrowserProcess(
            pid=pid, name=matched, profile=profile,
            cpu_pct=min(cpu_ticks / 100.0, 100.0),
            mem_mb=round(mem_mb, 1),
            priority="high" if profile in ("energy-grid", "quantum-dev") else "normal",
        ))

    return browsers


def _run_qaoa_optimization(processes: List[BrowserProcess]) -> Optional[QAOABrowserEvent]:
    """
    Ejecuta QAOA simulado para redistribuir carga entre procesos de navegadores.
    En producción: llama al photonic_core.rs via subprocess o PyO3.
    """
    if not processes:
        return None

    total_cpu = sum(p.cpu_pct for p in processes)
    if total_cpu < CPU_SPIKE_THRESHOLD_PCT:
        return None

    log.info("QAOA trigger: CPU=%.1f%% > %.1f%%", total_cpu, CPU_SPIKE_THRESHOLD_PCT)

    # Simular QAOA: reducir CPU de procesos no prioritarios
    optimized_cpu = 0.0
    for p in processes:
        if p.priority != "high":
            optimized_cpu += p.cpu_pct * 0.6   # 40% reducción via cgroup
        else:
            optimized_cpu += p.cpu_pct          # sin cambio

    cpu_after     = optimized_cpu
    energy_saved  = (total_cpu - cpu_after) * 0.001 * 65.0  # ~65W TDP, en Joules/tick

    # Aplicar priorización via cgroup manager si disponible
    if _is_cgroup_v2():
        for p in processes:
            if p.priority != "high":
                try:
                    subprocess.run(
                        [sys.executable,
                         str(Path(__file__).parent / "browser_cgroup_manager.py"),
                         "--pid", str(p.pid), "--priority", "low"],
                        capture_output=True, timeout=2,
                    )
                except Exception:
                    pass

    return QAOABrowserEvent(
        trigger="cpu_spike",
        affected_pids=[p.pid for p in processes],
        cpu_before=round(total_cpu, 2),
        cpu_after=round(cpu_after, 2),
        action_taken="cgroup_renice_low_priority",
        energy_saved_j=round(energy_saved, 4),
    )


# ── Worker background ──────────────────────────────────────────────────

_worker_running = False

def _qaoa_worker() -> None:
    global _worker_running
    _worker_running = True
    log.info("QAOA browser worker iniciado (intervalo=%.1fs)", QAOA_WORKER_INTERVAL_S)

    while _worker_running:
        try:
            procs = _detect_browser_processes()
            _state.update_processes(procs)

            evt = _run_qaoa_optimization(procs)
            if evt:
                _state.add_qaoa_event(evt)
                log.info("QAOA aplicado: CPU %.1f%% → %.1f%%, ahorro=%.4fJ",
                         evt.cpu_before, evt.cpu_after, evt.energy_saved_j)
        except Exception as e:
            log.warning("Error en QAOA worker: %s", e)

        time.sleep(QAOA_WORKER_INTERVAL_S)


def start_qaoa_worker() -> threading.Thread:
    t = threading.Thread(target=_qaoa_worker, daemon=True, name="qaoa-browser-worker")
    t.start()
    log.info("Worker QAOA de browsers iniciado en background")
    return t


# ── Blueprint Flask ───────────────────────────────────────────────────

browser_bp = Blueprint("browsers", __name__, url_prefix="/api/v1/browser")


@browser_bp.get("/status")
def browser_status():
    """Estado actual de todos los navegadores."""
    procs = _detect_browser_processes()
    _state.update_processes(procs)
    snap = _state.snapshot()
    snap["profiles"] = BROWSER_PROFILES
    snap["timestamp"] = datetime.now(timezone.utc).isoformat()
    return jsonify({"success": True, "data": snap})


@browser_bp.get("/history")
def browser_history():
    """Historial de métricas de los últimos N ciclos."""
    n = int(request.args.get("n", 50))
    with _state._lock:
        data = list(_state.history)[-n:]
    return jsonify({"success": True, "data": data, "total": len(_state.history)})


@browser_bp.get("/profiles")
def browser_profiles():
    """Perfiles de navegador disponibles."""
    return jsonify({"success": True, "data": BROWSER_PROFILES})


@browser_bp.post("/open")
def browser_open():
    """
    Abre un navegador con el perfil especificado.

    Body JSON:
        profile: "energy-grid" | "quantum-dev" | "secure"
        url:     (opcional) URL a abrir
    """
    data    = request.get_json(silent=True) or {}
    profile = data.get("profile", "energy-grid")
    url     = data.get("url", BROWSER_PROFILES.get(profile, {}).get("url", f"http://localhost:{API_PORT}"))

    if profile not in BROWSER_PROFILES:
        return jsonify({"success": False, "error": f"Perfil desconocido: {profile}"}), 400

    result = subprocess.run(
        ["/usr/bin/qeos-browser", profile, url],
        capture_output=True, timeout=5,
    )

    return jsonify({
        "success":  result.returncode == 0,
        "profile":  profile,
        "url":      url,
        "launched": result.returncode == 0,
    })


@browser_bp.post("/optimize")
def browser_optimize():
    """Forzar optimización QAOA manual en todos los navegadores."""
    procs = _detect_browser_processes()
    _state.update_processes(procs)

    if not procs:
        return jsonify({"success": True, "message": "Sin procesos de navegadores activos"})

    evt = _run_qaoa_optimization(procs)
    if evt:
        _state.add_qaoa_event(evt)
        return jsonify({
            "success":       True,
            "cpu_before":    evt.cpu_before,
            "cpu_after":     evt.cpu_after,
            "reduction_pct": round((1 - evt.cpu_after / max(evt.cpu_before, 0.01)) * 100, 1),
            "energy_saved_j": evt.energy_saved_j,
            "pids_affected":  evt.affected_pids,
        })
    return jsonify({"success": True, "message": "CPU dentro del umbral — sin optimización necesaria"})


@browser_bp.get("/qaoa/events")
def qaoa_events():
    """Últimos eventos QAOA aplicados a navegadores."""
    n = int(request.args.get("n", 20))
    with _state._lock:
        evts = list(_state.qaoa_events)[:n]
    return jsonify({
        "success":     True,
        "events":      evts,
        "total":       _state.optimizations,
        "energy_saved_j": round(_state.energy_saved_j, 4),
    })


@browser_bp.post("/metrics")
def receive_metrics():
    """Recibir telemetría del daemon Rust (PhotonicQ Bridge)."""
    data = request.get_json(silent=True) or {}
    procs = []
    for p_data in data.get("processes", []):
        try:
            procs.append(BrowserProcess(
                pid=p_data["pid"],
                name=p_data.get("name", "unknown"),
                profile=p_data.get("profile", "secure"),
                cpu_pct=float(p_data.get("cpu_pct", 0)),
                mem_mb=float(p_data.get("mem_mb", 0)),
                priority=p_data.get("priority", "normal"),
            ))
        except (KeyError, ValueError):
            continue

    _state.update_processes(procs)
    total = _state.total_cpu_pct

    # Trigger QAOA automático si hay spike
    evt = None
    if total >= CPU_SPIKE_THRESHOLD_PCT:
        evt = _run_qaoa_optimization(procs)
        if evt:
            _state.add_qaoa_event(evt)

    return jsonify({
        "success":   True,
        "received":  len(procs),
        "total_cpu": round(total, 2),
        "qaoa_triggered": evt is not None,
    })


@browser_bp.post("/config")
def update_config():
    """Actualizar configuración del monitor en tiempo real."""
    data = request.get_json(silent=True) or {}
    with _state._lock:
        for key in ("cpu_spike_threshold_pct", "qaoa_worker_interval_s"):
            if key in data:
                _state.config[key] = float(data[key])
    return jsonify({"success": True, "config": _state.config})


# ── Integración principal ─────────────────────────────────────────────

def integrate_browsers(flask_app: Flask) -> None:
    """
    Integra todos los endpoints de browsers en el app Flask existente.

    Llamar desde api/server.py:
        from browser_integration import integrate_browsers
        integrate_browsers(app)
    """
    flask_app.register_blueprint(browser_bp)
    start_qaoa_worker()
    log.info("✅ Browser integration activada — endpoints en /api/v1/browser/")
    log.info("   Perfiles: %s", ", ".join(BROWSER_PROFILES.keys()))
    log.info("   QAOA worker: cada %.1fs, umbral CPU=%.1f%%",
             QAOA_WORKER_INTERVAL_S, CPU_SPIKE_THRESHOLD_PCT)
