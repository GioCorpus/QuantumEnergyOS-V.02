// reionization.rs
use crate::dark_ages::DarkAges;
use crate::entropy::EntropyTracker;

pub struct ReionizationPhase {
    ionized_fraction: f64,      // de 0.0 a 1.0
    bubble_count: u32,          // número de burbujas ionizadas
    uv_flux: f64,               // tasa de ionización por estrella
    entropy_guard: EntropyTracker,
}

impl ReionizationPhase {
    pub fn new() -> Self {
        ReionizationPhase {
            ionized_fraction: 0.0,
            bubble_count: 0,
            uv_flux: 1e-3,          // arbitrario, ~10^{-3} por Myr
            entropy_guard: EntropyTracker::new(),
        }
    }

    pub fn ionize(&mut self, da: &mut DarkAges, time: f64) {
        if da.neutral_h_fraction > 0.99 { return; }  // aún muy oscuro

        let delta_ion = self.uv_flux * (time / 1e8).powf(2.0);  // exponencial con galaxias
        self.ionized_fraction += delta_ion;
        self.ionized_fraction = self.ionized_fraction.clamp(0.0, 1.0);

        if delta_ion > 0.01 {
            self.bubble_count += 1;
            println!("Burbuja ionizada —luz se expande.");
        }

        da.neutral_h_fraction = 1.0 - self.ionized_fraction;
        self.entropy_guard.tick(delta_ion * 1e-8);  // entropía sube con fotones

        if self.ionized_fraction > 0.99 {
            println!("Reionización completa: universo transparente, galaxias brillando.");
        }
    }
}

# fn reion_not_early() {
    let mut rp = ReionizationPhase::new();
    assert!(rp.ionized_fraction < 0.1, "¡Ionizado prematuro! Estrellas antes de formarse?");
}
