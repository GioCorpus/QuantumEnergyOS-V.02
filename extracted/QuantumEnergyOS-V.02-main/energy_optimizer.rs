// ═══════════════════════════════════════════════════════════════════════
//  energy_optimizer.rs — Optimizador Energético en Tiempo Real
//
//  Combina QAOA fotónico, datos de sensores de la red CFE,
//  predicción solar, y modelos clásicos para prevenir apagones
//  en el noroeste de México.
//
//  Pipeline:
//    Sensores → Predicción → QAOA → Acción → Monitoreo
//
//  Casos de uso reales:
//    - Verano Mexicali 50°C: pico de A/C 14:00-18:00 → redistribuir carga
//    - Tormenta solar: proteger transformadores críticos
//    - Falla en línea transmisión: rerouting automático en <500 ms
// ═══════════════════════════════════════════════════════════════════════

use std::collections::VecDeque;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

use serde::{Deserialize, Serialize};
use crate::photonic_core::{PhotonicCore, QuantumBackend, QuantumResult};

// ── Constantes de la red BC ───────────────────────────────────────────

/// Nodos de la red eléctrica de Baja California
pub const GRID_NODES: &[GridNodeInfo] = &[
    GridNodeInfo { id: 0, name: "Mexicali Centro",   lat: 32.6245, lon: -115.4523, zone: "Mexicali"  },
    GridNodeInfo { id: 1, name: "Mexicali Industrial",lat: 32.6800, lon: -115.3200, zone: "Mexicali"  },
    GridNodeInfo { id: 2, name: "Tijuana Norte",     lat: 32.5149, lon: -117.0382, zone: "Tijuana"   },
    GridNodeInfo { id: 3, name: "Tijuana Este",      lat: 32.4800, lon: -116.9500, zone: "Tijuana"   },
    GridNodeInfo { id: 4, name: "Ensenada",          lat: 31.8676, lon: -116.5960, zone: "Ensenada"  },
    GridNodeInfo { id: 5, name: "Tecate",            lat: 32.5735, lon: -116.6272, zone: "Tecate"    },
    GridNodeInfo { id: 6, name: "Rosarito",          lat: 32.3328, lon: -117.0647, zone: "Rosarito"  },
    GridNodeInfo { id: 7, name: "San Felipe",        lat: 31.0219, lon: -114.8471, zone: "San Felipe" },
];

pub const CAPACITY_KW: &[f64] = &[120_000.0, 80_000.0, 130_000.0, 90_000.0,
                                    65_000.0, 30_000.0, 35_000.0, 15_000.0];
pub const CRITICAL_THRESHOLD: f64 = 0.85; // 85% = alerta amarilla
pub const EMERGENCY_THRESHOLD: f64 = 0.95; // 95% = alerta roja

/// Temperatura del desierto de BC que dispara la demanda de A/C
pub const MEXICALI_SUMMER_TEMP_C: f64 = 48.0;

#[derive(Clone, Copy, Debug)]
pub struct GridNodeInfo {
    pub id:   usize,
    pub name: &'static str,
    pub lat:  f64,
    pub lon:  f64,
    pub zone: &'static str,
}

