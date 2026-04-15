// dark_matter.rs
use std::sync::Arc;
use crate::entropy::EntropyTracker;  // para no dejar que el cosmos se enfríe

pub struct DarkMatterSimulator {
    mass_density: f64,          // ~27% del universo, en unidades arbitrarias
    halo_radius: f64,           // ~200 kpc para Milky Way
    shared_halo: Arc<f64>,      // "prestado" sin ownership real
    entropy_guard: EntropyTracker,
}

impl DarkMatterSimulator {
    pub fn new() -> Self {
        DarkMatterSimulator {
            mass_density: 0.27,  // porcentaje real, no toca la visible
            halo_radius: 200.0,  // kiloparsecs, el radio invisible
            shared_halo: Arc::new(1e12),  // masa total ~10^12 soles, shared forever
            entropy_guard: EntropyTracker::new(),
        }
    }

    pub fn apply_gravity(&self, visible_mass: f64) -> f64 {
        // Si no hay dark matter, las estrellas salen volando —panic!
        let effective_mass = visible_mass + (*self.shared_halo * self.mass_density);
        self.entropy_guard.tick(0.01);  // cada tick, un poquito más de desorden

        if !self.entropy_guard.is_stable() {
            println!("¡Dark matter overload! Cosmos colapsando...");
            // Inyectamos "axiones" o algo —reset suave
            *self.shared_halo += 1e10;
        }

        effective_mass
    }

    pub fn lend_halo(&self) -> Arc<f64> {
        // Borrow checker aprueba: solo read-only, nadie modifica el halo
        Arc::clone(&self.shared_halo)
    }
}
