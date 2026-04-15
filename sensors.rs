// ═══════════════════════════════════════════════════════════════════════
//  sensors.rs — Sensores del dispositivo + Predicción solar
//  iOS: CoreMotion, CoreLocation  |  Android: SensorManager, BatteryManager
// ═══════════════════════════════════════════════════════════════════════

use serde::{Deserialize, Serialize};
use crate::{BatteryState, AmbientData, SolarForecast};

// ── Batería ──────────────────────────────────────────────────────────
pub async fn read_battery() -> Result<BatteryState, String> {
    // tauri-plugin-battery maneja iOS + Android automáticamente
    // En desktop: lee /sys/class/power_supply/ (Linux) o acpi (macOS)
    #[cfg(target_os = "android")]
    return read_battery_android().await;

    #[cfg(target_os = "ios")]
    return read_battery_ios().await;

    #[cfg(not(any(target_os = "android", target_os = "ios")))]
    return Ok(BatteryState {
        level_pct:     read_battery_desktop(),
        charging:      is_charging_desktop(),
        voltage_v:     None,
        temperature_c: None,
        health:        "Unknown".into(),
    });
}

#[cfg(not(any(target_os = "android", target_os = "ios")))]
fn read_battery_desktop() -> f64 {
    // Linux: /sys/class/power_supply/BAT0/capacity
    std::fs::read_to_string("/sys/class/power_supply/BAT0/capacity")
        .ok()
        .and_then(|s| s.trim().parse::<f64>().ok())
        .unwrap_or(100.0)
}

#[cfg(not(any(target_os = "android", target_os = "ios")))]
fn is_charging_desktop() -> bool {
    std::fs::read_to_string("/sys/class/power_supply/BAT0/status")
        .map(|s| s.trim() == "Charging")
        .unwrap_or(true)
}

// ── Sensores ambientales ─────────────────────────────────────────────
pub async fn read_ambient() -> Result<AmbientData, String> {
    // Placeholder — en móvil real se conecta via JNI (Android) o CoreMotion (iOS)
    Ok(AmbientData {
        temperature_c: None,
        humidity_pct:  None,
        pressure_hpa:  None,
        light_lux:     None,
        acceleration:  Some([0.0, 0.0, -9.81]), // Reposo
    })
}

// ═══════════════════════════════════════════════════════════════════════
//  Predicción solar — eventos que afectan la red eléctrica
// ═══════════════════════════════════════════════════════════════════════

pub struct SolarEventPredictor {
    // Historial de índices Kp (actividad geomagnética)
    kp_history: Vec<f64>,
}

impl SolarEventPredictor {
    pub fn new() -> Self {
        // Valores Kp históricos simulados (0-9, donde 5+ = tormenta)
        Self {
            kp_history: vec![1.0, 1.3, 2.1, 1.8, 2.5, 3.0, 2.2, 1.9, 2.8, 3.5],
        }
    }

    /// Predice actividad solar y su impacto en la red eléctrica.
    /// El noroeste de México (Mexicali, Sonora) es vulnerable a tormentas
    /// solares que pueden sobrecargar líneas de alta tensión.
    pub fn forecast(&self, lat: f64, lon: f64) -> Result<SolarForecast, String> {
        // Modelo simplificado: tendencia de los últimos 10 índices Kp
        let recent_avg: f64 = self.kp_history.iter().rev().take(5).sum::<f64>() / 5.0;
        let trend: f64 = self.kp_history.windows(2)
            .map(|w| w[1] - w[0])
            .sum::<f64>() / self.kp_history.len() as f64;

        let predicted_kp = (recent_avg + trend * 3.0).clamp(0.0, 9.0);

        // Factor geográfico: latitudes altas son más vulnerables
        let geo_factor = 1.0 - (lat.abs() / 90.0) * 0.5;
        let effective_kp = predicted_kp * geo_factor;

        let (risk_level, alert_message, grid_impact) = match effective_kp as u32 {
            0..=2 => ("LOW",     "Actividad solar tranquila. Red eléctrica estable.", 0.02),
            3..=4 => ("MEDIUM",  "Tormenta geomagnética menor. Monitorear la red.", 0.08),
            5..=6 => ("HIGH",    "⚠️ Tormenta solar moderada. Activar protección QAOA.", 0.18),
            7..=8 => ("EXTREME", "🚨 Tormenta solar severa. Riesgo para transformadores.", 0.35),
            _     => ("EXTREME", "🚨 TORMENTA SOLAR EXTREMA. Desconectar sistemas sensibles.", 0.60),
        };

        let next_event = if trend > 0.3 {
            Some(24.0 / trend)   // Horas estimadas hasta el próximo evento
        } else {
            None
        };

        let recommendation = match risk_level {
            "LOW"     => "Sin acción necesaria. Sistema cuántico en standby.",
            "MEDIUM"  => "Ejecutar QAOA preventivo. Revisar nodos de alta tensión.",
            "HIGH"    => "Activar protección topológica. Redistribuir carga via QAOA.",
            "EXTREME" => "PROTOCOLO DE EMERGENCIA: Activar Cuarzo 4D, aislar nodos críticos.",
            _         => "Protocolo de emergencia activo.",
        };

        Ok(SolarForecast {
            location:         format!("Lat {:.2}°, Lon {:.2}°", lat, lon),
            risk_level:       risk_level.into(),
            kp_index:         effective_kp,
            alert_message:    alert_message.into(),
            grid_impact_pct:  grid_impact * 100.0,
            recommendation:   recommendation.into(),
            next_event_hours: next_event,
        })
    }
}
