// axion.rs
use std::f64::consts::PI;
use crate::dark_matter::DarkMatterSimulator;
use crate::entropy::EntropyTracker;

pub struct AxionField {
    mass: f64,                  // en eV, ~10^{-6} a 10^{-3} —muy liviano
    frequency: f64,             // ω = m c² / ħ, para resonancia
    density: f64,               // rho_a ~ 0.3 GeV/cm³ en halos
    entropy_guard: EntropyTracker,
}

impl AxionField {
    pub fn new() -> Self {
        AxionField {
            mass: 1e-6,             // valor típico, ajustable
            frequency: 1e-6 * 1.602e-19 / (6.582e-16),  // Hz, ~GHz range
            density: 0.3,           // GeV/cm³, del halo
            entropy_guard: EntropyTracker::new(),
        }
    }

    pub fn resonate_with_cavity(&self, cavity_freq: f64) -> f64 {
        // Si coinciden, boost de conversión a fotones —¡detección!
        let detuning = (self.frequency - cavity_freq).abs();
        let enhancement = if detuning < 1e-3 * self.frequency { PI } else { 1.0 };
        
        self.entropy_guard.tick(0.005);  // axiones generan entropía lenta
        self.density * enhancement
    }

    pub fn lend_to_dm(&self, dm: &mut DarkMatterSimulator) {
        // Axiones "prestados" al halo sin ownership —Arc sería ideal
        let extra_mass = self.density * 1e12;  // escalado galáctico
        *dm.shared_halo += extra_mass;
        println!("Axiones inyectados: halo más robusto.");
    }
}

# fn axion_mass_positive() {
    let ax = AxionField::new();
    assert!(ax.mass > 0.0, "¡Axiones negativos? El universo se encoge");
    assert!(!ax.mass.is_nan(), "NaN en masa axiónica? Bug cósmico");
    println!("Axión chequeado: liviano y estable.");
}
