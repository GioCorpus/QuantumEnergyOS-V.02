#!/usr/bin/env python3
"""
browser_monitor_module.py — QuantumEnergyOS V.02
══════════════════════════════════════════════════════════════════════════

Módulo Flask que extiende el server.py existente con endpoints para:

  1. Recibir telemetría del daemon Rust  →  POST /api/browser_metrics
  2. Recibir triggers QAOA del daemon    →  POST /api/qaoa/trigger_browser
  3. Consultar estado actual             →  GET  /api/browser/status
  4. Consultar historial (últimas N)     →  GET  /api/browser/history
  5. Configurar el daemon en tiempo real →  POST /api/browser/config
  6. Forzar optimización QAOA manual     →  POST /api/browser/optimize

## Integración con server.py existente

Agregar al final del server.py (antes del `if __name__ == '__main__':`):

    from browser_monitor_module import register_browser_routes, start_qaoa_worker
    register_browser_routes(app, energy_data, quantum_state)
    start_qaoa_worker()

El módulo comparte los dicts `energy_data` y `quantum_state` del server.py,
añadiendo las claves `browser_load` y `qaoa_browser_events` a cada uno.

Autor: Giovanny Anthony Corpus Bernal — Mexicali, BC
"""

from flask import Flask, Blueprint, request, jsonify
from dataclasses import dataclass, asdict, field
from typing import List, Dict, Optional, Any
from collections import deque
from datetime import datetime
import threading
import time
import math
import json
import logging

logger = logging.getLogger("quantum_browser_monitor")
logging.basicConfig(
    level=logging.INFO,
    format="[%(asctime)s] %(levelname)s %(name)s: %(message)s",
    datefmt="%H:%M:%S"
)

# ── Constantes ────────────────────────────────────────────────────────────────

# Umbral de CPU (%) para activar QAOA — debe coincidir con el daemon Rust
CPU_SPIKE_THRESHOLD_PCT: float = 8.0

# Cuántas muestras de historial conservar en memoria
HISTORY_MAX_SAMPLES: int = 200

# Intervalo del worker QAOA en background [s]
QAOA_WORKER_INTERVAL_S: float = 5.0

# Cuántos eventos QAOA recientes conservar
QAOA_EVENTS_MAX: int = 50

# ── Modelos de datos ──────────────────────────────────────────────────────────

@dataclass
class BrowserProcessRecord:
    pid: int
    comm: str
    kind: str
    cpu_pct: float
    nice: int
    rss_kb: int


@dataclass
class BrowserMetricsSample:
    """Una muestra completa de telemetría enviada por el daemon Rust."""
    timestamp: int           # Unix timestamp
    total_cpu_pct: float     # CPU total de navegadores [%]
    total_gpu_pct: float     # GPU total [%]
    total_rss_mb: float      # RAM total de navegadores [MB]
    num_cpus: int            # Núcleos lógicos del sistema
    qaoa_triggered: bool     # ¿El daemon disparó QAOA?
    browser_count: int       # Número de procesos de navegadores
    processes: List[BrowserProcessRecord] = field(default_factory=list)

    def to_dict(self) -> dict:
        d = asdict(self)
        d['datetime'] = datetime.fromtimestamp(self.timestamp).isoformat()
        d['cpu_per_core'] = self.total_cpu_pct / max(self.num_cpus, 1)
        return d


@dataclass
class QaoaBrowserEvent:
    """Evento registrado cuando se activa el optimizador QAOA por carga de navegadores."""
    timestamp: int
    trigger_source: str       # "daemon_threshold" | "manual" | "worker_watchdog"
    cpu_pct_at_trigger: float
    gpu_pct_at_trigger: float
    action_taken: str         # Descripción de la acción QAOA tomada
    energy_delta_w: float     # Estimación de watts ahorrados/redistribuidos
    qaoa_iteration: int       # Número de iteración QAOA


# ── Estado global del módulo ──────────────────────────────────────────────────