// ═══════════════════════════════════════════════════════════════════════
//  ESTADO DE LA RED EN TIEMPO REAL
// ═══════════════════════════════════════════════════════════════════════

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GridState {
    pub timestamp_unix: u64,
    pub nodes:          Vec<NodeState>,
    pub total_load_kw:  f64,
    pub total_cap_kw:   f64,
    pub load_factor:    f64,
    pub alert_level:    AlertLevel,
    pub solar_kp_index: f64,
    pub temperature_c:  f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeState {
    pub id:           usize,
    pub name:         String,
    pub load_kw:      f64,
    pub capacity_kw:  f64,
    pub load_pct:     f64,
    pub status:       NodeStatus,
    pub voltage_kv:   f64,
    pub frequency_hz: f64,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum NodeStatus {
    Normal,
    Warning,    // 85-95% capacidad
    Critical,   // >95% capacidad
    Overloaded, // >100% capacidad — apagón inminente
    Offline,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum AlertLevel {
    Green,   // Todo OK
    Yellow,  // Monitoreo activo
    Orange,  // Acción preventiva recomendada
    Red,     // Emergencia — riesgo de apagón
    Black,   // Apagón en progreso
}

impl GridState {
    pub fn from_sensor_data(loads: &[f64], temp_c: f64, kp_index: f64) -> Self {
        let nodes: Vec<NodeState> = loads.iter().enumerate().map(|(i, &load)| {
            let cap = CAPACITY_KW.get(i).copied().unwrap_or(50_000.0);
            let pct = load / cap * 100.0;
            let status = if pct >= 100.0 { NodeStatus::Overloaded }
                    else if pct >= 95.0  { NodeStatus::Critical    }
                    else if pct >= 85.0  { NodeStatus::Warning     }
                    else                 { NodeStatus::Normal       };
            NodeState {
                id:           i,
                name:         GRID_NODES.get(i).map(|n| n.name).unwrap_or("Nodo").into(),
                load_kw:      load,
                capacity_kw:  cap,
                load_pct:     (pct * 10.0).round() / 10.0,
                status,
                voltage_kv:   115.0 * (1.0 - (pct - 50.0).max(0.0) / 1000.0),
                frequency_hz: 60.0 * (1.0 - (pct - 90.0).max(0.0) / 10_000.0),
            }
        }).collect();

        let total_load: f64 = loads.iter().sum();
        let total_cap:  f64 = CAPACITY_KW.iter().take(loads.len()).sum();
        let lf = total_load / total_cap;

        let alert = if lf >= 1.0      { AlertLevel::Black  }
               else if lf >= 0.95     { AlertLevel::Red    }
               else if lf >= 0.90     { AlertLevel::Orange }
               else if lf >= 0.85 || kp_index >= 5.0 { AlertLevel::Yellow }
               else                   { AlertLevel::Green  };

        Self {
            timestamp_unix: unix_now(),
            nodes,
            total_load_kw:  total_load,
            total_cap_kw:   total_cap,
            load_factor:    (lf * 1000.0).round() / 1000.0,
            alert_level:    alert,
            solar_kp_index: kp_index,
            temperature_c:  temp_c,
        }
    }

    pub fn critical_nodes(&self) -> Vec<&NodeState> {
        self.nodes.iter().filter(|n|
            matches!(n.status, NodeStatus::Critical | NodeStatus::Overloaded)
        ).collect()
    }

    pub fn needs_qaoa(&self) -> bool {
        self.load_factor >= CRITICAL_THRESHOLD || self.solar_kp_index >= 4.0
    }
}

// ═══════════════════════════════════════════════════════════════════════
//  OPTIMIZADOR PRINCIPAL
// ═══════════════════════════════════════════════════════════════════════

pub struct EnergyOptimizer {
    pub core:        PhotonicCore,
    pub history:     VecDeque<GridState>,
    pub actions:     Vec<OptimizationAction>,
    pub stats:       OptimizerStats,
    last_qaoa:       Instant,
    qaoa_cooldown:   Duration,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OptimizationAction {
    pub timestamp:   u64,
    pub action_type: ActionType,
    pub target_node: usize,
    pub magnitude:   f64,
    pub reason:      String,
    pub energy_saved_kw: f64,
    pub executed:    bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ActionType {
    LoadRedistribution,  // Redirigir carga de nodo A a nodo B
    DemandResponse,      // Reducir demanda en zona (A/C scheduling)
    CapacitorBank,       // Activar banco de capacitores
    GridIsland,          // Aislar zona para proteger el resto
    SolarCurtailment,    // Reducir generación solar si hay exceso
    EmergencyBrake,      // Protocolo de emergencia completo
}

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct OptimizerStats {
    pub total_optimizations:   u64,
    pub blackouts_prevented:   u32,
    pub total_energy_saved_kw: f64,
    pub avg_response_ms:       f64,
    pub qaoa_invocations:      u32,
    pub ibm_backend_uses:      u32,
    pub uptime_pct:            f64,
}

impl EnergyOptimizer {
    pub fn new() -> Self {
        Self {
            core:          PhotonicCore::new(QuantumBackend::Auto),
            history:       VecDeque::with_capacity(1440), // 24h de historia
            actions:       Vec::new(),
            stats:         OptimizerStats::default(),
            last_qaoa:     Instant::now() - Duration::from_secs(3600),
            qaoa_cooldown: Duration::from_millis(100), // máximo 10 QAOA/s
        }
    }

    /// Ciclo principal de optimización — llamar cada 100 ms en producción
    pub fn optimize_cycle(&mut self, state: GridState) -> Vec<OptimizationAction> {
        let t_start = Instant::now();
        self.history.push_back(state.clone());
        if self.history.len() > 1440 { self.history.pop_front(); }

        let mut actions = Vec::new();

        // 1. Tormenta solar detectada
        if state.solar_kp_index >= 5.0 {
            actions.extend(self.handle_solar_storm(&state));
        }

        // 2. QAOA si hay carga crítica y el cooldown expiró
        if state.needs_qaoa() && self.last_qaoa.elapsed() >= self.qaoa_cooldown {
            let qaoa_actions = self.run_qaoa_optimization(&state);
            actions.extend(qaoa_actions);
            self.last_qaoa = Instant::now();
            self.stats.qaoa_invocations += 1;
        }

        // 3. Acciones locales inmediatas (sin latencia cuántica)
        actions.extend(self.local_fast_actions(&state));

        // Actualizar estadísticas
        let latency_ms = t_start.elapsed().as_secs_f64() * 1000.0;
        self.stats.total_optimizations += 1;
        let n = self.stats.total_optimizations as f64;
        self.stats.avg_response_ms =
            (self.stats.avg_response_ms * (n - 1.0) + latency_ms) / n;

        if state.alert_level == AlertLevel::Black {
            // Apagón en progreso — registrar para prevenir en el futuro
        } else if matches!(state.alert_level, AlertLevel::Red | AlertLevel::Orange)
               && !actions.is_empty() {
            self.stats.blackouts_prevented += 1;
        }

        let saved: f64 = actions.iter().map(|a| a.energy_saved_kw).sum();
        self.stats.total_energy_saved_kw += saved;

        self.actions.extend(actions.clone());
        actions
    }

    /// Ejecutar QAOA para redistribución óptima de carga
    fn run_qaoa_optimization(&mut self, state: &GridState) -> Vec<OptimizationAction> {
        let loads: Vec<f64>    = state.nodes.iter().map(|n| n.load_kw).collect();
        let caps:  Vec<f64>    = state.nodes.iter().map(|n| n.capacity_kw).collect();

        let result = self.core.run_qaoa_grid(&loads, &caps, 2);

        if !result.success { return vec![]; }

        let bits = match result.bitstring {
            Some(ref b) => b.clone(),
            None        => return vec![],
        };

        let energy_saved = result.energy_saved_kw(&loads, &caps);

        // Traducir bitstring QAOA a acciones concretas
        bits.iter().enumerate()
            .filter(|(i, &b)| b == 1 && *i < state.nodes.len())
            .filter(|(i, _)| {
                state.nodes[*i].load_pct >= CRITICAL_THRESHOLD * 100.0
            })
            .map(|(i, _)| OptimizationAction {
                timestamp:        unix_now(),
                action_type:      ActionType::LoadRedistribution,
                target_node:      i,
                magnitude:        state.nodes[i].load_kw * 0.15, // Redirigir 15%
                reason:           format!("QAOA: nodo {} al {:.1}% — redirigir carga",
                                          state.nodes[i].name, state.nodes[i].load_pct),
                energy_saved_kw:  energy_saved / bits.len() as f64,
                executed:         false,
            })
            .collect()
    }

    /// Acciones inmediatas sin necesitar QAOA (reglas clásicas)
    fn local_fast_actions(&self, state: &GridState) -> Vec<OptimizationAction> {
        let mut actions = Vec::new();

        for node in &state.nodes {
            if node.status == NodeStatus::Overloaded {
                actions.push(OptimizationAction {
                    timestamp:       unix_now(),
                    action_type:     ActionType::DemandResponse,
                    target_node:     node.id,
                    magnitude:       node.load_kw - node.capacity_kw,
                    reason:          format!("{} sobrecargado — demand response inmediato", node.name),
                    energy_saved_kw: (node.load_kw - node.capacity_kw) * 0.8,
                    executed:        false,
                });
            }

            // Verano BC: pico A/C 14:00-18:00
            if state.temperature_c >= MEXICALI_SUMMER_TEMP_C && node.load_pct >= 80.0 {
                actions.push(OptimizationAction {
                    timestamp:       unix_now(),
                    action_type:     ActionType::CapacitorBank,
                    target_node:     node.id,
                    magnitude:       node.capacity_kw * 0.05,
                    reason:          format!("Calor extremo {:.0}°C — activar banco capacitores en {}",
                                             state.temperature_c, node.name),
                    energy_saved_kw: node.capacity_kw * 0.03,
                    executed:        false,
                });
            }
        }
        actions
    }

    /// Protocolo para tormentas solares (Kp ≥ 5)
    fn handle_solar_storm(&self, state: &GridState) -> Vec<OptimizationAction> {
        let kp = state.solar_kp_index;
        let severity = if kp >= 8.0 { "EXTREMA" } else if kp >= 6.0 { "SEVERA" } else { "MODERADA" };

        state.nodes.iter()
            .filter(|n| n.load_pct >= 70.0) // Proteger nodos con alta carga
            .map(|n| OptimizationAction {
                timestamp:       unix_now(),
                action_type:     if kp >= 7.0 { ActionType::GridIsland }
                                 else         { ActionType::LoadRedistribution },
                target_node:     n.id,
                magnitude:       n.load_kw * (kp / 9.0) * 0.2,
                reason:          format!("☀️  Tormenta solar {} (Kp={:.1}) — proteger {}",
                                         severity, kp, n.name),
                energy_saved_kw: n.load_kw * 0.1,
                executed:        false,
            })
            .collect()
    }

    /// Predecir estado de la red en las próximas N horas usando historial
    pub fn predict_next_hours(&self, hours: usize) -> Vec<GridPrediction> {
        if self.history.len() < 2 { return vec![]; }

        let recent: Vec<&GridState> = self.history.iter().rev().take(60).collect();
        let avg_load: f64 = recent.iter().map(|s| s.total_load_kw).sum::<f64>()
                          / recent.len() as f64;
        let trend: f64 = if recent.len() >= 2 {
            (recent[0].total_load_kw - recent[recent.len()-1].total_load_kw)
            / recent.len() as f64
        } else { 0.0 };

        (1..=hours).map(|h| {
            let predicted_load = (avg_load + trend * h as f64).max(0.0);
            let total_cap: f64 = CAPACITY_KW.iter().sum();
            let lf = predicted_load / total_cap;
            GridPrediction {
                hours_from_now: h,
                predicted_load_kw: predicted_load,
                load_factor:       lf,
                risk_level:        if lf >= 0.95 { "HIGH" }
                                   else if lf >= 0.85 { "MEDIUM" }
                                   else { "LOW" },
                qaoa_recommended:  lf >= CRITICAL_THRESHOLD,
            }
        }).collect()
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct GridPrediction {
    pub hours_from_now:    usize,
    pub predicted_load_kw: f64,
    pub load_factor:       f64,
    pub risk_level:        &'static str,
    pub qaoa_recommended:  bool,
}

fn unix_now() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
}
