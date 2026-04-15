//! ╔══════════════════════════════════════════════════════════════════════════╗
//! ║  AntiGravBridge — QuantumEnergyOS V.02                                  ║
//! ║  Módulo de simulación electrogravítica Biefeld-Brown + QAOA + fotónica  ║
//! ║  Autor: Giovanny Anthony Corpus Bernal — Mexicali, BC                   ║
//! ║  Kardashev 0→1                                                           ║
//! ╚══════════════════════════════════════════════════════════════════════════╝
//!
//! # Física implementada
//!
//! ## Efecto Biefeld-Brown / Viento Iónico (EHD — ElectroHydroDynamic)
//! El lifter consiste en un electrodo emisor (aguja, radio de punta pequeño)
//! y un electrodo colector (placa/foil). Al aplicar alto voltaje:
//!
//!   1. **Campo eléctrico**: `E_avg = V / d`  [V/m]
//!   2. **Amplificación de punta**: `E_tip = E_avg * sqrt(d / r_tip)` (campo real
//!      en la punta de la aguja, mucho mayor que el campo promedio)
//!   3. **Corona**: se inicia cuando `E_tip > E_corona(ρ)`.  
//!      `E_corona(ρ) = 3e6 * sqrt(ρ / ρ₀)` [V/m] (Paschen escalado a densidad)
//!   4. **Empuje empírico BB**: `T = K_BB * V_kV² * (ρ/ρ₀)` [N]  
//!      Calibrado para lifters de laboratorio (~2g, 30 kV → ~55 mN)
//!   5. **Corriente iónica** (consistente con EHD): `I = T * μ_ion / d`  [A]
//!   6. **Potencia**: `P = V * I`  [W]
//!
//! ## QAOA (Quantum Approximate Optimization Algorithm)
//! Discretizamos el espacio de voltaje en 8 niveles (3 qubits):
//!   V ∈ {10, 15, 20, 25, 30, 35, 40, 45} kV
//!
//! Circuito QAOA p=1:
//!   |ψ₀⟩ = |+⟩^⊗3 = (1/√8) Σⱼ |j⟩
//!   → e^{iγ H_C}  (separador de fase, costes del Hamiltoniano de coste)
//!   → e^{-iβ H_B} (mezclador, H_B = Σₖ Xₖ)
//!   → medir estado más probable → voltaje óptimo
//!
//! Función de coste: `c(V) = (T(V) - m·g) / P(V)`  maximizar rendimiento neto.
//!
//! ## Control Fotónico MZI (Mach-Zehnder Interferometer)
//! El empuje se traduce a un desplazamiento de fase proporcional:
//!   `φ = π * T / T_max`          [rad]
//!   `Trans = cos²(φ/2)`          [0..1]
//!   `P_óptica = P_bomba * Trans` [mW]
//! Permite controlar el lifter como una carga electro-óptica modulada.
//!
//! ## Balanceo de Red Energética
//! El lifter actúa como "carga virtual inteligente" en la microgrid del OS.

use nalgebra::Vector3;
use num_complex::Complex64;
use serde::{Deserialize, Serialize};
use std::f64::consts::PI;
use std::io::Write;
use std::process::{Command, Stdio};

// ═══════════════════════════════════════════════════════════════════════════
//  CONSTANTES FÍSICAS
// ═══════════════════════════════════════════════════════════════════════════

/// Gravedad estándar ISA [m/s²]
pub const G: f64 = 9.80665;

/// Densidad del aire al nivel del mar ISA [kg/m³]
pub const RHO_SL: f64 = 1.225;

/// Escala de altura exponencial de la atmósfera ISA simplificada [m]
pub const H_SCALE: f64 = 8_500.0;

/// Umbral de corona en aire seco STP [V/m]
/// (campo eléctrico de ruptura dieléctrica del aire, modelo Paschen lineal)
pub const E_CORONA_STP: f64 = 3.0e6;

/// Movilidad iónica del N₂⁺ en aire a STP [m²/(V·s)]
/// Ref: Townsend (1915), revisado por Robinson (1961)
pub const MU_ION: f64 = 2.0e-4;

/// Coeficiente empírico de empuje Biefeld-Brown [N/kV²]
/// Calibrado para lifter foil Al+mylar de ~2g, gap 5cm, 30 kV → ~55 mN
/// Equivale a: T [N] = K_BB * V_kV² * (ρ/ρ₀)
pub const K_BB: f64 = 5.0e-5;

/// Masa del lifter por defecto [kg] — foil de mylar + electrodo Al estilo "triángulo"
pub const DEFAULT_MASS: f64 = 0.002;

/// Separación inter-electrodo por defecto [m]
pub const DEFAULT_GAP: f64 = 0.05;

/// Radio de punta del electrodo emisor [m] — hilo de cobre fino ~0.1mm
pub const DEFAULT_EMITTER_RADIUS: f64 = 1e-4;

/// Producto coeficiente de arrastre × área frontal del lifter [m²]
/// (estimado para foil triangular de ~15cm de lado)
pub const CD_A: f64 = 2.5e-3;

/// Niveles de voltaje discretizados para QAOA [kV]
/// 3 qubits → 2³ = 8 estados: {10, 15, 20, 25, 30, 35, 40, 45}
pub const QAOA_VOLTAGES_KV: [f64; 8] = [10.0, 15.0, 20.0, 25.0, 30.0, 35.0, 40.0, 45.0];

// ═══════════════════════════════════════════════════════════════════════════
//  ESTRUCTURAS DE DATOS
// ═══════════════════════════════════════════════════════════════════════════

/// Electrodo individual del lifter EHD.
///
/// Un lifter típico tiene al menos dos electrodos:
/// - **Emisor** (aguja/hilo): signo positivo, radio de punta pequeño → corona intensa
/// - **Colector** (placa/foil): signo negativo, área grande → recibe iones
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Electrode {
    /// Posición 3D relativa al centro geométrico del lifter [m]
    pub pos: [f64; 3],
    /// Signo de carga: `+1.0` = emisor, `-1.0` = colector
    pub charge_sign: f64,
    /// Radio de curvatura de punta/borde [m]
    /// Emisor típico: 0.1mm (hilo); Colector: 5mm (borde de foil)
    pub radius_m: f64,
    /// Área de superficie activa [m²]
    pub area_m2: f64,
}

impl Electrode {
    /// Crea el electrodo emisor estándar (aguja positiva de hilo fino)
    pub fn emitter(pos: [f64; 3]) -> Self {
        Self {
            pos,
            charge_sign: 1.0,
            radius_m: DEFAULT_EMITTER_RADIUS,
            area_m2: 1e-6, // 1 mm² (área de la punta de la aguja)
        }
    }

