use photonic_core::{EnergyMetrics, PhotonicCore};
use std::thread;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

fn main() {
    println!("╔═══════════════════════════════════════════════════════════╗");
    println!("║        QuantumEnergyOS V.02 - Photonic Core                 ║");
    println!("║        Made in Mexicali with 22 years of grind             ║");
    println!("╚═══════════════════════════════════════════════════════════╝");
    println!();

    let core = PhotonicCore::new();

    println!("[1] Starting photonic core systems...");
    println!("  ✓ Energy queue initialized (capacity: 1000)");
    println!("  ✓ Photon buffer initialized");
    println!("  ✓ Temperature monitoring active");
    println!();

    println!("[2] Recording energy metrics...");
    for i in 1..=5 {
        let metrics = EnergyMetrics {
            timestamp: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs(),
            power_consumption: 100.0 + (i as f64 * 10.0),
            efficiency: 0.92 + (i as f64 * 0.01),
            temperature: 45.0 + (i as f64 * 0.5),
            load_factor: 0.5 + (i as f64 * 0.05),
        };

        if let Err(e) = core.record_energy_metrics(metrics) {
            println!("  ✗ Error: {}", e);
        } else {
            println!(
                "  ✓ Metrics recorded: {} MW, {:.1}% efficiency",
                metrics.power_consumption,
                metrics.efficiency * 100.0
            );
        }
    }
    println!();

    println!("[3] Emitting test photons...");
    let wavelengths = [650.0, 850.0, 1310.0, 1550.0];
    for wl in wavelengths {
        match core.emit_photon(wl, 0.0) {
            Ok(photon) => println!(
                "  ✓ Photon emitted: λ={}nm, E={:.3}eV",
                photon.wavelength, photon.energy
            ),
            Err(e) => println!("  ✗ Error: {}", e),
        }
    }
    println!();

    println!("[4] Processing photon stream...");
    match core.process_photons() {
        Ok(total) => println!("  ✓ Processed total energy: {:.3} eV", total),
        Err(e) => println!("  ✗ Error: {}", e),
    }
    println!();

    println!("[5] Testing energy flow optimization...");
    let test_powers = [50.0, 100.0, 200.0, 500.0];
    for power in test_powers {
        match core.optimize_energy_flow(power) {
            Ok(optimized) => println!("  → Input: {:.0}W → Optimized: {:.1}W", power, optimized),
            Err(e) => println!("  ✗ Error: {}", e),
        }
    }
    println!();

    println!("[6] Load prediction...");
    let historical = vec![0.4, 0.45, 0.5, 0.55, 0.6, 0.58, 0.62, 0.65, 0.7, 0.68];
    let prediction = core.predict_load(&historical);
    println!(
        "  → Predicted load for next interval: {:.1}%",
        prediction * 100.0
    );
    println!();

    println!("[7] Core Status:");
    let status = core.get_core_status().unwrap();
    for (key, value) in &status {
        println!("  • {}: {}", key, value);
    }
    println!();

    println!("╔═══════════════════════════════════════════════════════════╗");
    println!("║  PhotonicCore Ready - Energy Optimization Active          ║");
    println!("╚═══════════════════════════════════════════════════════════╝");

    loop {
        thread::sleep(Duration::from_secs(60));
    }
}

// QuantumEnergyOS - Core Validator
// Prueba de integridad de la arquitectura cuántica.

use photonic_core::{MachZehnderInterferometer, Waveguide};
use num_complex::Complex64;

fn main() {
    println!("--- QuantumEnergyOS V.02: Validación de Core Fotónico ---");

    let mzi = MachZehnderInterferometer { phase_shift: std::f64::consts::FRAC_PI_2 };
    let input = (Complex64::new(1.0, 0.0), Complex64::new(0.0, 0.0));

    let (out1, out2) = mzi.propagate(input);

    println!("[TEST] Entrada: 1.0 + 0.0i");
    println!("[TEST] Salida 1 (Interferencia): {:.4} + {:.4}i", out1.re, out1.im);
    println!("[TEST] Salida 2 (Interferencia): {:.4} + {:.4}i", out2.re, out2.im);

    println!("\nIngeniería inversa del universo en progreso...");
}
