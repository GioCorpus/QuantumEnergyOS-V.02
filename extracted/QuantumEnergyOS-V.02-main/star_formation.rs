// star_formation.rs
use crate::structure_formation::StructureFormation;
use crate::entropy::EntropyTracker;

pub struct StarFormation {
    core_mass: f64,             // crece hasta ~0.08-100 soles
    accretion_rate: f64,        // Mdot ~10^{-5} M_sun/yr
    temp_core: f64,             // sube a 10^7 K para fusión
    feedback_factor: f64,       // radiación expulsa gas si demasiado
    entropy_guard: EntropyTracker,
}

impl StarFormation {
    pub fn new() -> Self {
        StarFormation {
            core_mass: 0.01,        // masa inicial Jeans
            accretion_rate: 1e-5,
            temp_core: 1e4,
            feedback_factor: 0.1,
            entropy_guard: EntropyTracker::new(),
        }
    }

    pub fn accrete(&mut self, sf: &StructureFormation, time: f64) {
        let delta_m = self.accretion_rate * time / 1e6;  // Myr scale
        self.core_mass += delta_m * (1.0 - self.feedback_factor);

        self.temp_core += delta_m * 1e6;  // calentamiento por compresión
        
        // Feedback: si masa > 50 M_sun, expulsa 20%
        if self.core_mass > 50.0 {
            self.core_mass *= 0.8;
            println!("¡Feedback fuerte! Supernova inminente.");
        }

        self.entropy_guard.tick(self.temp_core * 1e-12);

        if self.temp_core > 1e7 {
            println!("¡Ignición! Fusión H→He, estrella en main-sequence.");
        }
    }

    pub fn is_star(&self) -> bool {
        self.core_mass >= 0.08 && self.temp_core >= 1e7
    }
}

# fn star_real() {
    let mut st = StarFormation::new();
    assert!(st.core_mass >= 0.08, "¡Demasiado pequeña! No es estrella");
}
