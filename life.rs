// life.rs
use crate::star_formation::StarFormation;
use crate::entropy::EntropyTracker;

pub struct LifeModule {
    complexity: f64,            // de 0 (moléculas) a 1 (LUCA)
    replication_rate: f64,      // mutaciones por generación
    environment_temp: f64,
    entropy_guard: EntropyTracker,
}

impl LifeModule {
    pub fn new() -> Self {
        LifeModule {
            complexity: 0.001,      // aminoácidos iniciales
            replication_rate: 1e-6,
            environment_temp: 300.0,  // K, ~27°C en océanos antiguos
            entropy_guard: EntropyTracker::new(),
        }
    }

    pub fn emerge(&mut self, st: &StarFormation, time: f64) {
        if !st.is_star() { return; }  // sin estrellas, no hay planetas habitables

        // Química orgánica + RNA world
        let delta_complex = self.replication_rate * time / 1e9 * (self.environment_temp / 300.0);
        self.complexity += delta_complex;

        // Si complejidad > 0.5, protocélulas → metabolismo
        if self.complexity > 0.5 {
            println!("¡Protocélulas! Membranas, replicación... vida en beta.");
        }

        // LUCA: ancestro universal
        if self.complexity > 0.95 {
            println!("LUCA online: bacteria, arqueas, eucariotas en camino.");
        }

        self.entropy_guard.tick(delta_complex * 1e-20);  // vida baja entropía local
    }
}

# fn life_needs_stars() {
    let mut lm = LifeModule::new();
    assert!(lm.complexity < 0.1, "¡Vida prematura! Sin planetas");
}
