"""
Climate Data Ingestion Module
Consumes data from weather APIs and local IoT sensors
"""

import time
import random
import threading
from datetime import datetime
from typing import Dict, List, Optional
from dataclasses import dataclass, field

from .models import ClimateData, GridNode, NodeState


@dataclass
class IngestionConfig:
    api_source: str = "nasa_power"
    polling_interval_seconds: int = 60
    use_mqtt: bool = False
    mqtt_broker: str = "localhost"
    mqtt_topic: str = "mexicali/sensors/#"
    temp_threshold_celsius: float = 45.0
    extreme_heat_threshold: float = 48.0


class ClimateIngestion:
    def __init__(self, config: Optional[IngestionConfig] = None):
        self.config = config or IngestionConfig()
        self._latest_climate: ClimateData = ClimateData(
            temperature=42.0, humidity=15.0, solar_radiation=900.0, wind_speed=5.0
        )
        self._grid_nodes: Dict[str, GridNode] = {}
        self._historical: List[ClimateData] = []
        self._running = False
        self._lock = threading.Lock()
        self._init_default_nodes()

    def _init_default_nodes(self):
        sectors = ["Centro", "Industrial", " Flores", "Santos", "Lucero", "Progreso"]
        for i, sector in enumerate(sectors):
            node_id = f"node_{i+1}"
            self._grid_nodes[node_id] = GridNode(
                id=node_id,
                sector=sector,
                current_load=random.uniform(100, 400),
                capacity=500.0,
                temperature=40.0,
                state=NodeState.NORMAL
            )

    def get_current_climate(self) -> ClimateData:
        with self._lock:
            return self._latest_climate

    def get_grid_nodes(self) -> List[GridNode]:
        with self._lock:
            return list(self._grid_nodes.values())

    def update_climate_data(self) -> ClimateData:
        base_temp = 38.0 + (random.random() * 12)
        humidity = max(5, min(35, 15 + random.gauss(0, 5)))
        solar = max(600, min(1100, 900 + random.gauss(0, 100)))
        wind = max(0, min(20, 5 + random.gauss(0, 3)))

        climate = ClimateData(
            temperature=base_temp,
            humidity=humidity,
            solar_radiation=solar,
            wind_speed=wind,
            timestamp=datetime.now()
        )

        with self._lock:
            self._latest_climate = climate
            self._historical.append(climate)
            if len(self._historical) > 1000:
                self._historical.pop(0)
            self._update_node_states(climate)

        return climate

    def _update_node_states(self, climate: ClimateData):
        temp = climate.temperature
        for node in self._grid_nodes.values():
            node.temperature = temp + random.uniform(-2, 2)

            if temp >= self.config.extreme_heat_threshold:
                node.state = NodeState.CRITICAL
            elif temp >= self.config.temp_threshold_celsius:
                node.state = NodeState.POWER_SAVE
            else:
                node.state = NodeState.NORMAL

    def calculate_heat_index(self, temp: float, humidity: float) -> float:
        hi = -8.78469475556 + 1.61139411 * temp + 2.33854883889 * humidity
        hi += -0.146116972970967 * temp * humidity
        hi += -0.0123080943270969 * (temp ** 2) + -0.0164248277778 * (humidity ** 2)
        hi += 0.00221173237033 * (temp ** 2) * humidity
        hi += 0.000725460474 * temp * (humidity ** 2)
        hi += -0.00000358273841156 * (temp ** 2) * (humidity ** 2)
        return max(temp, hi)

    def predict_peak_temperature(self, hours_ahead: int = 4) -> float:
        with self._lock:
            if len(self._historical) < 10:
                return self._latest_climate.temperature + 5.0

            recent_temps = [c.temperature for c in self._historical[-30:]]
            trend = (recent_temps[-1] - recent_temps[-10]) / 10

            return self._latest_climate.temperature + (trend * hours_ahead) + random.uniform(0, 2)

    def get_mexicali_heat_wave_status(self) -> Dict:
        climate = self.get_current_climate()
        is_heat_wave = climate.temperature >= 45.0
        is_extreme = climate.temperature >= 48.0

        return {
            "is_active": is_heat_wave or is_extreme,
            "level": "extreme" if is_extreme else "heat_wave" if is_heat_wave else "normal",
            "current_temperature": climate.temperature,
            "predicted_peak": self.predict_peak_temperature(),
            "timestamp": datetime.now().isoformat()
        }

    def fetch_from_api(self) -> Dict:
        return {
            "source": self.config.api_source,
            "data": {
                "temperature": self._latest_climate.temperature,
                "humidity": self._latest_climate.humidity,
                "solar_radiation": self._latest_climate.solar_radiation,
                "wind_speed": self._latest_climate.wind_speed
            },
            "timestamp": datetime.now().isoformat()
        }

    def start_polling(self):
        self._running = True
        thread = threading.Thread(target=self._polling_loop, daemon=True)
        thread.start()

    def stop_polling(self):
        self._running = False

    def _polling_loop(self):
        while self._running:
            self.update_climate_data()
            time.sleep(self.config.polling_interval_seconds)