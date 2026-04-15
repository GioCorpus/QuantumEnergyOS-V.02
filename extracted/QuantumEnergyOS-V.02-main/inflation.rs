// inflation.rs
use std::f64::consts::{E, PI};
use crate::entropy::EntropyTracker;
use crate::axion::AxionField;  // para semillas post-inflación

pub struct InflationPhase {
    scale_factor: f64,          // a(t) ~ e^{H t}, H constante
    hubble_param: f64,          // ~10^{-32} s^{-1} en Planck units
    duration: f64,              // ~10^{-32} s, pero escalado
    fluctuation_amp: f64,       // δρ/ρ ~ 10^{-5}, semillas CMB
    entropy_guard: EntropyTracker,
}

impl InflationPhase {
    pub fn new() -> Self {
        InflationPhase {
            scale_factor: 1.0,
            hubble_param: 1e-32,    // valor típico, ajustable
            duration: 1e-32,
            fluctuation_amp: 1e-5,
            entropy_guard: EntropyTracker::new(),
        }
    }

    pub fn expand(&mut self, dt: f64) -> f64 {
        // Inflación: escala exponencial, flatness problem solved
        self.scale_factor *= E.powf(self.hubble_param * dt);
        
        // Genera fluctuaciones cuánticas como ruido
        let delta_rho = self.fluctuation_amp * (rand::random::<f64>() - 0.5);
        self.entropy_guard.tick(delta_rho.abs() * 1e10);  // entropía crece con expansión
        
        if self.scale_factor > 1e60 {  // fin inflación, reheating
            println!("Inflación terminada: universo plano y listo para nucleosíntesis.");
            self.inject_seeds();
        }
        
        self.scale_factor
    }

    fn inject_seeds(&self) {
        // Aquí entran axiones o inflatones —semillas para CMB
        println!("Fluctuaciones inyectadas: δρ/ρ = {:.2e}", self.fluctuation_amp);
    }
}

# fn inflation_positive() {
    let mut infl = InflationPhase::new();
    assert!(infl.hubble_param > 0.0, "¡H negativo? Big Crunch instantáneo");
    assert!(!infl.scale_factor.is_nan(), "NaN en expansión? Bug en el Big Bang");
    println!("Inflación chequeada: expansión eterna.");
}