class BrowserMonitorState:
    """
    Estado compartido thread-safe entre el Flask y el worker QAOA.
    Se inicializa una sola instancia global al registrar el módulo.
    """
    def __init__(self):
        self._lock = threading.RLock()

        # Última muestra recibida del daemon
        self.latest_sample: Optional[BrowserMetricsSample] = None

        # Historial circular de muestras
        self.history: deque = deque(maxlen=HISTORY_MAX_SAMPLES)

        # Historial de eventos QAOA
        self.qaoa_events: deque = deque(maxlen=QAOA_EVENTS_MAX)

        # Contador de iteraciones QAOA
        self.qaoa_iteration: int = 0

        # Configuración editable en runtime
        self.config: dict = {
            "cpu_threshold_pct": CPU_SPIKE_THRESHOLD_PCT,
            "report_interval_s": 30,
            "high_priority_keywords": [
                "energy", "solar", "quantum", "arxiv", "wikipedia",
                "nasa", "cfe.mx", "cenace.gob"
            ],
            "nice_high": -5,
            "nice_low": 10,
            "qaoa_enabled": True,
        }

        # Referencia al energy_data y quantum_state del server.py principal
        self._shared_energy_data: Optional[dict] = None
        self._shared_quantum_state: Optional[dict] = None

        # Stats del módulo
        self.stats = {
            "samples_received": 0,
            "qaoa_triggers_total": 0,
            "last_spike_time": None,
            "uptime_start": time.time(),
        }

    def attach_shared_state(self, energy_data: dict, quantum_state: dict):
        """Enlaza con los dicts compartidos del server.py principal."""
        with self._lock:
            self._shared_energy_data = energy_data
            self._shared_quantum_state = quantum_state
            # Añadir claves de navegadores al estado compartido
            energy_data.setdefault("browser_load_w", 0.0)
            energy_data.setdefault("browser_cpu_pct", 0.0)
            quantum_state.setdefault("qaoa_browser_events", [])

    def ingest_sample(self, sample: BrowserMetricsSample):
        """
        Ingiere una nueva muestra y actualiza el estado compartido con server.py.
        Thread-safe.
        """
        with self._lock:
            self.latest_sample = sample
            self.history.append(sample)
            self.stats["samples_received"] += 1

            # Actualizar los dicts del server.py principal
            if self._shared_energy_data is not None:
                # Estimar consumo energético: ~2W por % de CPU de navegadores
                # (muy aproximado, pero útil para el balanceador de red)
                estimated_browser_w = sample.total_cpu_pct * 2.0
                self._shared_energy_data["browser_load_w"] = estimated_browser_w
                self._shared_energy_data["browser_cpu_pct"] = sample.total_cpu_pct

                # Añadir al current_load si supera el umbral (carga real en la red)
                if sample.total_cpu_pct > self.config["cpu_threshold_pct"]:
                    self._shared_energy_data["current_load"] = (
                        self._shared_energy_data.get("current_load", 0.0)
                        + estimated_browser_w * 0.1  # peso moderado
                    )

            if sample.qaoa_triggered:
                self.stats["qaoa_triggers_total"] += 1
                self.stats["last_spike_time"] = datetime.now().isoformat()

    def register_qaoa_event(self, event: QaoaBrowserEvent):
        """Registra un evento QAOA y lo propaga al quantum_state del server.py."""
        with self._lock:
            self.qaoa_events.append(event)
            self.qaoa_iteration += 1
            if self._shared_quantum_state is not None:
                events_list = self._shared_quantum_state.get("qaoa_browser_events", [])
                events_list.append(asdict(event))
                if len(events_list) > QAOA_EVENTS_MAX:
                    events_list.pop(0)
                self._shared_quantum_state["qaoa_browser_events"] = events_list

    def get_latest_or_empty(self) -> dict:
        with self._lock:
            if self.latest_sample:
                return self.latest_sample.to_dict()
            return {
                "status": "no_data",
                "message": "Daemon no ha enviado datos aún. "
                           "Inicia quantum-browser-daemon para recibir métricas reales.",
                "timestamp": int(time.time()),
            }

    def get_history_dicts(self, n: int = 50) -> List[dict]:
        with self._lock:
            return [s.to_dict() for s in list(self.history)[-n:]]

    def get_config(self) -> dict:
        with self._lock:
            return dict(self.config)

    def update_config(self, updates: dict) -> dict:
        with self._lock:
            allowed_keys = set(self.config.keys())
            for k, v in updates.items():
                if k in allowed_keys:
                    self.config[k] = v
            return dict(self.config)


