// src/antigrav_bridge.rs
use std::f64::consts::PI;

# struct Electrode {
    pos: (f64, f64),      // posición x,y
    charge: f64,          // +1 o -1 (positivo pequeño = thrust up)
    radius: f64,          // pequeño = corona fuerte
}

# pub struct AntiGravSim {
    electrodes: Vec<Electrode>,
    voltage_kv: f64,      // 10..50 kV típico
    air_density: f64,     // kg/m³, baja = menos thrust
    thrust: f64,          // fuerza neta (N)
    pos: (f64, f64),      // posición del lifter
    vel: (f64, f64),      // velocidad
}

impl AntiGravSim {
    pub fn new(voltage_kv: f64) -> Self {
        let small_pos = Electrode { pos: (0.0, 0.0), charge: 1.0, radius: 0.001 }; // aguja positiva
        let large_pos = Electrode { pos: (0.0, 0.05), charge: -1.0, radius: 0.02 }; // placa negativa

        AntiGravSim {
            electrodes: vec! ,
            voltage_kv,
            air_density: 1.225,
            thrust: 0.0,
            pos: (0.0, 0.0),
            vel: (0.0, 0.0),
        }
    }

    /// Simula un tick: calcula campo E, ioniza, thrust ≈ I * d / (2π ε0) simplificado
    pub fn update(&mut self, dt: f64) {
        let delta_y = self.electrodes[1].pos.1 - self.electrodes[0].pos.1;
        let e_field = self.voltage_kv * 1000.0 / delta_y; // V/m ≈ 10^6 V/m

        // Corona threshold ~3e6 V/m, pero usamos approx para thrust
        let ion_current = 1e-6 * e_field.powi(2); // A, rough empirical
        let momentum_transfer = ion_current * 1e-19 * 1e5; // masa ion * vel aprox

        // Thrust neto hacia placa negativa (arriba si invertimos y)
        self.thrust = momentum_transfer * self.air_density.sqrt(); // kg m/s²

        // Aplicar fuerza (contra gravedad, digamos 9.81 * masa)
        let accel_y = self.thrust / 0.05 - 9.81; // masa lifter ~50g
        self.vel.1 += accel_y * dt;
        self.pos.1 += self.vel.1 * dt;

        if self.pos.1 < 0.0 { self.pos.1 = 0.0; self.vel.1 = 0.0; } // suelo
    }

    pub fn get_status(&self) -> String {
        format!(
            "Thrust: {:.3} N | Altura: {:.2} m | Vel: {:.2} m/s | Voltaje: {} kV",
            self.thrust, self.pos.1, self.vel.1, self.voltage_kv
        )
    }
}

// Ejemplo uso en main o kernel
fn main() {
    let mut sim = AntiGravSim::new(30.0);
    for _ in 0..100 {
        sim.update(0.01);
        println!("{}", sim.get_status());
    }
}