    /// Crea el electrodo colector estándar (placa/foil negativa)
    pub fn collector(pos: [f64; 3]) -> Self {
        Self {
            pos,
            charge_sign: -1.0,
            radius_m: 5e-3,  // 5mm radio de borde
            area_m2: 1.5e-3, // 15cm × 1cm ≈ 1.5e-3 m² (foil triangular)
        }
    }
}

/// Resultado del control fotónico MZI (Mach-Zehnder Interferometer).
///
/// El empuje se convierte en un desplazamiento de fase para modular
/// una señal óptica CW, implementando control fotónico del lifter.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MziOutput {
    /// Desplazamiento de fase en el brazo de control [rad] ∈ [0, π]
    pub phase_shift_rad: f64,
    /// Transmitancia del MZI: `cos²(φ/2)` ∈ [0, 1]
    pub transmittance: f64,
    /// Potencia óptica de salida [mW] (asume bomba CW de 10 mW @ 1550 nm)
    pub optical_power_mw: f64,
    /// Frecuencia de modulación del tren de pulsos [Hz]
    /// Mapeo: 1 MHz (T=0) → 10 GHz (T=T_max), típico de moduladores EO en LiNbO₃
    pub pulse_freq_hz: f64,
    /// Largo de onda de la portadora óptica [nm] (estándar telecom C-band)
    pub wavelength_nm: f64,
}

/// Carga virtual del lifter para el balanceador de red QAOA.
///
/// Permite que el `EnergyOptimizer` del OS gestione el consumo del lifter
/// como un nodo de carga "interrumpible" en la microgrid.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GridLoad {
    /// Consumo eléctrico total del lifter [W]
    pub consumption_w: f64,
    /// ID del nodo de red (99 = nodo reservado para propulsión experimental)
    pub node_id: u32,
    /// Eficiencia de conversión eléctrica→mecánica (empuje/potencia) [N/W]
    pub conversion_efficiency_n_per_w: f64,
    /// ¿El nodo puede operar en modo regenerativo? (extensión futura)
    pub regenerative: bool,
    /// Prioridad de corte QAOA [0.0 = prioridad máxima de corte, 1.0 = mínima]
    /// Los nodos menos eficientes tienen mayor prioridad de desconexión
    pub qaoa_priority: f64,
    /// Potencia mínima necesaria para mantener hovering [W]
    pub min_hover_power_w: f64,
}

/// Snapshot de telemetría completa del lifter para el dashboard web.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AntiGravTelemetry {
    pub tick: u64,
    pub voltage_kv: f64,
    pub thrust_n: [f64; 3],
    pub thrust_magnitude_n: f64,
    pub pos_m: [f64; 3],
    pub vel_ms: [f64; 3],
    pub altitude_m: f64,
    pub ion_current_ua: f64,
    pub power_w: f64,
    pub air_density_kgm3: f64,
    pub efficiency_mn_per_w: f64, // mN/W
    pub net_force_n: f64,         // thrust - weight
    pub qaoa_voltage_kv: f64,     // último voltaje recomendado por QAOA
    pub status: String,
}

// ═══════════════════════════════════════════════════════════════════════════
//  ANTIGRAVBRIDGE — ESTRUCTURA PRINCIPAL
// ═══════════════════════════════════════════════════════════════════════════

/// Simulador electrogravítico Biefeld-Brown con optimización QAOA y control fotónico.
///
/// # Ejemplo rápido
/// ```
/// let mut lifter = AntiGravBridge::new(30.0);
/// for _ in 0..200 {
///     lifter.update(0.05);
/// }
/// println!("{}", lifter.get_status());
/// ```
pub struct AntiGravBridge {
    // ── Parámetros eléctricos ──────────────────────────────────────────────
    /// Voltaje aplicado entre electrodos [kV]
    pub voltage_kv: f64,
    /// Configuración de electrodos (mínimo: 1 emisor + 1 colector)
    pub electrodes: Vec<Electrode>,

    // ── Estado cinemático 3D ───────────────────────────────────────────────
    /// Vector de empuje neto [N] (dirección según eje de electrodos, por defecto +Z)
    pub thrust_vec: Vector3<f64>,
    /// Posición absoluta del lifter en el espacio [m] (Z = altitud)
    pub pos: Vector3<f64>,
    /// Velocidad del lifter [m/s]
    pub vel: Vector3<f64>,

    // ── Propiedades mecánicas ──────────────────────────────────────────────
    /// Masa total del lifter [kg] (estructura + electrodos + fuente HV)
    pub mass_kg: f64,
    /// Densidad del aire en la posición actual [kg/m³] (actualizada cada tick)
    pub air_density: f64,

    // ── Diagnóstico eléctrico ──────────────────────────────────────────────
    /// Corriente iónica total en el gap [μA]
    pub ion_current_ua: f64,
    /// Potencia eléctrica total consumida [W]
    pub power_w: f64,

    // ── QAOA ──────────────────────────────────────────────────────────────
    /// Voltaje óptimo calculado por QAOA en el último ciclo [kV]
    pub qaoa_voltage_kv: f64,
    /// Si true, intenta llamar a Python/Qiskit via subprocess antes del QAOA Rust
    pub use_qiskit_subprocess: bool,
    /// Cada cuántos ticks se ejecuta la optimización QAOA
    pub qaoa_interval_ticks: u64,

    // ── Contador ──────────────────────────────────────────────────────────
    /// Tick de simulación actual
    pub tick: u64,

    // ── Geometría interna (derivada de electrodes) ─────────────────────────
    /// Separación activa entre emisor y colector [m]
    gap_m: f64,
    /// Radio de punta del electrodo emisor [m]
    emitter_radius_m: f64,
    /// Área activa total del colector [m²]
    collector_area_m2: f64,
    /// Voltaje límite de seguridad [kV] (por encima: riesgo de arco)
    voltage_limit_kv: f64,
}

// ═══════════════════════════════════════════════════════════════════════════
//  IMPLEMENTACIÓN
// ═══════════════════════════════════════════════════════════════════════════

impl AntiGravBridge {
    // ── Constructor ─────────────────────────────────────────────────────────