# Instancia global del módulo
_state = BrowserMonitorState()


# ── QAOA Browser Worker ───────────────────────────────────────────────────────

def _qaoa_browser_optimize(cpu_pct: float, gpu_pct: float, source: str) -> QaoaBrowserEvent:
    """
    Ejecuta el algoritmo QAOA para redistribuir la carga de red generada
    por los navegadores.

    ## Modelo QAOA simplificado (3 qubits, p=1)

    El espacio de decisión tiene 8 acciones posibles:

      0: No hacer nada
      1: Reducir frecuencia de CPU del navegador (cpufreq)
      2: Mover proceso a cgroup de baja prioridad
      3: Activar modo de ahorro de GPU
      4: Redistribuir carga a otro nodo de red
      5: Throttle de I/O del navegador
      6: Combinar CPU + GPU throttle
      7: Activar balanceo cuántico completo

    La función de coste maximiza: ahorro_energético / impacto_usuario.

    En producción, esto se puede sustituir llamando al qaoa_optimize.py
    del módulo antigrav-bridge o a un circuito Qiskit-Aer real.
    """
    import math

    # ── Función de coste para cada acción ────────────────────────────────────
    # c(acción) = (ahorro_estimado_W) / (1 + penalización_usuario)
    weight = (cpu_pct / 100.0)  # 0..1, normalizado

    costs = [
        0.0,                                # 0: noop
        weight * 15.0 / (1 + 0.1),         # 1: cpufreq reduce — bajo impacto
        weight * 20.0 / (1 + 0.2),         # 2: cgroup — impacto medio-bajo
        (gpu_pct / 100.0) * 25.0 / 1.15,   # 3: GPU throttle
        weight * 30.0 / (1 + 0.3),         # 4: redistribuir nodo
        weight * 10.0 / (1 + 0.05),        # 5: I/O throttle — muy bajo impacto
        (weight + gpu_pct / 200.0) * 35.0 / (1 + 0.4),  # 6: CPU + GPU
        weight * 50.0 / (1 + 0.8),         # 7: balanceo completo — alto impacto
    ]

    # ── QAOA p=1 clásico (espejo del código Rust en antigrav_bridge.rs) ──────
    n_states = 8
    n_qubits = 3
    inv_sqrt8 = 1.0 / math.sqrt(n_states)

    # Barrido de grilla 15×15 sobre (γ, β)
    best_action = 1  # default: cpufreq
    best_exp = -math.inf

    for gi in range(1, 16):
        for bi in range(1, 16):
            gamma = gi * math.pi / 15
            beta = bi * (math.pi / 2) / 15

            # Inicializar |+⟩^3
            psi_re = [inv_sqrt8] * n_states
            psi_im = [0.0] * n_states

            # Separador de fase: ψ_j *= e^{iγ·c_j}
            for j in range(n_states):
                cos_g = math.cos(gamma * costs[j])
                sin_g = math.sin(gamma * costs[j])
                re_new = psi_re[j] * cos_g - psi_im[j] * sin_g
                im_new = psi_re[j] * sin_g + psi_im[j] * cos_g
                psi_re[j] = re_new
                psi_im[j] = im_new

            # Mezclador: e^{-iβ X_k} para cada qubit k
            for k in range(n_qubits):
                cos_b = math.cos(beta)
                sin_b = math.sin(beta)
                new_re = [0.0] * n_states
                new_im = [0.0] * n_states
                for j in range(n_states):
                    partner = j ^ (1 << k)
                    # cos(β)·ψ_j − i·sin(β)·ψ_partner
                    new_re[j] = cos_b * psi_re[j] + sin_b * psi_im[partner]
                    new_im[j] = cos_b * psi_im[j] - sin_b * psi_re[partner]
                psi_re = new_re
                psi_im = new_im

            # Valor esperado ⟨H_C⟩ = Σ |ψ_j|² · c_j
            exp_val = sum(
                (psi_re[j]**2 + psi_im[j]**2) * costs[j]
                for j in range(n_states)
            )

            if exp_val > best_exp:
                best_exp = exp_val
                # Estado más probable
                probs = [psi_re[j]**2 + psi_im[j]**2 for j in range(n_states)]
                best_action = probs.index(max(probs))

    # ── Acciones concretas del SO ─────────────────────────────────────────────
    action_descriptions = [
        "NOOP: Sin acción necesaria",
        "cpufreq: Reducción de frecuencia CPU del navegador vía cpufreq-set",
        "cgroup: Mover procesos de navegador a cgroup cpu.weight=256 (25% del peso normal)",
        "gpu_throttle: Límite de frecuencia GPU via sysfs (gt_max_freq_mhz reducido 30%)",
        "grid_redistribute: Redistribuir carga al nodo de red con menor utilización",
        "io_throttle: Límite de I/O del navegador vía blkio.weight=128",
        "cpu_gpu_combined: cpufreq + gpu_throttle simultáneos",
        "full_quantum_balance: Activar balanceo cuántico completo del EnergyOptimizer",
    ]

    energy_savings = [0.0, 15.0, 20.0, 25.0, 30.0, 10.0, 35.0, 50.0]
    estimated_saving = energy_savings[best_action] * (cpu_pct / 100.0)

    return QaoaBrowserEvent(
        timestamp=int(time.time()),
        trigger_source=source,
        cpu_pct_at_trigger=cpu_pct,
        gpu_pct_at_trigger=gpu_pct,
        action_taken=action_descriptions[best_action],
        energy_delta_w=estimated_saving,
        qaoa_iteration=_state.qaoa_iteration,
    )


