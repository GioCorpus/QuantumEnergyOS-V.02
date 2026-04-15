// reheating.rs
use std::f64::consts::E;
use crate::inflation::InflationPhase;
use crate::entropy::EntropyTracker;

pub struct ReheatingPhase {
    temperature: f64,           // en GeV, ~10^15 al inicio
    inflaton_decay_rate: f64,   // Γ ~ 10^{-3} H
    radiation_density: f64,
    entropy_guard: EntropyTracker,
}

impl ReheatingPhase {
    pub fn new() -> Self {
        ReheatingPhase {
            temperature: 1e15,      // típico post-inflación
            inflaton_decay_rate: 1e-3,  // decay rate
            radiation_density: 0.0,
            entropy_guard: EntropyTracker::new(),
        }
    }

    pub fn decay_inflaton(&mut self, infl: &InflationPhase, dt: f64) {
        // Inflaton oscila y decae → energía → radiación
        let decay = self.inflaton_decay_rate * dt;
        self.radiation_density += infl.scale_factor.powf(4.0) * decay;  // rho_r ~ a^{-4}
        
        // Temperatura: T ~ rho^{1/4}
        self.temperature = (self.radiation_density).powf(0.25);
        
        self.entropy_guard.tick(self.temperature * 1e-10);  // entropía crece con calor
        
        if self.temperature < 1e9 {  // fin reheating, ~MeV
            println!("Reheating completado: plasma a 10^9 K, nucleosíntesis lista.");
        }
    }

    pub fn get_temp(&self) -> f64 {
        self.temperature
    }
}

# fn reheating_hot() {
    let mut rh = ReheatingPhase::new();
    assert!(rh.temperature > 1e12, "¡Frío prematuro! No hay partículas");
    assert!(!rh.temperature.is_nan(), "NaN en temperatura? Big Bang congelado");
    println!("Reheating chequeado: universo hirviendo.");
}