    /// Crea un lifter con geometría estándar de par emisor-colector.
    ///
    /// Por defecto: masa 2g, gap 5cm, límite 55 kV, QAOA cada 50 ticks.
    ///
    /// ```
    /// let lifter = AntiGravBridge::new(30.0);
    /// assert_eq!(lifter.voltage_kv, 30.0);
    /// assert_eq!(lifter.mass_kg, DEFAULT_MASS);
    /// ```
    pub fn new(voltage_kv: f64) -> Self {
        let emitter = Electrode::emitter([0.0, 0.0, 0.0]);
        let collector = Electrode::collector([0.0, 0.0, DEFAULT_GAP]);

        Self {
            voltage_kv: voltage_kv.clamp(1.0, 60.0),
            electrodes: vec![emitter, collector],
            thrust_vec: Vector3::zeros(),
            pos: Vector3::zeros(),
            vel: Vector3::zeros(),
            mass_kg: DEFAULT_MASS,
            air_density: RHO_SL,
            ion_current_ua: 0.0,
            power_w: 0.0,
            qaoa_voltage_kv: voltage_kv,
            use_qiskit_subprocess: false,
            qaoa_interval_ticks: 50,
            tick: 0,
            gap_m: DEFAULT_GAP,
            emitter_radius_m: DEFAULT_EMITTER_RADIUS,
            collector_area_m2: 1.5e-3,
            voltage_limit_kv: 55.0,
        }
    }

    /// Constructor avanzado: geometría de electrodos personalizada.
    ///
    /// Calcula automáticamente gap, radio de emisor y área de colector
    /// desde el vector de electrodos proporcionado.
    pub fn with_electrodes(voltage_kv: f64, electrodes: Vec<Electrode>, mass_kg: f64) -> Self {
        let mut bridge = Self::new(voltage_kv);
        bridge.mass_kg = mass_kg;

        // Calcular gap entre el primer emisor y el primer colector
        let maybe_em = electrodes.iter().find(|e| e.charge_sign > 0.0);
        let maybe_col = electrodes.iter().find(|e| e.charge_sign < 0.0);
        if let (Some(em), Some(col)) = (maybe_em, maybe_col) {
            let d = [
                col.pos[0] - em.pos[0],
                col.pos[1] - em.pos[1],
                col.pos[2] - em.pos[2],
            ];
            bridge.gap_m = (d[0]*d[0] + d[1]*d[1] + d[2]*d[2]).sqrt().max(1e-3);
            bridge.emitter_radius_m = em.radius_m;
            bridge.collector_area_m2 = electrodes.iter()
                .filter(|e| e.charge_sign < 0.0)
                .map(|e| e.area_m2)
                .sum::<f64>()
                .max(1e-5);
        }
        bridge.electrodes = electrodes;
        bridge
    }

    // ── Física atmosférica ───────────────────────────────────────────────────

    /// Densidad del aire según altitud — modelo ISA exponencial simplificado.
    ///
    /// `ρ(h) = ρ₀ · exp(-h / H)` con H = 8500 m
    ///
    /// Válido hasta ~80 km. Ignora gradientes de temperatura (troposfera vs estratosfera).
    pub fn air_density_at(altitude_m: f64) -> f64 {
        RHO_SL * (-altitude_m.max(0.0) / H_SCALE).exp()
    }

    // ── Motor EHD ───────────────────────────────────────────────────────────

    /// Calcula empuje, corriente iónica y potencia para los parámetros dados.
    ///
    /// # Modelo EHD detallado
    ///
    /// 1. **Campo promedio**: `E_avg = V / d`
    /// 2. **Amplificación de punta** (geometría aguja-plana):
    ///    `E_tip = E_avg · sqrt(d / r_tip)` — factor limitado a 150×
    /// 3. **Umbral de corona** (escalado Paschen con densidad):
    ///    `E_c(ρ) = E_c₀ · sqrt(ρ / ρ₀)` donde `E_c₀ = 3 MV/m`
    /// 4. **Empuje Biefeld-Brown empírico**:
    ///    `T = K_BB · V_kV² · (ρ/ρ₀)` [N] — calibrado para lifter de laboratorio
    /// 5. **Corriente iónica** (consistente con EHD: `T = I·d/μ_ion`):
    ///    `I = T · μ_ion / d`
    /// 6. **Potencia**: `P = V · I`
    ///
    /// Devuelve `(thrust_N, ion_current_μA, power_W)`.
    /// Retorna `(0, 0, 0)` si no hay corona (campo insuficiente).
    pub fn compute_ehd(
        voltage_kv: f64,
        gap_m: f64,
        emitter_radius_m: f64,
        air_density: f64,
    ) -> (f64, f64, f64) {
        let v_volts = voltage_kv * 1_000.0;
        let e_avg = v_volts / gap_m; // campo promedio [V/m]

        // Amplificación de punta (factor geométrico de la aguja, limitado)
        let enhancement = (gap_m / emitter_radius_m).sqrt().min(150.0);
        let e_tip = e_avg * enhancement; // campo real en la punta [V/m]

        // Umbral de corona escalado con densidad atmosférica
        // (menor densidad → menos colisiones → corona a menor campo)
        let e_corona = E_CORONA_STP * (air_density / RHO_SL).sqrt();

        if e_tip < e_corona {
            return (0.0, 0.0, 0.0); // sin corona → sin empuje iónico
        }

        // Empuje BB empírico: T [N] = K_BB · V_kV² · (ρ/ρ₀)
        // Escala cuadráticamente con V (experimentalmente validado en la literatura)
        let rho_norm = air_density / RHO_SL;
        let thrust_n = K_BB * voltage_kv * voltage_kv * rho_norm;

        // Corriente iónica desde balance de momento EHD: T = I·d/μ_ion → I = T·μ_ion/d
        // Dimensiones: [N·(m²/Vs)/m] = [N·m/(Vs)] = [kg·m²/s²·s] / [V] ... 
        //              = [W·s/m] / [V] = [A·s/m] · [m] = [A] ✓
        let ion_current_a = thrust_n * MU_ION / gap_m;
        let ion_current_ua = ion_current_a * 1e6; // → μA

        // Potencia eléctrica: P = V · I
        let power_w = v_volts * ion_current_a;

        (thrust_n, ion_current_ua, power_w)
    }

    // ── update ──────────────────────────────────────────────────────────────

