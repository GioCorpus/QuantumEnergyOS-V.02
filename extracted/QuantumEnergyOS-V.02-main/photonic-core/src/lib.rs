// QuantumEnergyOS - Photonic Core Library
// Modelado de componentes nanométricos y propagación de luz.

use num_complex::Complex64;

#[derive(Debug, Clone)]
pub struct Waveguide {
    pub length_um: f64,
    pub loss_db_cm: f64,
}

pub struct MachZehnderInterferometer {
    pub phase_shift: f64, // Ajustable eléctricamente o térmicamente
}

impl MachZehnderInterferometer {
    /// Transforma señales de entrada (E1, E2) usando interferencia cuántica.
    /// Matriz de transferencia: $$ U = \frac{1}{\sqrt{2}} \begin{pmatrix} e^{i\phi} & 1 \\ 1 & -e^{-i\phi} \end{pmatrix} $$
    pub fn propagate(&self, e_in: (Complex64, Complex64)) -> (Complex64, Complex64) {
        let phi = Complex64::from_polar(1.0, self.phase_shift);
        let inv_phi = Complex64::from_polar(1.0, -self.phase_shift);
        
        let out1 = (e_in.0 * phi + e_in.1) / 2.0_f64.sqrt();
        let out2 = (e_in.0 - e_in.1 * inv_phi) / 2.0_f64.sqrt();
        
        (out1, out2)
    }
}

// QuantumEnergyOS - Photonic Core Primitives
// Implementación de física cuántica aplicada a la ingeniería de software.

pub struct MachZehnderInterferometer {
    pub phase_shifter: f64,
}

impl MachZehnderInterferometer {
    pub fn new(initial_phase: f64) -> Self {
        Self { phase_shifter: initial_phase }
    }

    /// Simula la interferencia de dos señales lumínicas.
    /// Salida basada en la transformación de Hadamard:
    /// $$ \psi_{out} = \cos(\phi/2) \psi_1 + i\sin(\phi/2) \psi_2 $$
    pub fn interfere(&self, input_a: f64, input_b: f64) -> (f64, f64) {
        let out_a = input_a * (self.phase_shifter / 2.0).cos();
        let out_b = input_b * (self.phase_shifter / 2.0).sin();
        (out_a, out_b)
    }
}

pub struct Waveguide {
    pub length_nm: u32,
    pub loss_db: f32,
}