def _qaoa_worker_loop():
    """
    Worker en background que vigila el estado del monitor y activa QAOA
    si se mantiene un pico sostenido (más de 3 muestras consecutivas sobre umbral).

    Corre en un hilo daemon separado, no bloquea el servidor Flask.
    """
    logger.info("[QAOA worker] Iniciando worker de vigilancia en background")
    consecutive_spikes = 0
    SPIKE_SUSTAIN_COUNT = 3  # Activar QAOA solo si 3 muestras consecutivas sobre umbral

    while True:
        time.sleep(QAOA_WORKER_INTERVAL_S)

        try:
            with _state._lock:
                sample = _state.latest_sample
                qaoa_enabled = _state.config.get("qaoa_enabled", True)
                threshold = _state.config.get("cpu_threshold_pct", CPU_SPIKE_THRESHOLD_PCT)

            if sample is None or not qaoa_enabled:
                consecutive_spikes = 0
                continue

            cpu_normalized = sample.total_cpu_pct / max(sample.num_cpus, 1)

            if cpu_normalized > threshold:
                consecutive_spikes += 1
                logger.warning(
                    "[QAOA worker] Pico sostenido %d/3 — CPU: %.2f%% (umbral: %.1f%%)",
                    consecutive_spikes, cpu_normalized, threshold
                )
            else:
                if consecutive_spikes > 0:
                    logger.info("[QAOA worker] Pico resuelto tras %d muestras", consecutive_spikes)
                consecutive_spikes = 0
                continue

            if consecutive_spikes >= SPIKE_SUSTAIN_COUNT:
                consecutive_spikes = 0  # Reset para no disparar en loop
                logger.warning(
                    "[QAOA worker] ¡Activando QAOA! CPU sostenida: %.2f%%", cpu_normalized
                )
                event = _qaoa_browser_optimize(
                    cpu_pct=sample.total_cpu_pct,
                    gpu_pct=sample.total_gpu_pct,
                    source="worker_watchdog",
                )
                _state.register_qaoa_event(event)
                logger.info(
                    "[QAOA worker] Acción: %s | Ahorro estimado: %.1f W",
                    event.action_taken, event.energy_delta_w
                )

        except Exception as e:
            logger.error("[QAOA worker] Error en loop: %s", e, exc_info=True)


def start_qaoa_worker():
    """Inicia el worker QAOA en un hilo daemon. Llamar una sola vez al inicio."""
    t = threading.Thread(target=_qaoa_worker_loop, name="qaoa-browser-worker", daemon=True)
    t.start()
    logger.info("[QAOA worker] Hilo iniciado (daemon=True)")


# ── Endpoints Flask ───────────────────────────────────────────────────────────