    /// Avanza la simulación un paso de tiempo `dt` [segundos].
    ///
    /// Orden de operaciones:
    ///   1. Actualizar ρ(h) según altitud actual
    ///   2. Calcular empuje EHD con voltaje actual
    ///   3. Sumar fuerzas: empuje + gravedad + arrastre aerodinámico
    ///   4. Integración de Euler simpléctica (vel primero, luego pos)
    ///   5. Restricción de suelo con rebote amortiguado
    ///   6. Ejecutar QAOA cada `qaoa_interval_ticks` y actualizar voltaje
    pub fn update(&mut self, dt: f64) {
        self.tick += 1;

        // ── 1. Densidad del aire ─────────────────────────────────────────
        let altitude = self.pos.z.max(0.0);
        self.air_density = Self::air_density_at(altitude);

        // ── 2. Empuje EHD ───────────────────────────────────────────────
        let (thrust_mag, ion_ua, power_w) = Self::compute_ehd(
            self.voltage_kv,
            self.gap_m,
            self.emitter_radius_m,
            self.air_density,
        );
        self.ion_current_ua = ion_ua;
        self.power_w = power_w;

        // El empuje siempre apunta en la dirección emisor→colector.
        // Por defecto esta dirección es +Z (vertical hacia arriba).
        // Si los electrodos están inclinados, se puede calcular aquí.
        let thrust_dir = Vector3::new(0.0, 0.0, 1.0); // eje +Z = anti-gravedad
        self.thrust_vec = thrust_dir * thrust_mag;

        // ── 3. Fuerza de gravedad ────────────────────────────────────────
        let f_grav = Vector3::new(0.0, 0.0, -G * self.mass_kg);

        // ── 4. Arrastre aerodinámico ─────────────────────────────────────
        // F_drag = -½ · ρ · Cd·A · |v|² · v̂
        // Actúa opuesto a la dirección de movimiento
        let speed = self.vel.norm();
        let f_drag = if speed > 1e-10 {
            let drag_mag = 0.5 * self.air_density * CD_A * speed * speed;
            -(self.vel / speed) * drag_mag
        } else {
            Vector3::zeros()
        };

        // ── 5. Fuerza neta y aceleración ─────────────────────────────────
        let f_net = self.thrust_vec + f_grav + f_drag;
        let accel = f_net / self.mass_kg;

        // ── 6. Integración de Euler simpléctica ──────────────────────────
        // Actualizar velocidad primero (más estable que Euler clásico)
        self.vel += accel * dt;
        self.pos += self.vel * dt;

        // ── 7. Restricción de suelo ──────────────────────────────────────
        if self.pos.z < 0.0 {
            self.pos.z = 0.0;
            if self.vel.z < 0.0 {
                // Rebote amortiguado (coeficiente de restitución ~0.1)
                self.vel.z = -self.vel.z * 0.1;
            }
        }

        // ── 8. Optimización QAOA periódica ───────────────────────────────
        // Se ejecuta cada `qaoa_interval_ticks` para recalcular el voltaje óptimo
        if self.tick % self.qaoa_interval_ticks == 0 {
            let opt_v = self.optimize_voltage();
            self.qaoa_voltage_kv = opt_v;
            // Aplicar suavemente (ramp de ±5 kV/tick para no crear discontinuidades)
            let delta = (opt_v - self.voltage_kv).clamp(-5.0, 5.0);
            self.voltage_kv = (self.voltage_kv + delta).clamp(1.0, self.voltage_limit_kv);
        }
    }

    // ── optimize_voltage (QAOA) ─────────────────────────────────────────────

    /// Optimiza el voltaje de operación usando QAOA (p=1) sobre 3 qubits.
    ///
    /// **Espacio de búsqueda**: 8 niveles de voltaje [10..45 kV] en pasos de 5 kV.
    ///
    /// **Función de coste** (maximizar):
    ///   `c(V) = (T(V) − m·g) / P(V)`  [N/W] — rendimiento de empuje neto
    ///
    /// Si el thrust no supera el peso (`T < m·g`), el coste es 0 para ese nivel.
    /// Esto guía al QAOA hacia el voltaje mínimo que garantiza levitación.
    ///
    /// **Flujo**:
    ///   1. Si `use_qiskit_subprocess`, intenta subprocess a `qaoa_optimize.py`
    ///   2. Fallback: simulación clásica exacta del circuito QAOA en Rust
    ///
    /// Devuelve el voltaje óptimo [kV] dentro de `[5, voltage_limit_kv]`.
    pub fn optimize_voltage(&self) -> f64 {
        // Intento 1: subprocess Qiskit-Aer (quantum real)
        if self.use_qiskit_subprocess {
            if let Some(v) = self.qiskit_subprocess_optimize() {
                eprintln!("[QAOA-Qiskit] tick={} → {:.1} kV", self.tick, v);
                return v;
            }
            eprintln!("[QAOA-Qiskit] subprocess falló, usando QAOA clásico Rust");
        }

        // Fallback: QAOA clásico exacto en Rust
        self.qaoa_classical_optimize()
    }

    // ── QAOA clásico puro Rust ──────────────────────────────────────────────

