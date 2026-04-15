// ═══════════════════════════════════════════════════════════════════════
//  qcall — API Pública para Operaciones Fotónicas
//
//  Esta es la interfaz que ven las aplicaciones: VQE para química,
//  QAOA para la red eléctrica, multiplicación matricial óptica,
//  y acceso directo al hardware de silicio fotónico.
//
//  Uso desde aplicación:
//
//    use qcall::prelude::*;
//
//    // Balancear la red de Baja California con QAOA fotónico
//    let result = qcall_qaoa(QAOARequest::baja_california(p_layers: 2))?;
//    println!("Ahorro: {:.1} kW", result.energy_saved_kw);
//
//    // Simular molécula con VQE fotónico
//    let energy = qcall_vqe(VQERequest::h2o(n_modes: 8, n_layers: 4))?;
//
//    // Multiplicación matricial óptica (O(1) en hardware)
//    let output = qcall_matmul_optical(matrix, input_vector)?;
// ═══════════════════════════════════════════════════════════════════════

#![no_std]
extern crate alloc;

use alloc::{vec::Vec, string::String};
use serde::{Deserialize, Serialize};

pub mod prelude {
    pub use super::{
        qcall_vqe, qcall_qaoa, qcall_matmul_optical,
        qcall_homodyne, qcall_generate_pairs, qcall_gkp_correct,
        VQERequest, VQEResult, QAOARequest, QAOAResult,
        MatMulRequest, MatMulResult, HomodyneRequest, HomodyneResult,
    };
}

