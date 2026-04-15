// entropy.rs
use std::f64::consts::LOG2;

pub struct EntropyTracker {
    total: f64,  // en unidades de Boltzmann, ~1e80 para el universo actual
}

impl EntropyTracker {
    pub fn new() -> Self {
        EntropyTracker { total: 1e80f64.log(LOG2) }  // log2 para bits cósmicos
    }

    pub fn tick(&mut self, delta: f64) {
        self.total += delta;
        if self.total > 1e123 {  // umbral arbitrario antes de heat death
            println!("¡Alerta! Heat death inminente. Inyectando foam...");
            self.total *= 0.999;  // "reset" suave, como si nada
        }
    }

    pub fn is_stable(&self) -> bool {
        self.total < 1e123
    }
}
