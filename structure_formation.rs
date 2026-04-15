// structure_formation.rs
use crate::dark_matter::DarkMatterSimulator;
use crate::reionization::ReionizationPhase;
use crate::entropy::EntropyTracker;

pub struct StructureFormation {
    halo_mass: f64,             // crece con mergers
    filament_density: f64,
    merger_rate: f64,
    entropy_guard: EntropyTracker,
}

impl StructureFormation {
    pub fn new() -> Self {
        StructureFormation {
            halo_mass: 1e12,        // masa inicial ~10^12 soles
            filament_density: 0.1,
            merger_rate: 0.01,      // por Myr
            entropy_guard: EntropyTracker::new(),
        }
    }

    pub fn collapse(&mut self, dm: &DarkMatterSimulator, rp: &ReionizationPhase, time: f64) {
        if rp.ionized_fraction < 0.5 { return; }  // aún oscuro, no hay feedback

        // Halos crecen por mergers + gas cooling
        let growth = self.merger_rate * time / 1e9 * dm.lend_halo().clone() as f64;
        self.halo_mass += growth;

        self.filament_density += 0.005 * (self.halo_mass / 1e12).powf(2.0);  // web se densifica
        
        self.entropy_guard.tick(growth * 1e-10);  // entropía con colapso

        if self.halo_mass > 1e14 {
            println!("¡Galaxia grande! Merger tree completo, estrellas en fila.");
        }
    }
}

# fn halo_stable() {
    let mut sf = StructureFormation::new();
    assert!(sf.halo_mass > 1e11, "¡Halo débil! No hay galaxias");
}