def register_browser_routes(app: Flask, energy_data: dict, quantum_state: dict):
    """
    Registra todas las rutas del módulo en la app Flask existente.
    Llamar desde server.py después de crear la instancia de Flask.

    Ejemplo:
        from browser_monitor_module import register_browser_routes, start_qaoa_worker
        register_browser_routes(app, energy_data, quantum_state)
        start_qaoa_worker()
    """
    _state.attach_shared_state(energy_data, quantum_state)
    logger.info("[monitor] Módulo registrado en Flask. Escuchando en /api/browser_*")

    # ── POST /api/browser_metrics ─────────────────────────────────────────────
    # Recibe telemetría del daemon Rust cada 30s
    @app.route("/api/browser_metrics", methods=["POST"])
    def receive_browser_metrics():
        """
        Endpoint principal: recibe la telemetría del daemon Rust.

        Body JSON esperado (generado por telemetry_to_json() en main.rs):
        {
            "timestamp": 1700000000,
            "total_cpu_pct": 12.5,
            "total_gpu_pct": 3.2,
            "total_rss_mb": 1200.5,
            "num_cpus": 8,
            "qaoa_triggered": false,
            "browser_count": 4,
            "processes": [
                {"pid": 1234, "comm": "firefox", "kind": "firefox",
                 "cpu_pct": 8.1, "nice": 0, "rss_kb": 512000}
            ]
        }
        """
        data = request.get_json(silent=True)
        if not data:
            return jsonify({"error": "JSON inválido o vacío"}), 400

        try:
            processes = [
                BrowserProcessRecord(
                    pid=p.get("pid", 0),
                    comm=p.get("comm", ""),
                    kind=p.get("kind", "unknown"),
                    cpu_pct=float(p.get("cpu_pct", 0.0)),
                    nice=int(p.get("nice", 0)),
                    rss_kb=int(p.get("rss_kb", 0)),
                )
                for p in data.get("processes", [])
            ]

            sample = BrowserMetricsSample(
                timestamp=int(data.get("timestamp", time.time())),
                total_cpu_pct=float(data.get("total_cpu_pct", 0.0)),
                total_gpu_pct=float(data.get("total_gpu_pct", 0.0)),
                total_rss_mb=float(data.get("total_rss_mb", 0.0)),
                num_cpus=int(data.get("num_cpus", 1)),
                qaoa_triggered=bool(data.get("qaoa_triggered", False)),
                browser_count=int(data.get("browser_count", 0)),
                processes=processes,
            )
            _state.ingest_sample(sample)

            return jsonify({
                "status": "ok",
                "samples_received": _state.stats["samples_received"],
                "timestamp": int(time.time()),
            }), 200

        except (KeyError, ValueError, TypeError) as e:
            logger.warning("Error procesando browser_metrics: %s | data=%s", e, str(data)[:200])
            return jsonify({"error": str(e)}), 422

    # ── POST /api/qaoa/trigger_browser ────────────────────────────────────────
    # Llamado por el daemon Rust cuando supera el umbral inmediatamente
    @app.route("/api/qaoa/trigger_browser", methods=["POST"])
    def qaoa_trigger_browser():
        """
        El daemon Rust llama este endpoint cuando detecta un pico de CPU
        que supera el umbral en tiempo real (no espera al worker de 30s).
        """
        data = request.get_json(silent=True) or {}
        cpu_pct = float(data.get("cpu_pct", 0.0))
        gpu_pct = float(data.get("gpu_pct", 0.0))

        if not _state.config.get("qaoa_enabled", True):
            return jsonify({"status": "qaoa_disabled"}), 200

        logger.warning(
            "[QAOA trigger] CPU: %.2f%% | GPU: %.2f%% — ejecutando QAOA",
            cpu_pct, gpu_pct
        )

        event = _qaoa_browser_optimize(cpu_pct, gpu_pct, source="daemon_threshold")
        _state.register_qaoa_event(event)

        return jsonify({
            "status": "qaoa_executed",
            "action_taken": event.action_taken,
            "energy_delta_w": event.energy_delta_w,
            "qaoa_iteration": event.qaoa_iteration,
            "timestamp": event.timestamp,
        }), 200

    # ── GET /api/browser/status ───────────────────────────────────────────────
    @app.route("/api/browser/status")
    def browser_status():
        """Estado actual del monitor de navegadores."""
        data = _state.get_latest_or_empty()
        uptime_s = int(time.time() - _state.stats["uptime_start"])
        return jsonify({
            **data,
            "module_stats": {
                **_state.stats,
                "uptime_s": uptime_s,
                "qaoa_events_count": len(_state.qaoa_events),
                "history_samples": len(_state.history),
            },
            "config": _state.get_config(),
        })

    # ── GET /api/browser/history ──────────────────────────────────────────────
    @app.route("/api/browser/history")
    def browser_history():
        """Historial de muestras de telemetría (últimas N)."""
        n = min(int(request.args.get("n", 50)), HISTORY_MAX_SAMPLES)
        samples = _state.get_history_dicts(n)
        return jsonify({
            "count": len(samples),
            "samples": samples,
            "timestamp": int(time.time()),
        })

    # ── GET /api/browser/qaoa_events ─────────────────────────────────────────
    @app.route("/api/browser/qaoa_events")
    def browser_qaoa_events():
        """Historial de eventos QAOA disparados."""
        with _state._lock:
            events = [asdict(e) for e in list(_state.qaoa_events)]
        return jsonify({
            "count": len(events),
            "total_triggers": _state.stats["qaoa_triggers_total"],
            "events": events[-20:],
            "timestamp": int(time.time()),
        })

    # ── POST /api/browser/config ──────────────────────────────────────────────
    @app.route("/api/browser/config", methods=["POST"])
    def browser_config():
        """
        Actualiza la configuración del módulo en tiempo real.

        Body JSON: cualquier subconjunto de las claves de config:
            {"cpu_threshold_pct": 12.0, "qaoa_enabled": false}
        """
        updates = request.get_json(silent=True) or {}
        new_config = _state.update_config(updates)
        logger.info("Config actualizada: %s", new_config)
        return jsonify({"status": "ok", "config": new_config})

    # ── POST /api/browser/optimize ────────────────────────────────────────────
    @app.route("/api/browser/optimize", methods=["POST"])
    def browser_optimize_manual():
        """Fuerza una ejecución manual del optimizador QAOA."""
        latest = _state.latest_sample
        if latest is None:
            return jsonify({"error": "Sin datos del daemon aún"}), 503

        event = _qaoa_browser_optimize(
            cpu_pct=latest.total_cpu_pct,
            gpu_pct=latest.total_gpu_pct,
            source="manual",
        )
        _state.register_qaoa_event(event)
        logger.info("[manual QAOA] Acción: %s", event.action_taken)
        return jsonify({
            "status": "ok",
            "event": asdict(event),
        })

    logger.info("[monitor] Rutas registradas: /api/browser_metrics, /api/browser/*, /api/qaoa/trigger_browser")