// ═══════════════════════════════════════════════════════════════════════
//  VQE — Variational Quantum Eigensolver fotónico
//
//  Calcula la energía del estado fundamental de una molécula.
//  Aplicación directa: simulación de catalizadores para energía limpia.
// ═══════════════════════════════════════════════════════════════════════

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VQERequest {
    /// Número de modos fotónicos (= número de qubits lógicos)
    pub n_modes:       u32,
    /// Profundidad del ansatz (más capas = más precisión, más tiempo)
    pub n_layers:      u32,
    /// Parámetros variacionales iniciales
    pub ansatz_params: Vec<f64>,
    /// Elementos del observable (Hamiltoniano de la molécula)
    pub observable:    Vec<f64>,
    /// Molécula objetivo
    pub molecule:      MoleculeSpec,
    /// Convergencia: criterio de parada |ΔE| < threshold
    pub convergence:   f64,
    /// Máximo número de iteraciones
    pub max_iter:      u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MoleculeSpec {
    pub name:       String,
    pub n_electrons: u32,
    pub n_orbitals:  u32,
    pub basis_set:   BasisSet,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum BasisSet {
    STO3G,    // Mínima — para pruebas rápidas
    CCPVDZ,   // Double-zeta — balance precisión/costo
    CCPVTZ,   // Triple-zeta — alta precisión
}

impl VQERequest {
    pub fn h2(n_modes: u32, n_layers: u32) -> Self {
        Self {
            n_modes,
            n_layers,
            ansatz_params: (0..2*n_modes).map(|i| (i as f64 * 0.1) % 3.14159).collect(),
            observable:    vec![-1.1175, 0.3980, -0.3980, 0.1809, 0.1809, 0.1809],
            molecule:      MoleculeSpec {
                name:        "H2".into(),
                n_electrons: 2,
                n_orbitals:  2,
                basis_set:   BasisSet::STO3G,
            },
            convergence: 1e-6,
            max_iter:    100,
        }
    }

    pub fn h2o(n_modes: u32, n_layers: u32) -> Self {
        Self {
            n_modes,
            n_layers,
            ansatz_params: (0..2*n_modes).map(|i| (i as f64 * 0.05) % 6.28318).collect(),
            observable:    (0..n_modes*n_modes).map(|i| (i as f64 * 0.01) - 75.0).collect(),
            molecule:      MoleculeSpec {
                name:        "H2O".into(),
                n_electrons: 10,
                n_orbitals:  7,
                basis_set:   BasisSet::CCPVDZ,
            },
            convergence: 1e-8,
            max_iter:    500,
        }
    }

    /// H₂O₂ — molécula relevante para catálisis de hidrógeno (energía limpia)
    pub fn h2o2_catalyst(n_modes: u32) -> Self {
        Self {
            n_modes,
            n_layers: 6,
            ansatz_params: (0..3*n_modes).map(|i| (i as f64 * 0.03) % 6.28318).collect(),
            observable:    (0..n_modes*n_modes).map(|i| (i as f64 * 0.005) - 150.0).collect(),
            molecule:      MoleculeSpec {
                name:        "H2O2".into(),
                n_electrons: 18,
                n_orbitals:  10,
                basis_set:   BasisSet::CCPVTZ,
            },
            convergence: 1e-10,
            max_iter:    1000,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VQEResult {
    pub energy_hartree:        f64,  // Energía en unidades atómicas (Hartree)
    pub energy_ev:             f64,  // Energía en eV
    pub converged:             bool,
    pub iterations:            u32,
    pub optimal_params:        Vec<f64>,
    pub circuit_depth:         u32,
    pub execution_time_ms:     f64,
    pub chip_used:             String,
    pub squeezing_db_used:     f64,
}

impl VQEResult {
    /// Energía de enlace en kJ/mol (para comparar con literatura)
    pub fn binding_energy_kj_mol(&self) -> f64 {
        // 1 Hartree = 2625.5 kJ/mol
        self.energy_hartree * 2625.5
    }
}

// ═══════════════════════════════════════════════════════════════════════
//  QAOA — Quantum Approximate Optimization fotónico
//  Aplicación: balanceo de red eléctrica en tiempo real
// ═══════════════════════════════════════════════════════════════════════

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QAOARequest {
    pub nodes:    Vec<GridNode>,
    pub p_layers: u32,
    pub gamma:    Vec<f64>,
    pub beta:     Vec<f64>,
    pub region:   String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GridNode {
    pub id:          u32,
    pub name:        String,
    pub load_kw:     f64,
    pub capacity_kw: f64,
    pub overloaded:  bool,
}

impl QAOARequest {
    pub fn baja_california(p_layers: u32) -> Self {
        Self {
            nodes: vec![
                GridNode { id: 0, name: "Mexicali".into(),   load_kw: 85_000.0, capacity_kw: 120_000.0, overloaded: false },
                GridNode { id: 1, name: "Tijuana".into(),    load_kw: 95_000.0, capacity_kw: 130_000.0, overloaded: false },
                GridNode { id: 2, name: "Ensenada".into(),   load_kw: 42_000.0, capacity_kw: 65_000.0,  overloaded: false },
                GridNode { id: 3, name: "Tecate".into(),     load_kw: 18_000.0, capacity_kw: 30_000.0,  overloaded: false },
                GridNode { id: 4, name: "San Felipe".into(), load_kw: 8_500.0,  capacity_kw: 15_000.0,  overloaded: false },
                GridNode { id: 5, name: "Rosarito".into(),   load_kw: 22_000.0, capacity_kw: 35_000.0,  overloaded: false },
            ],
            p_layers,
            gamma: (0..p_layers).map(|i| 0.5 + 0.1 * i as f64).collect(),
            beta:  (0..p_layers).map(|i| 0.3 - 0.05 * i as f64).collect(),
            region: "Baja California".into(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QAOAResult {
    pub optimal_config:  Vec<u8>,   // Bitstring: 1=nodo activo
    pub energy_saved_kw: f64,
    pub qaoa_energy:     f64,
    pub approximation_ratio: f64,
    pub execution_time_ms: f64,
    pub chip_used:       String,
    pub n_photons_used:  u64,
}

impl QAOAResult {
    pub fn energy_saved_mwh_per_year(&self) -> f64 {
        self.energy_saved_kw * 8760.0 / 1000.0
    }

    pub fn co2_avoided_tons_per_year(&self) -> f64 {
        // Factor de emisión CFE: ~0.45 kg CO₂/kWh
        self.energy_saved_mwh_per_year() * 0.45
    }
}

// ═══════════════════════════════════════════════════════════════════════
//  MULTIPLICACIÓN MATRICIAL ÓPTICA
//  La operación más poderosa del PIC: O(N) en energía vs O(N²) digital
// ═══════════════════════════════════════════════════════════════════════

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MatMulRequest {
    /// Matriz U como lista de ángulos MZI (thetas y phis)
    pub matrix_thetas: Vec<f64>,
    pub matrix_phis:   Vec<f64>,
    /// Vector de entrada (amplitudes de luz)
    pub input_vector:  Vec<f64>,
    /// Chip a usar
    pub chip_id:       u8,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MatMulResult {
    pub output_vector:   Vec<f64>,
    pub execution_ns:    f64,    // típicamente ~1-5 ns en hardware
    pub energy_pj:       f64,    // energía en picojoules
    pub throughput_tops: f64,    // TeraOPS
}

// ═══════════════════════════════════════════════════════════════════════
//  HOMODYNE MEASUREMENT
// ═══════════════════════════════════════════════════════════════════════

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HomodyneRequest {
    pub mode_idx:   u32,
    pub quadrature: u8,   // 0=X̂, 1=P̂
    pub chip_id:    u8,
    pub n_shots:    u32,  // número de mediciones para estadística
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HomodyneResult {
    pub outcomes:      Vec<f64>,
    pub mean:          f64,
    pub variance:      f64,
    pub shot_noise_db: f64,
    pub squeezed:      bool,
}

// ═══════════════════════════════════════════════════════════════════════
//  FUNCIONES PÚBLICAS — Interface to kernel syscalls
// ═══════════════════════════════════════════════════════════════════════

/// Invocar VQE fotónico para simulación molecular
pub fn qcall_vqe(req: VQERequest) -> Result<VQEResult, QCallError> {
    // En kernel real: traduce a PhotonicSyscall::RunVQE y envía al bridge
    // Aquí: implementación de referencia
    if req.n_modes > 64 {
        return Err(QCallError::TooManyModes { requested: req.n_modes, max: 64 });
    }
    if req.ansatz_params.len() < (2 * req.n_modes) as usize {
        return Err(QCallError::InsufficientParams {
            expected: 2 * req.n_modes,
            got: req.ansatz_params.len() as u32,
        });
    }

    // Energía simulada (en hardware real: resultado del chip fotónico)
    let energy = req.observable.first().copied().unwrap_or(-1.0)
               * libm::cos(req.ansatz_params.first().copied().unwrap_or(0.5));

    Ok(VQEResult {
        energy_hartree:    energy,
        energy_ev:         energy * 27.2114,
        converged:         true,
        iterations:        42,
        optimal_params:    req.ansatz_params,
        circuit_depth:     req.n_layers * req.n_modes,
        execution_time_ms: 1.5,
        chip_used:         "Xanadu Borealis (simulado)".into(),
        squeezing_db_used: 12.0,
    })
}

/// Invocar QAOA fotónico para balanceo de red eléctrica
pub fn qcall_qaoa(req: QAOARequest) -> Result<QAOAResult, QCallError> {
    if req.nodes.len() > 32 {
        return Err(QCallError::TooManyNodes { max: 32 });
    }
    if req.gamma.len() != req.p_layers as usize {
        return Err(QCallError::MismatchedParams);
    }

    let total_load: f64  = req.nodes.iter().map(|n| n.load_kw).sum();
    let total_cap: f64   = req.nodes.iter().map(|n| n.capacity_kw).sum();
    let baseline_eff     = total_load / total_cap;

    // Ahorro estimado por QAOA vs balanceo clásico greedy
    let qaoa_improvement = 0.15 * (1.0 - libm::exp(-req.p_layers as f64 * 0.5));
    let energy_saved     = total_load * qaoa_improvement;

    let n = req.nodes.len();
    let optimal_config: Vec<u8> = req.nodes.iter().map(|node| {
        if node.load_kw / node.capacity_kw < 0.8 { 1 } else { 0 }
    }).collect();

    Ok(QAOAResult {
        optimal_config,
        energy_saved_kw:      energy_saved,
        qaoa_energy:          -(total_load * (1.0 - qaoa_improvement)),
        approximation_ratio:  0.85 + 0.02 * req.p_layers as f64,
        execution_time_ms:    0.8,
        chip_used:            "QuiX 12-mode (simulado)".into(),
        n_photons_used:       n as u64 * req.p_layers as u64 * 100,
    })
}

/// Multiplicación matricial óptica — instantánea en hardware
pub fn qcall_matmul_optical(req: MatMulRequest) -> Result<MatMulResult, QCallError> {
    let n = req.input_vector.len();
    if req.matrix_thetas.len() < n * (n.saturating_sub(1)) / 2 {
        return Err(QCallError::InsufficientParams {
            expected: (n * (n-1) / 2) as u32,
            got:      req.matrix_thetas.len() as u32,
        });
    }

    // Simular propagación por el mesh MZI
    let mut output = req.input_vector.clone();
    for i in 0..n.min(req.matrix_thetas.len()) {
        let theta = req.matrix_thetas[i];
        let phi   = req.matrix_phis.get(i).copied().unwrap_or(0.0);
        if i + 1 < n {
            let c = libm::cos(theta / 2.0);
            let s = libm::sin(theta / 2.0);
            let a = output[i];
            let b = output[i + 1];
            output[i]   = c * a - s * b * libm::cos(phi);
            output[i+1] = s * a + c * b * libm::cos(phi);
        }
    }

    let throughput_tops = n as f64 * n as f64 / 1e3; // estimado para chip real
    Ok(MatMulResult {
        output_vector:   output,
        execution_ns:    2.0 * n as f64 * 0.007, // tiempo de tránsito en waveguide
        energy_pj:       n as f64 * 0.5,           // ~0.5 pJ por operación óptica
        throughput_tops,
    })
}

/// Medición homodyne de una cuadratura
pub fn qcall_homodyne(req: HomodyneRequest) -> Result<HomodyneResult, QCallError> {
    let state = photonic::homodyne::CVState::vacuum();
    let quad = match req.quadrature {
        0 => photonic::homodyne::Quadrature::X,
        1 => photonic::homodyne::Quadrature::P,
        _ => return Err(QCallError::InvalidQuadrature),
    };

    let mut outcomes = Vec::new();
    let mut sum = 0.0;
    for shot in 0..req.n_shots {
        let r = photonic::homodyne::measure_homodyne(
            &state, quad, 0.97, req.mode_idx as u64 + shot as u64,
        );
        sum += r.outcome;
        outcomes.push(r.outcome);
    }

    let mean = sum / req.n_shots as f64;
    let variance = outcomes.iter().map(|&x| (x - mean).powi(2)).sum::<f64>() / req.n_shots as f64;
    let shot_noise_db = 10.0 * libm::log10(variance / 0.5);

    Ok(HomodyneResult {
        outcomes,
        mean,
        variance,
        shot_noise_db,
        squeezed: variance < 0.5,
    })
}

/// Generar pares de fotones entrelazados
pub fn qcall_generate_pairs(n_pairs: u32, chip_id: u8) -> Result<EntangledPairsResult, QCallError> {
    if n_pairs > 10_000 {
        return Err(QCallError::TooManyPairs { max: 10_000 });
    }
    Ok(EntangledPairsResult {
        n_pairs_generated: n_pairs,
        fidelity:          0.97,
        lambda_signal_nm:  1550.0,
        lambda_idler_nm:   1550.0,
        pair_rate_mhz:     1.0,
        chip_id,
    })
}

/// Trigger de corrección GKP en un modo específico
pub fn qcall_gkp_correct(mode_idx: u32, chip_id: u8) -> Result<GKPResult, QCallError> {
    let mut corrector = photonic::error_correction::GKPCorrector::new_12db();
    let state = photonic::homodyne::CVState::vacuum();
    let r = corrector.run_correction_cycle(&state, mode_idx as u64);

    Ok(GKPResult {
        corrected:          r.correction_applied,
        residual_error:     r.residual_error,
        ancillas_consumed:  r.ancilla_consumed,
        feedback_latency_ns: r.feedback_latency_ns,
        logical_error_rate: corrector.estimated_logical_error_rate(),
    })
}

// ── Tipos adicionales ─────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EntangledPairsResult {
    pub n_pairs_generated: u32,
    pub fidelity:          f64,
    pub lambda_signal_nm:  f64,
    pub lambda_idler_nm:   f64,
    pub pair_rate_mhz:     f64,
    pub chip_id:           u8,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GKPResult {
    pub corrected:           bool,
    pub residual_error:      f64,
    pub ancillas_consumed:   u32,
    pub feedback_latency_ns: f64,
    pub logical_error_rate:  f64,
}

// ── Errores ───────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum QCallError {
    TooManyModes      { requested: u32, max: u32 },
    TooManyNodes      { max: u32 },
    TooManyPairs      { max: u32 },
    InsufficientParams { expected: u32, got: u32 },
    MismatchedParams,
    InvalidQuadrature,
    ChipNotFound      { chip_id: u8 },
    DecoherenceExpired,
    CalibrationRequired { chip_id: u8 },
    BridgeLatencyExceeded { latency_ns: f64, limit_ns: f64 },
}

// Helper
mod libm {
    pub fn cos(x: f64) -> f64 { ::libm::cos(x) }
    pub fn sin(x: f64) -> f64 { ::libm::sin(x) }
    pub fn exp(x: f64) -> f64 { ::libm::exp(x) }
    pub fn log10(x: f64) -> f64 { ::libm::log10(x) }
}
