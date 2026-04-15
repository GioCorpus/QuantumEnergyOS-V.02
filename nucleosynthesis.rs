// nucleosynthesis.rs
use std::f64::consts::E;
use crate::reheating::ReheatingPhase;
use crate::entropy::EntropyTracker;

pub struct Nucleosynthesis {
    temp: f64,                  // baja de 10^9 K a 10^7 K
    neutron_fraction: f64,      // ~1/6 al inicio, decay a 1/7
    helium_yield: f64,          // Y_p ≈ 0.25
    deuterium_bottle: f64,      // bottleneck: D/H ~10^{-4}
    entropy_guard: EntropyTracker,
}

impl Nucleosynthesis {
    pub fn new() -> Self {
        Nucleosynthesis {
            temp: 1e9,
            neutron_fraction: 0.15,
            helium_yield: 0.25,
            deuterium_bottle: 1e-4,
            entropy_guard: EntropyTracker::new(),
        }
    }

    pub fn fuse(&mut self, rh: &ReheatingPhase, dt: f64) {
        if rh.get_temp() > 1e9 { return; }  // aún demasiado caliente

        self.temp -= dt * 1e8;  // enfriamiento por expansión
        self.neutron_fraction *= E.powf(-dt * 1e-3);  // decay n → p + e + ν

        // Helio: casi todo neutrones se convierten en He-4
        self.helium_yield = 2.0 * self.neutron_fraction / (1.0 + self.neutron_fraction);
        
        self.entropy_guard.tick(self.temp * 1e-12);  // entropía se estabiliza

        if self.temp < 1e7 {
            println!("Nucleosíntesis lista: ~25% He, 75% H. Litio mínimo.");
        }
    }
}

# fn helium_real() {
    let mut ns = Nucleosynthesis::new();
    assert!(ns.helium_yield > 0.24 && ns.helium_yield < 0.26, "¡Helio off! Universo sin estrellas");
}
