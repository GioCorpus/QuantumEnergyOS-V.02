"""
climate_orchestrator/__init__.py — Quantum Climate Orchestrator V1
Integración de predicción meteorológica + evaluación de riesgo + acciones automáticas
"""
from __future__ import annotations

import json
import logging
import os
import time
from dataclasses import dataclass, field, asdict
from datetime import datetime, timezone
from enum import Enum
from typing import Any, Dict, List, Optional

from .weather import WeatherClient, WeatherData
from .bridge import ClimateBridge
from .actions import ActionRegistry, Action

log = logging.getLogger("qeos.climate")

# ── Enums ────────────────────────────────────────────────────────────────

class RiskLevel(str, Enum):
    LOW = "low"
    MEDIUM = "medium"
    HIGH = "high"
    CRITICAL = "critical"

class EventType(str, Enum):
    HEATWAVE = "heatwave"
    BLACKOUT_RISK = "blackout_risk"
    CPU_OVERLOAD = "cpu_overload"
    EMERGENCY = "emergency"
    NONE = "none"

# ── Data classes ─────────────────────────────────────────────────────────

@dataclass
class Prediction:
    event_type: str
    confidence: float  # 0.0 - 1.0
    details: str
    timeframe_min: int

    def to_dict(self) -> dict:
        return asdict(self)

@dataclass
class Action:
    id: str
    name: str
    description: str
    impact: str
    command: Optional[str] = None
    executed: bool = False
    dry_run: bool = True
    result: Optional[str] = None

    def to_dict(self) -> dict:
        d = asdict(self)
        if self.dry_run:
            d["note"] = "[DRY RUN] Action would be executed in production"
        return d

@dataclass
class ClimateData:
    temperature_c: float
    humidity: float
    wind_kph: float
    forecast: List[WeatherData] = field(default_factory=list)
    grid_load_percent: float = 0.0
    cpu_load_percent: float = 0.0
    battery_percent: float = 100.0
    timestamp: str = field(default_factory=lambda: datetime.now(timezone.utc).isoformat())

    def to_dict(self) -> dict:
        return asdict(self)

@dataclass
class OrchestratorResult:
    risk_level: str
    summary: str
    predictions: List[Prediction] = field(default_factory=list)
    actions: List[Action] = field(default_factory=list)
    climate_data: ClimateData = field(default_factory=ClimateData)
    analysis_timestamp: str = field(default_factory=lambda: datetime.now(timezone.utc).isoformat())

    def to_json(self) -> str:
        payload = {
            "risk_level": self.risk_level,
            "summary": self.summary,
            "predictions": [p.to_dict() for p in self.predictions],
            "actions": [a.to_dict() for a in self.actions],
            "climate_data": self.climate_data.to_dict(),
            "analysis_timestamp": self.analysis_timestamp,
        }
        return json.dumps(payload, indent=2, ensure_ascii=False)

# ── Core Orchestrator ─────────────────────────────────────────────────────

