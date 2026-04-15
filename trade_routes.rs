// trade_routes.rs
use crate::galaxy_evolution::GalaxyEvolution;
use crate::entropy::EntropyTracker;

pub struct TradeRoutes {
    route_length: f64,          // en años luz
    wormhole_count: u32,
    supernova_hubs: u32,
    latency: f64,               // tiempo de tránsito
    energy_cost: f64,
    entropy_guard: EntropyTracker,
}

impl TradeRoutes {
    pub fn new() -> Self {
        TradeRoutes {
            route_length: 100.0,
            wormhole_count: 5,
            supernova_hubs: 3,
            latency: 50.0,          // años, reducible con wormholes
            energy_cost: 1e12,      // en joules, arbitrario
            entropy_guard: EntropyTracker::new(),
        }
    }

    pub fn transit(&mut self, ge: &GalaxyEvolution, time: f64) {
        // Wormholes acortan: cada uno divide latencia por 10
        self.latency = self.route_length / (self.wormhole_count as f64).powf(2.0);
        
        // Supernovas: hubs de recarga, pero riesgosos
        self.energy_cost -= self.supernova_hubs as f64 * 1e10;
        
        self.entropy_guard.tick(self.latency * 1e-5);  // comercio genera entropía

        if self.latency < 1.0 {
            println!("¡Ruta optimizada! Entrega en menos de un año luz.");
        }
    }

    pub fn add_wormhole(&mut self) {
        self.wormhole_count += 1;
        println!("Nuevo shortcut: wormhole activado.");
    }
}

# fn route_open() {
    let mut tr = TradeRoutes::new();
    assert!(tr.latency < 100.0, "¡Bloqueo! Comercio detenido");
}
