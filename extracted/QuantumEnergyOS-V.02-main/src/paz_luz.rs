// src/paz_luz.rs
use std::io::{self, Write};
use std::thread;
use std::time::Duration;

const MANTRA: &str = "On abokya beiroshanou makabodara mani handoma jinbara harabari taya un";

pub struct SanacionBridge {
    energia: f64,          // 0.0 a 1.0 —paz interna
    respiraciones: u32,
}

impl SanacionBridge {
    pub fn new() -> Self {
        SanacionBridge { energia: 0.0, respiraciones: 0 }
    }

    /// Repite el mantra en terminal con "luz" visual (ondas ASCII)
    pub fn recitar_mantra(&mut self, reps: u32) {
        println!("\n ");
        for i in 0..reps {
            print!("\r | Energía: {:.2} ", MANTRA, self.energia);
            io::stdout().flush().unwrap();

            // Simula luz: onda que crece
            let onda = "🌟".repeat((self.energia * 10.0) as usize);
            println!("\n{}", onda);

            // "Respiración" —aumenta paz
            self.energia += 0.05;
            if self.energia > 1.0 { self.energia = 1.0; }
            self.respiraciones += 1;

            thread::sleep(Duration::from_millis(800));
        }
        println!("\nPaz alcanzada. Energía: {:.2} —¡Om! 🌟", self.energia);
    }

    /// Sanación: "biofeedback" fake —baja estrés con tiempo
    pub fn canalizar_luz(&mut self, minutos: u64) {
        println!("Canalizando luz... manos sobre chakra.");
        for _ in 0..minutos {
            println!("Luz dorada fluye: {}", "✨".repeat(5));
            self.energia += 0.1;
            thread::sleep(Duration::from_secs(60)); // 1 min real
        }
        println!("Sanado. Energía: {:.2} —todo en equilibrio.", self.energia);
    }
}

// Ejemplo uso
fn main() {
    let mut paz = SanacionBridge::new();
    paz.recitar_mantra(5);          // 5 repeticiones del mantra
    paz.canalizar_luz(2);           // 2 minutos de "sanación"
}