class ClimateOrchestrator:
    """
    Orquestador Climático Inteligente
    Fusiona datos meteorológicos con métricas del sistema para tomar decisiones.
    """

    RISK_THRESHOLDS = {
        "TEMPERATURE_CRITICAL": 45.0,
        "TEMPERATURE_HIGH": 38.0,
        "GRID_LOAD_MAX": 0.85,
        "CPU_LOAD_MAX": 0.80,
        "BATTERY_EMERGENCY": 30.0,
        "BATTERY_LOW": 50.0,
    }

    def __init__(
        self,
        weather_client: Optional[WeatherClient] = None,
        bridge: Optional[ClimateBridge] = None,
        dry_run: bool = True,
    ):
        self.weather_client = weather_client or WeatherClient(dry_run=dry_run)
        self.bridge = bridge or ClimateBridge(dry_run=dry_run)
        self.dry_run = dry_run
        self.registry = ActionRegistry()
        self._load_actions()

    def _load_actions(self) -> None:
        """Registra acciones predefinidas."""
        self.registry.register(Action(
            id="limit_cpu_frequency",
            name="Limitar frecuencia CPU",
            description="Reduce la frecuencia máxima del CPU para ahorrar energía",
            impact="bajo",
            command="cpufreq",
        ))
        self.registry.register(Action(
            id="shutdown_non_essential",
            name="Apagar servicios no esenciales",
            description="Detiene servicios de desarrollo, logs verbosos, etc.",
            impact="medio",
            command="systemctl stop",
        ))
        self.registry.register(Action(
            id="enable_power_saving",
            name="Activar ahorro de energía",
            description="Habilita modos de ahorro de energía del sistema",
            impact="bajo",
            command="powertop",
        ))
        self.registry.register(Action(
            id="shed_load",
            name="Desconectar cargas no críticas",
            description="Desconecta cargas programables (calentadores, bombas, etc.)",
            impact="alto",
            command="mqtt off",
        ))
        self.registry.register(Action(
            id="enter_emergency_mode",
            name="Modo emergencia",
            description="Solo operaciones críticas, todo lo demás se suspende",
            impact="crítico",
            command="systemctl emergency",
        ))

    def collect_climate_data(self, location: Optional[str] = None) -> ClimateData:
        """Recolecta temperatura, pronóstico y datos del sistema."""
        location = location or os.getenv("QEOS_LOCATION", "Mexicali, Baja California, MX")

        # Datos meteorológicos
        current = self.weather_client.get_current(location)
        forecast = self.weather_client.get_forecast(location, hours=48)

        # Métricas del sistema (simuladas por ahora, en producción desde /proc o agent)
        grid_load = self._get_grid_load()
        cpu_load = self._get_cpu_load()
        battery = self._get_battery_percent()

        data = ClimateData(
            temperature_c=current.temp_c,
            humidity=current.humidity,
            wind_kph=current.wind_kph,
            forecast=forecast,
            grid_load_percent=grid_load,
            cpu_load_percent=cpu_load,
            battery_percent=battery,
        )
        return data

    def _get_grid_load(self) -> float:
        """Obtiene el porcentaje de carga de la red eléctrica local."""
        # En producción: consultar a API de CFE o medidor local
        # Por ahora simulamos basado en hora del día
        hour = datetime.now().hour
        # Simulación simple: días laborables 8-20hrs alta demanda
        if 8 <= hour <= 20:
            return 0.65 + (0.2 * (hour - 8) / 12)  # 65% -> 85%
        return 0.35

    def _get_cpu_load(self) -> float:
        """Obtiene el porcentaje de uso de CPU."""
        try:
            import psutil
            return psutil.cpu_percent(interval=0.1) / 100.0
        except ImportError:
            # Fallback: simulación
            return 0.45

    def _get_battery_percent(self) -> float:
        """Obtiene el porcentaje de carga de batería/UPS."""
        try:
            import psutil
            battery = psutil.sensors_battery()
            return battery.percent if battery else 100.0
        except Exception:
            return 100.0

    def evaluate_risk(self, data: ClimateData) -> tuple[RiskLevel, List[EventType], str]:
        """Evalúa el nivel de riesgo basado en reglas del sistema."""
        risks: List[EventType] = []
        explanation_parts = []

        # 1. Temperatura extrema
        if data.temperature_c >= self.RISK_THRESHOLDS["TEMPERATURE_CRITICAL"]:
            risks.append(EventType.HEATWAVE)
            explanation_parts.append(f"Temperatura crítica ({data.temperature_c:.1f}°C ≥ {self.RISK_THRESHOLDS['TEMPERATURE_CRITICAL']}°C) → Riesgo de sobrecalentamiento y alta demanda")
        elif data.temperature_c >= self.RISK_THRESHOLDS["TEMPERATURE_HIGH"]:
            risks.append(EventType.HEATWAVE)
            explanation_parts.append(f"Temperatura alta ({data.temperature_c:.1f}°C) → Posible ola de calor")

        # 2. Carga de red
        if data.grid_load_percent >= self.RISK_THRESHOLDS["GRID_LOAD_MAX"]:
            risks.append(EventType.BLACKOUT_RISK)
            explanation_parts.append(f"Carga de red {data.grid_load_percent*100:.1f}% ≥ {self.RISK_THRESHOLDS['GRID_LOAD_MAX']*100:.1f}% → Riesgo de apagón")
        elif data.grid_load_percent >= 0.75:
            explanation_parts.append(f"Carga de red elevada ({data.grid_load_percent*100:.1f}%) — monitorear")

        # 3. CPU overload
        if data.cpu_load_percent >= self.RISK_THRESHOLDS["CPU_LOAD_MAX"]:
            risks.append(EventType.CPU_OVERLOAD)
            explanation_parts.append(f"Uso de CPU {data.cpu_load_percent*100:.1f}% ≥ {self.RISK_THRESHOLDS['CPU_LOAD_MAX']*100:.1f}% → Reducir consumo")

        # 4. Batería / energías de reserva
        if data.battery_percent <= self.RISK_THRESHOLDS["BATTERY_EMERGENCY"]:
            risks.append(EventType.EMERGENCY)
            explanation_parts.append(f"Reserva energética {data.battery_percent:.0f}% ≤ {self.RISK_THRESHOLDS['BATTERY_EMERGENCY']}% → MODO EMERGENCIA")
        elif data.battery_percent <= self.RISK_THRESHOLDS["BATTERY_LOW"]:
            explanation_parts.append(f"Reserva energética baja ({data.battery_percent:.0f}%)")

        # 5. Pronóstico adjunto (ola de calor inminente)
        if any(f.temp_c >= 40 for f in data.forecast[:6]):  # Próximas 6 horas
            if EventType.HEATWAVE not in risks:
                risks.append(EventType.HEATWAVE)
            explanation_parts.append("Pronóstico: Temperaturas ≥40°C en las próximas 6h")

        # Determinar nivel de riesgo global
        if EventType.EMERGENCY in risks:
            level = RiskLevel.CRITICAL
        elif EventType.BLACKOUT_RISK in risks or EventType.HEATWAVE in risks:
            level = RiskLevel.HIGH
        elif len(risks) >= 2:
            level = RiskLevel.MEDIUM
        else:
            level = RiskLevel.LOW

        summary = "; ".join(explanation_parts) if explanation_parts else "Sistema estable — sin riesgos detectados"
        return level, risks, summary

    def recommend_actions(self, risk_level: RiskLevel, risks: List[EventType], data: ClimateData) -> List[Action]:
        """Recomienda acciones basadas en el perfil de riesgo."""
        actions: List[Action] = []

        if risk_level == RiskLevel.CRITICAL or EventType.EMERGENCY in risks:
            actions.append(self.registry.get("enter_emergency_mode"))
            actions.append(self.registry.get("shutdown_non_essential"))
            if data.battery_percent <= 30:
                actions.append(self.registry.get("shed_load"))

        if risk_level >= RiskLevel.HIGH:
            if EventType.HEATWAVE in risks:
                actions.append(self.registry.get("enable_power_saving"))
                actions.append(self.registry.get("limit_cpu_frequency"))
            if EventType.BLACKOUT_RISK in risks:
                actions.append(self.registry.get("shed_load"))

        if risk_level == RiskLevel.MEDIUM:
            actions.append(self.registry.get("enable_power_saving"))

        # Deduplicar y marcar dry_run
        seen = set()
        unique_actions = []
        for a in actions:
            if a.id not in seen:
                a.dry_run = self.dry_run
                seen.add(a.id)
                unique_actions.append(a)
        return unique_actions

    def analyze(self, location: Optional[str] = None) -> OrchestratorResult:
        """Pipeline completo:采集 → evaluar → predecir → accionar."""
        log.info("Iniciando análisis climático integral")

        # 1. Colectar datos
        climate_data = self.collect_climate_data(location)

        # 2. Evaluar riesgo
        risk_level, risks, summary = self.evaluate_risk(climate_data)

        # 3. Generar predicciones específicas
        predictions = self._generate_predictions(climate_data, risks, risk_level)

        # 4. Recomendar acciones
        actions = self.recommend_actions(risk_level, risks, climate_data)

        result = OrchestratorResult(
            risk_level=risk_level.value,
            summary=summary,
            predictions=predictions,
            actions=actions,
            climate_data=climate_data,
        )
        log.info(f"Análisis completado: Riesgo={risk_level.value} | Acciones={len(actions)}")
        return result

    def _generate_predictions(self, data: ClimateData, risks: List[EventType], level: RiskLevel) -> List[Prediction]:
        """Genera predicciones detalladas basadas en datos y riesgos."""
        preds: List[Prediction] = []

        # Predicción de ola de calor
        if EventType.HEATWAVE in risks:
            max_temp = max(data.temperature_c, max((f.temp_c for f in data.forecast[:12]), default=0))
            if max_temp >= 45:
                conf = 0.95
                msg = f"Ola de calor extremo: Temperatura máxima esperada {max_temp:.1f}°C"
            else:
                conf = 0.80
                msg = f"Ola de calor moderada: Máxima {max_temp:.1f}°C"
            preds.append(Prediction(
                event_type=EventType.HEATWAVE.value,
                confidence=conf,
                details=msg,
                timeframe_min=60 * 6,  # 6 horas
            ))

        # Predicción de apagón
        if EventType.BLACKOUT_RISK in risks or data.grid_load_percent >= 0.80:
            conf = 0.75 if data.grid_load_percent >= 0.90 else 0.60
            msg = f"Carga de red {data.grid_load_percent*100:.0f}% — posible inestabilidad en la red"
            preds.append(Prediction(
                event_type=EventType.BLACKOUT_RISK.value,
                confidence=conf,
                details=msg,
                timeframe_min=120,
            ))

        # Predicción de sobrecarga de CPU
        if EventType.CPU_OVERLOAD in risks:
            preds.append(Prediction(
                event_type=EventType.CPU_OVERLOAD.value,
                confidence=0.90,
                details=f"Uso de CPU en {data.cpu_load_percent*100:.1f}% — proceso consumo elevado",
                timeframe_min=30,
            ))

        return preds

    def execute_action(self, action_id: str, dry_run: Optional[bool] = None) -> dict:
        """Ejecuta una acción específica por ID."""
        action = self.registry.get(action_id)
        if not action:
            return {"success": False, "error": f"Action '{action_id}' not found"}

        dry = dry_run if dry_run is not None else self.dry_run
        result = self.bridge.execute(action, dry_run=dry)
        return {"success": True, "action": action_id, "dry_run": dry, "result": result}

    def execute_all(self, actions: List[Action], dry_run: Optional[bool] = None) -> dict:
        """Ejecuta una lista de acciones en orden."""
        results = []
        dry = dry_run if dry_run is not None else self.dry_run
        for action in actions:
            res = self.bridge.execute(action, dry_run=dry)
            results.append({"action": action.id, "success": res.get("success", False), "output": res.get("output", "")})
        return {"executed": len(results), "dry_run": dry, "results": results}
