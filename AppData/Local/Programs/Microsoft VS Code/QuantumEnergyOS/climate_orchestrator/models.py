"""
Data models for Climate Orchestrator
"""

from dataclasses import dataclass, field
from typing import List, Optional
from datetime import datetime
from enum import Enum


class NodeState(Enum):
    NORMAL = "normal"
    POWER_SAVE = "power_save"
    CRITICAL = "critical"
    OFFLINE = "offline"


class RiskLevel(Enum):
    LOW = "low"
    MEDIUM = "medium"
    HIGH = "high"
    EXTREME = "extreme"


@dataclass
class ClimateData:
    temperature: float
    humidity: float
    solar_radiation: float
    wind_speed: float
    timestamp: datetime = field(default_factory=datetime.now)
    location: str = "mexicali"


@dataclass
class GridNode:
    id: str
    sector: str
    current_load: float
    capacity: float
    temperature: float
    state: NodeState = NodeState.NORMAL


@dataclass
class RiskAssessment:
    risk_score: float
    risk_level: RiskLevel
    affected_nodes: List[str]
    recommended_actions: List[str]
    predicted_peak_temp: float
    timestamp: datetime = field(default_factory=datetime.now)


@dataclass
class OptimizationResult:
    optimal_states: dict
    energy_savings: float
    risk_reduction: float
    quantum_solution_time_ms: float
    timestamp: datetime = field(default_factory=datetime.now)
