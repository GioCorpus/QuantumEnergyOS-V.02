// galaxy_evolution.rs
use crate::structure_formation::StructureFormation;
use crate::star_formation::StarFormation;
use crate::entropy::EntropyTracker;

pub struct GalaxyEvolution {
    cluster_mass: f64,          // crece con mergers, ~10^14-10^15 M_sun
    galaxy_count: u32,
    agn_feedback: f64,          // agujeros negros regulan crecimiento
    entropy_guard: EntropyTracker,
}

impl GalaxyEvolution {
    pub fn new() -> Self {
        GalaxyEvolution {
            cluster_mass: 1e14,
            galaxy_count: 10,
            agn_feedback: 0.05,
            entropy_guard: EntropyTracker::new(),
        }
    }

    pub fn evolve_cluster(&mut self, sf: &StructureFormation, st: &StarFormation, time: f64) {
        // Mergers: clusters se juntan, masa sube
        let merger_delta = sf.halo_mass * 0.1 * (time / 1e10);  // escala Gyr
        self.cluster_mass += merger_delta;

        // Estrellas: si hay formación, cluster crece
        if st.is_star() {
            self.galaxy_count += 1;
        }

        // AGN feedback: evita overgrowth
        self.cluster_mass -= self.cluster_mass * self.agn_feedback * (time / 1e9);

        self.entropy_guard.tick(merger_delta * 1e-15);

        if self.cluster_mass > 1e15 {
            println!("¡Supercluster! Virgo-like, miles de galaxias unidas.");
        }
    }
}
