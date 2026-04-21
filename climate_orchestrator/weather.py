"""
climate_orchestrator/weather.py — Cliente de APIs meteorológicas
Soporta: OpenWeatherMap (principal), NOAA (opcional),.nasa (opcional)
"""
from __future__ import annotations

import json
import logging
import os
from dataclasses import dataclass
from datetime import datetime, timezone
from typing import List, Optional

import requests

log = logging.getLogger("qeos.climate.weather")

@dataclass
class WeatherData:
    """Datapoint meteorológico estandarizado."""
    temp_c: float
    humidity: float
    wind_kph: float
    pressure_hpa: float
    condition: str
    timestamp: str
    rain_1h_mm: float = 0.0
    heat_index_c: Optional[float] = None

    @classmethod
    def from_owm(cls, data: dict) -> "WeatherData":
        """Convierte respuesta de OpenWeatherMap a WeatherData."""
        main = data.get("main", {})
        wind = data.get("wind", {})
        rain = data.get("rain", {})
        return cls(
            temp_c=main.get("temp", 20.0) - 273.15 if data.get("units") == "metric" else main.get("temp", 20.0),
            humidity=main.get("humidity", 50.0),
            wind_kph=wind.get("speed", 0.0) * 3.6,  # m/s → kph
            pressure_hpa=main.get("pressure", 1013.0),
            condition=data.get("weather", [{}])[0].get("description", "unknown"),
            timestamp=datetime.fromtimestamp(data.get("dt", 0), tz=timezone.utc).isoformat(),
            rain_1h_mm=rain.get("1h", 0.0),
        )

class WeatherClient:
    """Cliente unificado para múltiples fuentes meteorológicas."""

    def __init__(self, api_key: Optional[str] = None, dry_run: bool = True):
        self.api_key = api_key or os.getenv("OPENWEATHER_API_KEY")
        self.dry_run = dry_run
        self.base_url = "https://api.openweathermap.org/data/2.5"
        self.session = requests.Session()
        if not self.dry_run and not self.api_key:
            log.warning("OPENWEATHER_API_KEY no configurada — usando modo simulación")
            self.dry_run = True

    def get_current(self, location: str) -> WeatherData:
        """Obtiene clima actual para una ubicación."""
        if self.dry_run or not self.api_key:
            return self._simulate_current(location)
        url = f"{self.base_url}/weather"
        params = {"q": location, "appid": self.api_key, "units": "metric"}
        resp = self.session.get(url, params=params, timeout=10)
        resp.raise_for_status()
        return WeatherData.from_owm(resp.json())

    def get_forecast(self, location: str, hours: int = 48) -> List[WeatherData]:
        """Obtiene pronóstico por horas."""
        if self.dry_run or not self.api_key:
            return self._simulate_forecast(location, hours)
        url = f"{self.base_url}/forecast"
        params = {"q": location, "appid": self.api_key, "units": "metric", "cnt": min(hours // 3, 40)}
        resp = self.session.get(url, params=params, timeout=10)
        resp.raise_for_status()
        data = resp.json().get("list", [])
        return [WeatherData.from_owm(item) for item in data]

    def _simulate_current(self, location: str) -> WeatherData:
        """Simulación realista para Mexicali (sin API key)."""
        log.info(f"Simulando clima actual para {location}")
        # Datos basados en climatología de Mexicali (verano)
        now = datetime.now(timezone.utc)
        hour = now.hour
        # Simula ciclo diurno: mínima 4am (~25°C), máxima 4pm (~45°C)
        base_temp = 25 + 20 * (1 - abs(hour - 16) / 12)  # Campana centrada en 16:00
        return WeatherData(
            temp_c=base_temp + (2 * (now.minute % 5 - 2)),  # jitter pequeño
            humidity=30.0 + (15 * (1 - abs(hour - 16) / 12)),
            wind_kph=12.0 + (5 * (now.minute % 3)),
            pressure_hpa=1012.0,
            condition="clear sky" if base_temp > 30 else "few clouds",
            timestamp=now.isoformat(),
        )

    def _simulate_forecast(self, location: str, hours: int) -> List[WeatherData]:
        """Genera pronóstico simulado consistente con current."""
        current = self._simulate_current(location)
        forecast: List[WeatherData] = []
        now = datetime.now(timezone.utc)
        for i in range(hours):
            ts = now.timestamp() + i * 3600
            dt = datetime.fromtimestamp(ts, tz=timezone.utc)
            # Temperaturas suben hasta las 4pm, luego bajan
            hour_norm = (dt.hour - 6) % 24  # 0 = 6am
            base_temp = 25 + 18 * (1 - abs(hour_norm - 10) / 10) if 6 <= dt.hour <= 20 else 27
            pred = WeatherData(
                temp_c=base_temp,
                humidity=35.0,
                wind_kph=10.0,
                pressure_hpa=1013.0,
                condition="clear sky",
                timestamp=dt.isoformat(),
            )
            forecast.append(pred)
        return forecast
