// universe_main.rs
mod entropy;
mod axion;
mod dark_matter;
mod inflation;
mod reheating;
mod nucleosynthesis;
mod cmb;
mod dark_ages;

use entropy::EntropyTracker;
use inflation::InflationPhase;
use reheating::ReheatingPhase;
use nucleosynthesis::Nucleosynthesis;
use cmb::CosmicMicrowaveBackground;
use dark_ages::DarkAges;

fn main() {
    println!("Iniciando universo... cargo run --release");

    let mut entropy = EntropyTracker::new();
    let mut infl = InflationPhase::new();
    let mut rh = ReheatingPhase::new();
    let mut ns = Nucleosynthesis::new();
    let cmb = CosmicMicrowaveBackground::new();
    let mut da = DarkAges::new(&cmb);

    // Simula ticks de tiempo (escalado)
    for t in (0..1_000_000).step_by(1000) {
        infl.expand(1e-32);
        rh.decay_inflaton(&infl, 1e-10);
        ns.fuse(&rh, 1e-5);
        da.evolve(t as f64);
        entropy.tick(1e-20);

        if !entropy.is_stable() {
            println!("¡Heat death! Pero... inyectamos foam.");
            entropy.total *= 0.99;
        }
    }

    println!("Universo estable: 13.8e9 años, expansión continua. Fin del main.");
}