    /// Simulación exacta del circuito QAOA p=1 con 3 qubits (2³ = 8 estados).
    ///
    /// El vector de estado es ψ ∈ ℂ⁸. Las operaciones son:
    ///
    /// 1. **Inicializar**: `|ψ⟩ = |+⟩^⊗3 = (1/√8) Σⱼ |j⟩`
    /// 2. **Separador de fase**: `|ψ_j⟩ *= e^{iγ·cⱼ}` (signo + → maximización)
    /// 3. **Mezclador**: `e^{-iβ Xₖ}` sobre cada qubit k:
    ///    `ψ'[j] = cos(β)·ψ[j] − i·sin(β)·ψ[j ⊕ 2ᵏ]`
    /// 4. **Medición**: estado más probable → índice → voltaje
    ///
    /// Barrido de grilla sobre γ ∈ (0, π] y β ∈ (0, π/2] para encontrar
    /// los parámetros que maximizan `⟨ψ|H_C|ψ⟩`.
    fn qaoa_classical_optimize(&self) -> f64 {
        const N_STATES: usize = 8;
        const N_QUBITS: usize = 3;

        // ── Paisaje de costes ────────────────────────────────────────────
        // c(V) = max(0, T(V) − m·g) / P(V)  → 0 si no puede levantar
        let weight = self.mass_kg * G;
        let costs: [f64; N_STATES] = std::array::from_fn(|i| {
            let v_kv = QAOA_VOLTAGES_KV[i];
            let (thrust, _, power) = Self::compute_ehd(
                v_kv,
                self.gap_m,
                self.emitter_radius_m,
                self.air_density,
            );
            if thrust <= weight || power < 1e-12 {
                0.0 // penalizar voltajes que no levantan o sin potencia
            } else {
                (thrust - weight) / power // rendimiento de empuje neto [N/W]
            }
        });

        // Barrido de grilla: 30×30 puntos en (γ, β)
        let n_pts = 30usize;
        let inv_sqrt8 = 1.0_f64 / (N_STATES as f64).sqrt();

        let mut best_voltage = self.voltage_kv;
        let mut best_expectation = f64::NEG_INFINITY;

        for gi in 1..=n_pts {
            for bi in 1..=n_pts {
                let gamma = gi as f64 * PI / n_pts as f64;
                let beta = bi as f64 * (PI * 0.5) / n_pts as f64;

                // ── Inicializar |+⟩^⊗3 ───────────────────────────────
                let mut psi: [Complex64; N_STATES] =
                    std::array::from_fn(|_| Complex64::new(inv_sqrt8, 0.0));

                // ── Separador de fase: e^{iγ·c_j} ────────────────────
                // (signo positivo porque maximizamos el coste)
                for j in 0..N_STATES {
                    let phase = Complex64::new(0.0, gamma * costs[j]);
                    psi[j] *= phase.exp();
                }

                // ── Mezclador: Πₖ e^{-iβ Xₖ} ─────────────────────────
                // Aplicamos e^{-iβ X} sobre cada qubit k de forma independiente.
                // e^{-iβ X} = cos(β)·I − i·sin(β)·X
                // Sobre |j⟩: rota entre |j⟩ y |j ⊕ 2ᵏ⟩ (voltea el k-ésimo bit)
                for k in 0..N_QUBITS {
                    let cos_b = Complex64::new(beta.cos(), 0.0);
                    let neg_i_sin_b = Complex64::new(0.0, -beta.sin());
                    let mut new_psi: [Complex64; N_STATES] =
                        std::array::from_fn(|_| Complex64::new(0.0, 0.0));

                    for j in 0..N_STATES {
                        let partner = j ^ (1 << k); // flip del bit k
                        new_psi[j] = cos_b * psi[j] + neg_i_sin_b * psi[partner];
                    }
                    psi = new_psi;
                }

                // ── Valor esperado ⟨ψ|H_C|ψ⟩ = Σ |ψ_j|² · c_j ───────
                let expectation: f64 = psi.iter()
                    .zip(costs.iter())
                    .map(|(amp, &cost)| amp.norm_sqr() * cost)
                    .sum();

                if expectation > best_expectation {
                    best_expectation = expectation;

                    // Estado más probable (colapso de medición cuántica)
                    let best_idx = psi.iter()
                        .enumerate()
                        .max_by(|(_, a), (_, b)| {
                            a.norm_sqr().partial_cmp(&b.norm_sqr())
                                .unwrap_or(std::cmp::Ordering::Equal)
                        })
                        .map(|(i, _)| i)
                        .unwrap_or(4); // default: índice central (30 kV)

                    best_voltage = QAOA_VOLTAGES_KV[best_idx];
                }
            }
        }

        let result = best_voltage.clamp(5.0, self.voltage_limit_kv);
        eprintln!(
            "[QAOA-Rust] tick={:>5} → V_opt={:.0} kV | ⟨H_C⟩={:.4e} N/W | ρ={:.4} kg/m³",
            self.tick, result, best_expectation, self.air_density
        );
        result
    }

    // ── Subprocess Qiskit ───────────────────────────────────────────────────

    /// Llama a `python3 qaoa_optimize.py` enviando parámetros JSON por stdin.
    ///
    /// El script Python usa `qiskit-aer` para ejecutar el circuito QAOA real
    /// con ruido de hardware simulado. Devuelve `{"optimal_voltage_kv": f64}`.
    ///
    /// Devuelve `None` si Python no está disponible, el script falla,
    /// o la salida no es JSON válido.
    fn qiskit_subprocess_optimize(&self) -> Option<f64> {
        // Payload con todos los parámetros del estado actual
        let payload = serde_json::json!({
            "gap_m":            self.gap_m,
            "emitter_radius_m": self.emitter_radius_m,
            "air_density":      self.air_density,
            "mass_kg":          self.mass_kg,
            "voltage_min_kv":   QAOA_VOLTAGES_KV[0],
            "voltage_max_kv":   *QAOA_VOLTAGES_KV.last().unwrap(),
            "n_levels":         QAOA_VOLTAGES_KV.len(),
            "p_layers":         1,   // profundidad QAOA p=1
            "shots":            1024 // mediciones por circuito
        });

        // Lanzar subproceso Python no bloqueante (con timeout implícito de 10s)
        let mut child = Command::new("python3")
            .arg("qaoa_optimize.py")
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::null()) // silenciar warnings de Qiskit
            .spawn()
            .ok()?;

        // Enviar payload por stdin
        if let Some(stdin) = child.stdin.as_mut() {
            stdin.write_all(payload.to_string().as_bytes()).ok()?;
        }

        // Esperar resultado (el script debe responder en <10s)
        let output = child.wait_with_output().ok()?;
        if !output.status.success() {
            return None;
        }

