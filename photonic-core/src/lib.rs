// QuantumEnergyOS - Photonic Core Library
// Modelado de componentes nanométricos y propagación de luz.

use num_complex::Complex64;

#[derive(Debug, Clone)]
pub struct Waveguide {
    pub length_um: f64,
    pub loss_db_cm: f64,
    pub length_nm: u32,
    pub loss_db: f32,
}

impl Waveguide {
    pub fn new_um(length_um: f64, loss_db_cm: f64) -> Self {
        Self {
            length_um,
            loss_db_cm,
            length_nm: 0,
            loss_db: 0.0,
        }
    }

    pub fn new_nm(length_nm: u32, loss_db: f32) -> Self {
        Self {
            length_um: 0.0,
            loss_db_cm: 0.0,
            length_nm,
            loss_db,
        }
    }
}

pub struct MachZehnderInterferometer {
    pub phase_shift: f64,
    pub phase_shifter: f64,
}

impl MachZehnderInterferometer {
    pub fn new(phase_shift: f64) -> Self {
        Self {
            phase_shift,
            phase_shifter: phase_shift,
        }
    }

    pub fn with_phase(phase: f64) -> Self {
        Self {
            phase_shift: phase,
            phase_shifter: phase,
        }
    }

    pub fn propagate(&self, e_in: (Complex64, Complex64)) -> (Complex64, Complex64) {
        let phi = Complex64::from_polar(1.0, self.phase_shift);
        let inv_phi = Complex64::from_polar(1.0, -self.phase_shift);

        let out1 = (e_in.0 * phi + e_in.1) / 2.0_f64.sqrt();
        let out2 = (e_in.0 - e_in.1 * inv_phi) / 2.0_f64.sqrt();

        (out1, out2)
    }

    pub fn interfere(&self, input_a: f64, input_b: f64) -> (f64, f64) {
        let out_a = input_a * (self.phase_shifter / 2.0).cos();
        let out_b = input_b * (self.phase_shifter / 2.0).sin();
        (out_a, out_b)
    }
}
