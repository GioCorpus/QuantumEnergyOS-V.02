// dark_ages.rs
use crate::cmb::CosmicMicrowaveBackground;
use crate::entropy::EntropyTracker;

pub struct DarkAges {
    temp: f64,                  // baja de 3000 K a ~10 K
    neutral_h_fraction: f64,    // ~1.0 al inicio
    reion_start_time: f64,      // ~200 Myr
    entropy_guard: EntropyTracker,
}

impl DarkAges {
    pub fn new(cmb: &CosmicMicrowaveBackground) -> Self {
        DarkAges {
            temp: cmb.get_temp_at_decoupling() * 0.001,  // enfriamiento por expansión
            neutral_h_fraction: 1.0,
            reion_start_time: 2e8,  // años
            entropy_guard: EntropyTracker::new(),
        }
    }

    pub fn evolve(&mut self, time: f64) {
        self.temp *= (1.0 / (1.0 + time / 1e5));  // ~T ∝ 1/a
        self.entropy_guard.tick(self.temp * 1e-15);

        if time > self.reion_start_time {
            self.neutral_h_fraction -= 0.01 * (time - self.reion_start_time) / 1e8;
            if self.neutral_h_fraction < 0.01 {
                println!("¡Reionización! Primeras estrellas encienden el cosmos.");
            }
        }
    }
}

# fn dark_not_frozen() {
    let cmb = CosmicMicrowaveBackground::new();
    let mut da = DarkAges::new(&cmb);
    assert!(da.temp > 10.0, "¡Demasiado frío! No hay reionización");
}