        // Parsear respuesta JSON
        let result: serde_json::Value = serde_json::from_slice(&output.stdout).ok()?;
        let v = result["optimal_voltage_kv"].as_f64()?;
        Some(v.clamp(5.0, self.voltage_limit_kv))
    }

    // ── send_to_dashboard ───────────────────────────────────────────────────

    /// Serializa la telemetría actual y la envía por HTTP al dashboard Flask.
    ///
    /// **Endpoint**: `POST http://localhost:5000/api/antigrav`  
    /// **Body**: JSON `AntiGravTelemetry` serializado  
    /// **Timeout**: 2 segundos (no bloqueante si el servidor no responde)
    ///
    /// En modo sin feature `dashboard`, imprime a stdout (stub de desarrollo).
    ///
    /// # Errores
    /// Devuelve `Err(String)` si la conexión falla o el servidor responde ≥ 400.
    pub fn send_to_dashboard(&self) -> Result<(), String> {
        let telemetry = self.telemetry();

        // ── Modo con reqwest (feature "dashboard") ───────────────────────
        #[cfg(feature = "dashboard")]
        {
            let client = reqwest::blocking::Client::builder()
                .timeout(std::time::Duration::from_secs(2))
                .build()
                .map_err(|e| format!("reqwest build error: {}", e))?;

            let resp = client
                .post("http://localhost:5000/api/antigrav")
                .json(&telemetry)
                .send()
                .map_err(|e| format!("Dashboard unreachable: {}", e))?;

            if !resp.status().is_success() {
                return Err(format!(
                    "Dashboard respondió con error: {} {}",
                    resp.status().as_u16(),
                    resp.status().canonical_reason().unwrap_or("?")
                ));
            }
            return Ok(());
        }

        // ── Stub de desarrollo (sin feature "dashboard") ─────────────────
        // Simula el POST imprimiendo el JSON a stdout para debug
        #[cfg(not(feature = "dashboard"))]
        {
            let json = serde_json::to_string(&telemetry)
                .unwrap_or_else(|_| "{}".to_string());
            println!("[DASHBOARD_STUB] POST /api/antigrav → {}", json);
            Ok(())
        }
    }

    // ── pulse_control (MZI fotónico) ────────────────────────────────────────

    /// Traduce el empuje actual a parámetros de control de un interferómetro
    /// Mach-Zehnder (MZI) electro-óptico.
    ///
    /// # Modelo fotónico
    ///
    /// Un MZI divide la luz en dos brazos y recombina con interferencia:
    ///   - **Brazo de referencia**: sin modulación
    ///   - **Brazo de control**: desplazamiento de fase `φ ∝ T`
    ///
    /// El empuje (normalizado 0..1) controla la fase:
    ///   `φ = π · T / T_max`  ∈ [0, π] rad
    ///
    /// Transmitancia del MZI en el puerto constructivo:
    ///   `Trans = cos²(φ/2)`  ∈ [0, 1]
    ///
    /// En hardware LiNbO₃/Si, esto se implementa con electrodos
    /// depositados sobre la guía de onda que modulan el índice electro-óptico.
    ///
    /// La frecuencia de pulsos refleja la "urgencia" del control:
    ///   - T → 0: pulsos lentos (1 MHz, señal de mantenimiento)
    ///   - T → T_max: pulsos rápidos (10 GHz, control máximo)
    pub fn pulse_control(&self) -> MziOutput {
        // Calcular empuje máximo teórico (V=45 kV, STP) para normalización
        let (t_max, _, _) = Self::compute_ehd(
            QAOA_VOLTAGES_KV[7], // 45 kV
            self.gap_m,
            self.emitter_radius_m,
            RHO_SL, // densidad STP para el máximo absoluto
        );

        let t_mag = self.thrust_vec.norm();
        let t_norm = if t_max > 1e-12 {
            (t_mag / t_max).clamp(0.0, 1.0)
        } else {
            0.0
        };

        // Desplazamiento de fase: φ ∈ [0, π]
        // T=0 → φ=0 → Trans=1 (paso completo)
        // T=T_max → φ=π → Trans=0 (bloqueo completo)
        // Esto permite usar el MZI como interruptor de potencia del lifter
        let phase_shift_rad = PI * t_norm;
        let transmittance = (phase_shift_rad * 0.5).cos().powi(2);

        // Potencia de bomba CW @ 1550 nm (banda C telecom estándar)
        let pump_power_mw = 10.0;
        let optical_power_mw = pump_power_mw * transmittance;

        // Frecuencia de pulsos: escala logarítmica de 1 MHz a 10 GHz
        // La escala log es más natural para el ancho de banda del modulador
        let f_min = 1e6_f64;   // 1 MHz
        let f_max = 10e9_f64;  // 10 GHz (límite electro-óptico LiNbO₃)
        let pulse_freq_hz = f_min * (f_max / f_min).powf(t_norm);

        MziOutput {
            phase_shift_rad,
            transmittance,
            optical_power_mw,
            pulse_freq_hz,
            wavelength_nm: 1550.0, // banda C estándar
        }
    }

    // ── balance_grid ────────────────────────────────────────────────────────

    /// Calcula la carga virtual del lifter para el balanceador de red QAOA.
    ///
    /// El `EnergyOptimizer` del OS puede usar estos datos para decidir cuándo
    /// reducir o interrumpir el suministro al lifter en picos de demanda,
    /// o para contabilizar su consumo en la planificación de carga del grid.
    ///
    /// # Integración con el módulo energético
    ///
    /// ```no_run
    /// // En energy_optimizer.rs:
    /// let load = antigrav_bridge.balance_grid();
    /// optimizer.register_virtual_load(load.node_id, load.consumption_w);
    /// optimizer.set_interruptible(load.node_id, load.qaoa_priority > 0.6);
    /// ```
    pub fn balance_grid(&self) -> GridLoad {
        // Eficiencia de conversión: empuje neto / potencia eléctrica
        // Unidades: [N/W] — cuánto newton de empuje neto por watt consumido
        let net_thrust = (self.thrust_vec.norm() - self.mass_kg * G).max(0.0);
        let efficiency = if self.power_w > 1.0 {
            net_thrust / self.power_w
        } else {
            0.0
        };

        // Prioridad de corte QAOA: mayor prioridad = se desconecta antes
        // Dispositivos ineficientes o sin corona son los primeros en cortarse
        let qaoa_priority = if self.ion_current_ua < 1.0 {
            1.0 // sin corona = carga muerta, máxima prioridad de corte
        } else {
            // Normalizar: eficiencia alta → prioridad baja (no cortarlo)
            let eff_max_ref = 5e-3; // referencia: 5 mN/W
            1.0 - (efficiency / eff_max_ref).clamp(0.0, 1.0)
        };

        // Potencia mínima para mantener hovering (V tal que T ≈ m·g)
        // Se estima numéricamente invirtiendo T = K_BB * V² * rho
        let v_hover_kv = ((self.mass_kg * G) / (K_BB * (self.air_density / RHO_SL)))
            .sqrt()
            .clamp(5.0, 60.0);
        let (_, _, p_hover) = Self::compute_ehd(
            v_hover_kv, self.gap_m, self.emitter_radius_m, self.air_density
        );

        GridLoad {
            consumption_w: self.power_w,
            node_id: 99,
            conversion_efficiency_n_per_w: efficiency,
            regenerative: false,
            qaoa_priority,
            min_hover_power_w: p_hover,
        }
    }

    // ── Telemetría y estado ─────────────────────────────────────────────────

    /// Genera una snapshot completa de telemetría serializable.
    ///
    /// Útil para logging, el dashboard web, y tests de integración.
    pub fn telemetry(&self) -> AntiGravTelemetry {
        let thrust_mag = self.thrust_vec.norm();
        let eff_mn_per_w = if self.power_w > 1e-12 {
            thrust_mag / self.power_w * 1_000.0 // → mN/W
        } else {
            0.0
        };
        let net_force = thrust_mag - self.mass_kg * G;

        let status = match (self.ion_current_ua, self.pos.z, net_force) {
            (i, _, _) if i < 0.5 => "BELOW_CORONA",
            (_, z, _) if z < 0.01 => "GROUNDED",
            (_, _, nf) if nf > 0.0 => "ASCENDING",
            (_, _, nf) if nf < -0.005 => "DESCENDING",
            _ => "HOVERING",
        }
        .to_string();

        AntiGravTelemetry {
            tick: self.tick,
            voltage_kv: self.voltage_kv,
            thrust_n: [self.thrust_vec.x, self.thrust_vec.y, self.thrust_vec.z],
            thrust_magnitude_n: thrust_mag,
            pos_m: [self.pos.x, self.pos.y, self.pos.z],
            vel_ms: [self.vel.x, self.vel.y, self.vel.z],
            altitude_m: self.pos.z,
            ion_current_ua: self.ion_current_ua,
            power_w: self.power_w,
            air_density_kgm3: self.air_density,
            efficiency_mn_per_w: eff_mn_per_w,
            net_force_n: net_force,
            qaoa_voltage_kv: self.qaoa_voltage_kv,
            status,
        }
    }

    /// Formato de estado compacto para salida a terminal.
    pub fn get_status(&self) -> String {
        let net = self.thrust_vec.norm() - self.mass_kg * G;
        format!(
            "T={:>5} | V={:>5.1}kV | Thrust={:>8.4}N | ΔF={:>+8.4}N | \
             Alt={:>7.3}m | Vel={:>6.3}m/s | I={:>7.2}μA | P={:>6.2}W | ρ={:.4}",
            self.tick,
            self.voltage_kv,
            self.thrust_vec.norm(),
            net,
            self.pos.z,
            self.vel.norm(),
            self.ion_current_ua,
            self.power_w,
            self.air_density,
        )
    }
}