# ── Script standalone para pruebas ───────────────────────────────────────────
# Permite probar el módulo sin el server.py completo:
#   python3 browser_monitor_module.py

if __name__ == "__main__":
    app = Flask(__name__)

    # Dicts dummy que simularán el energy_data y quantum_state del server.py real
    energy_data_stub = {
        "current_load": 450.0,
        "available_capacity": 600.0,
        "efficiency": 0.92,
    }
    quantum_state_stub = {
        "active_qubits": 8,
        "coherence": 0.95,
    }

    register_browser_routes(app, energy_data_stub, quantum_state_stub)
    start_qaoa_worker()

    print("╔═══════════════════════════════════════════════════════════╗")
    print("║  QuantumEnergyOS V.02 — Browser Monitor Module (standalone)║")
    print("╚═══════════════════════════════════════════════════════════╝")
    print()
    print("Endpoints disponibles:")
    print("  POST /api/browser_metrics       ← daemon Rust envía aquí")
    print("  POST /api/qaoa/trigger_browser  ← daemon Rust (pico inmediato)")
    print("  GET  /api/browser/status        ← estado actual")
    print("  GET  /api/browser/history       ← historial de muestras")
    print("  GET  /api/browser/qaoa_events   ← historial QAOA")
    print("  POST /api/browser/config        ← cambiar configuración")
    print("  POST /api/browser/optimize      ← forzar QAOA manual")
    print()
    print("Iniciando en http://localhost:5001 (standalone)")
    print()

    app.run(host="0.0.0.0", port=5001, debug=True, threaded=True)
