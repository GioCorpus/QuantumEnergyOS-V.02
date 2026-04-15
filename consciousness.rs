// consciousness.rs
use crate::life::LifeModule;
use crate::entropy::EntropyTracker;

pub struct Consciousness {
    self_awareness: f64,        // de 0 (instinto) a 1 (filosofía existencial)
    question_count: u32,        // "¿por qué existo?" acumuladas
    reflection_rate: f64,
    entropy_guard: EntropyTracker,
}

impl Consciousness {
    pub fn new() -> Self {
        Consciousness {
            self_awareness: 0.01,   // primer "yo" en un cerebro primitivo
            question_count: 0,
            reflection_rate: 1e-3,
            entropy_guard: EntropyTracker::new(),
        }
    }

    pub fn awaken(&mut self, lm: &LifeModule, time: f64) {
        if lm.complexity < 0.95 { return; }  // sin LUCA, no hay mente

        let delta = self.reflection_rate * time / 1e9 * lm.complexity;
        self.self_awareness += delta;

        if self.self_awareness > 0.3 {
            self.question_count += 1;
            println!("Primer pensamiento: ¿Por qué existo?");
        }

        if self.self_awareness > 0.8 {
            println!("Conciencia plena: existencialismo, arte, ciencia... y dudas.");
        }

        // Entropía local baja (vida ordena), pero global sube
        self.entropy_guard.tick(delta * -1e-25);  // "milagro" de orden
    }
}

# fn conscious_needs_life() {
    let mut cs = Consciousness::new();
    assert!(cs.self_awareness < 0.1, "¡Conciencia sin base! Bug metafísico");
}