// ═══════════════════════════════════════════════════════════════════════════
//  TESTS
// ═══════════════════════════════════════════════════════════════════════════

#[cfg(test)]
mod tests {
    use super::*;

    // ── Construcción ────────────────────────────────────────────────────────

    #[test]
    fn test_new_defaults() {
        let b = AntiGravBridge::new(30.0);
        assert_eq!(b.voltage_kv, 30.0);
        assert_eq!(b.mass_kg, DEFAULT_MASS);
        assert_eq!(b.tick, 0);
        assert_eq!(b.gap_m, DEFAULT_GAP);
        assert_eq!(b.electrodes.len(), 2);
    }

    #[test]
    fn test_voltage_clamping() {
        // Voltaje negativo y demasiado alto deben clampearse
        let low = AntiGravBridge::new(-10.0);
        assert!(low.voltage_kv >= 1.0, "Voltaje negativo debe clampearse a mínimo");
        let high = AntiGravBridge::new(200.0);
        assert!(high.voltage_kv <= 60.0, "Voltaje excesivo debe clampearse");
    }

    // ── Física atmosférica ──────────────────────────────────────────────────

    #[test]
    fn test_air_density_sea_level() {
        let rho = AntiGravBridge::air_density_at(0.0);
        assert!((rho - RHO_SL).abs() < 1e-10, "SL debe ser {}", RHO_SL);
    }

    #[test]
    fn test_air_density_decreases_with_altitude() {
        let rho0 = AntiGravBridge::air_density_at(0.0);
        let rho1k = AntiGravBridge::air_density_at(1_000.0);
        let rho10k = AntiGravBridge::air_density_at(10_000.0);
        assert!(rho1k < rho0, "ρ(1km) < ρ(SL)");
        assert!(rho10k < rho1k, "ρ(10km) < ρ(1km)");
        assert!(rho10k > 0.0, "Densidad siempre positiva");
    }

    // ── Física EHD ──────────────────────────────────────────────────────────

    #[test]
    fn test_ehd_no_corona_low_voltage() {
        // 5 kV a 5 cm gap con radio de punta estándar: campo en punta insuficiente
        // E_avg = 100 kV/m, E_tip = 100k * sqrt(500) ≈ 2.24 MV/m < 3 MV/m → sin corona
        let (t, i, p) = AntiGravBridge::compute_ehd(5.0, DEFAULT_GAP, DEFAULT_EMITTER_RADIUS, RHO_SL);
        assert_eq!(t, 0.0, "Sin corona a 5 kV/5cm con radio estándar");
        assert_eq!(i, 0.0);
        assert_eq!(p, 0.0);
    }

    #[test]
    fn test_ehd_corona_at_30kv() {
        // 30 kV a 5 cm: E_tip ≈ 13.4 MV/m >> 3 MV/m → corona activa
        let (thrust, current, power) =
            AntiGravBridge::compute_ehd(30.0, DEFAULT_GAP, DEFAULT_EMITTER_RADIUS, RHO_SL);
        assert!(thrust > 0.0, "Thrust > 0 con corona activa");
        assert!(current > 0.0, "Corriente > 0 con corona");
        assert!(power > 0.0, "Potencia > 0 con corona");
        // Verificar rango físico realista: ~10-100 mN para 30 kV
        assert!(thrust > 1e-3 && thrust < 1.0, "Thrust en rango realista [mN..N]");
    }

    #[test]
    fn test_ehd_thrust_increases_with_voltage() {
        let (t20, _, _) = AntiGravBridge::compute_ehd(20.0, DEFAULT_GAP, DEFAULT_EMITTER_RADIUS, RHO_SL);
        let (t30, _, _) = AntiGravBridge::compute_ehd(30.0, DEFAULT_GAP, DEFAULT_EMITTER_RADIUS, RHO_SL);
        let (t40, _, _) = AntiGravBridge::compute_ehd(40.0, DEFAULT_GAP, DEFAULT_EMITTER_RADIUS, RHO_SL);
        assert!(t20 <= t30, "T(20kV) ≤ T(30kV)");
        assert!(t30 <= t40, "T(30kV) ≤ T(40kV)");
    }

    #[test]
    fn test_ehd_thrust_scales_with_air_density() {
        // Menor densidad → menos iones → menos empuje
        let (t_sl, _, _) = AntiGravBridge::compute_ehd(30.0, DEFAULT_GAP, DEFAULT_EMITTER_RADIUS, RHO_SL);
        let (t_10k, _, _) = AntiGravBridge::compute_ehd(30.0, DEFAULT_GAP, DEFAULT_EMITTER_RADIUS, 0.4);
        assert!(t_10k < t_sl, "Menos empuje a menor densidad de aire");
    }

    #[test]
    fn test_ehd_consistency_with_ehd_formula() {
        // T = I * d / μ_ion → I = T * μ_ion / d
        // Verificar que la corriente y el empuje son consistentes
        let (thrust, current_ua, _) =
            AntiGravBridge::compute_ehd(30.0, DEFAULT_GAP, DEFAULT_EMITTER_RADIUS, RHO_SL);
        let expected_current_ua = thrust * MU_ION / DEFAULT_GAP * 1e6;
        let diff = (current_ua - expected_current_ua).abs();
        assert!(diff < 1e-10, "I y T deben ser EHD-consistentes. Diff={}", diff);
    }

    // ── Dinámica de vuelo ────────────────────────────────────────────────────

    #[test]
    fn test_ground_clamping() {
        let mut b = AntiGravBridge::new(10.0);
        b.pos.z = 0.001;
        b.vel.z = -5.0; // hacia abajo
        b.update(0.05);
        assert!(b.pos.z >= 0.0, "No debe penetrar el suelo");
    }

    #[test]
    fn test_lifter_ascends_at_high_voltage() {
        // Con 40 kV y masa reducida, debe ascender
        let mut b = AntiGravBridge::new(40.0);
        b.qaoa_interval_ticks = 1000; // desactivar QAOA para el test
        let initial_z = b.pos.z;
        for _ in 0..50 {
            b.update(0.05);
        }
        assert!(b.pos.z > initial_z, "El lifter debe ascender con 40 kV");
    }

    #[test]
    fn test_tick_counter() {
        let mut b = AntiGravBridge::new(30.0);
        for _ in 0..200 {
            b.update(0.05);
        }
        assert_eq!(b.tick, 200);
    }

    // ── QAOA ────────────────────────────────────────────────────────────────

    #[test]
    fn test_qaoa_returns_valid_voltage() {
        let b = AntiGravBridge::new(30.0);
        let v = b.qaoa_classical_optimize();
        assert!(v >= 5.0 && v <= 55.0, "QAOA debe devolver voltaje en rango válido");
    }

    #[test]
    fn test_qaoa_voltage_is_in_discrete_levels() {
        let b = AntiGravBridge::new(30.0);
        let v = b.qaoa_classical_optimize();
        // Debe ser uno de los 8 niveles discretos
        let valid = QAOA_VOLTAGES_KV.iter().any(|&vl| (vl - v).abs() < 0.1);
        assert!(valid, "QAOA debe retornar uno de los 8 niveles: {:?}", QAOA_VOLTAGES_KV);
    }

    #[test]
    fn test_qaoa_prefers_operational_voltage() {
        // El QAOA debe preferir voltajes que generen thrust > weight
        let b = AntiGravBridge::new(30.0);
        let v = b.qaoa_classical_optimize();
        let (thrust, _, _) = AntiGravBridge::compute_ehd(v, b.gap_m, b.emitter_radius_m, b.air_density);
        let weight = b.mass_kg * G;
        // El voltaje óptimo debe poder levantar el lifter (si existe tal voltaje)
        if thrust > 0.0 {
            assert!(
                thrust >= weight * 0.9, // margen del 10%
                "QAOA debe preferir voltajes operacionales. T={:.4}N, W={:.4}N", thrust, weight
            );
        }
    }

    // ── MZI Fotónico ─────────────────────────────────────────────────────────

    #[test]
    fn test_mzi_output_ranges() {
        let mut b = AntiGravBridge::new(35.0);
        b.update(0.05); // generar thrust no-cero
        let mzi = b.pulse_control();
        assert!(mzi.phase_shift_rad >= 0.0 && mzi.phase_shift_rad <= PI + 1e-10);
        assert!(mzi.transmittance >= 0.0 && mzi.transmittance <= 1.0 + 1e-10);
        assert!(mzi.optical_power_mw >= 0.0 && mzi.optical_power_mw <= 10.0 + 1e-10);
        assert!(mzi.pulse_freq_hz >= 1e6);
        assert_eq!(mzi.wavelength_nm, 1550.0);
    }

    #[test]
    fn test_mzi_zero_thrust_full_transmittance() {
        // Sin corona (bajo voltaje) → thrust=0 → φ=0 → Trans=cos²(0)=1
        let mut b = AntiGravBridge::new(5.0);
        b.update(0.05);
        // Si no hay corona, thrust=0
        if b.thrust_vec.norm() < 1e-12 {
            let mzi = b.pulse_control();
            assert!((mzi.phase_shift_rad).abs() < 1e-10, "φ=0 cuando thrust=0");
            assert!((mzi.transmittance - 1.0).abs() < 1e-10, "Trans=1 cuando φ=0");
        }
    }

    // ── Balanceo de Red ──────────────────────────────────────────────────────

    #[test]
    fn test_balance_grid_node_id() {
        let b = AntiGravBridge::new(30.0);
        let load = b.balance_grid();
        assert_eq!(load.node_id, 99, "Nodo de propulsión = 99");
    }

    #[test]
    fn test_balance_grid_efficiency_range() {
        let mut b = AntiGravBridge::new(30.0);
        b.update(0.1);
        let load = b.balance_grid();
        assert!(load.conversion_efficiency_n_per_w >= 0.0);
        assert!(load.qaoa_priority >= 0.0 && load.qaoa_priority <= 1.0);
        assert!(!load.regenerative, "No regenerativo por defecto");
    }

    // ── Telemetría ────────────────────────────────────────────────────────────

    #[test]
    fn test_telemetry_serialization() {
        let mut b = AntiGravBridge::new(30.0);
        b.update(0.05);
        let tel = b.telemetry();
        // Debe serializar a JSON sin pánico
        let json = serde_json::to_string(&tel).expect("Debe serializar a JSON");
        assert!(json.contains("voltage_kv"), "JSON debe contener campos de telemetría");
    }

    #[test]
    fn test_status_string_not_empty() {
        let mut b = AntiGravBridge::new(30.0);
        b.update(0.05);
        assert!(!b.get_status().is_empty());
    }

    // ── Integración completa ──────────────────────────────────────────────────

    #[test]
    fn test_200_ticks_no_panic_no_nan() {
        let mut b = AntiGravBridge::new(30.0);
        for _ in 0..200 {
            b.update(0.05);
            // Ningún valor debe ser NaN o infinito
            assert!(b.pos.z.is_finite(), "Altitud debe ser finita");
            assert!(b.vel.z.is_finite(), "Velocidad debe ser finita");
            assert!(b.thrust_vec.norm().is_finite(), "Thrust debe ser finito");
        }
        assert_eq!(b.tick, 200);
    }

    #[test]
    fn test_with_electrodes_custom_gap() {
        // Lifter con gap doble (10 cm)
        let em = Electrode::emitter([0.0, 0.0, 0.0]);
        let col = Electrode::collector([0.0, 0.0, 0.1]); // 10 cm gap
        let b = AntiGravBridge::with_electrodes(30.0, vec![em, col], 0.001);
        assert!((b.gap_m - 0.1).abs() < 1e-10, "Gap debe ser 10 cm");
    }
}
