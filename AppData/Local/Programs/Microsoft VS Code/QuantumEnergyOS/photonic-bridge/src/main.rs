use photonic_bridge::{PhotonicSignal, QuantumBridge};
use std::thread;
use std::time::Duration;

fn main() {
    println!("╔═══════════════════════════════════════════════════════════╗");
    println!("║        QuantumEnergyOS V.02 - Photonic Bridge              ║");
    println!("║        Made in Mexicali with 22 years of grind            ║");
    println!("╚═══════════════════════════════════════════════════════════╝");
    println!();

    let bridge = QuantumBridge::new();

    println!("[1] Initializing quantum state...");
    match bridge.initialize_qubit(0) {
        Ok(state) => println!(
            "  ✓ Qubit 0 initialized (coherence: {:.3})",
            state.coherence
        ),
        Err(e) => println!("  ✗ Error: {}", e),
    }
    match bridge.initialize_qubit(1) {
        Ok(state) => println!(
            "  ✓ Qubit 1 initialized (coherence: {:.3})",
            state.coherence
        ),
        Err(e) => println!("  ✗ Error: {}", e),
    }
    println!();

    println!("[2] Applying quantum gates...");
    if let Err(e) = bridge.apply_gate(0, "H") {
        println!("  ✗ Error applying H gate: {}", e);
    } else {
        println!("  ✓ Hadamard gate applied to Qubit 0");
    }
    if let Err(e) = bridge.apply_gate(1, "X") {
        println!("  ✗ Error applying X gate: {}", e);
    } else {
        println!("  ✓ Pauli-X gate applied to Qubit 1");
    }
    println!();

    println!("[3] Creating quantum entanglement...");
    match bridge.entangle_qubits(0, 1) {
        Ok(_) => println!("  ✓ Qubits 0 and 1 are now entangled"),
        Err(e) => println!("  ✗ Error: {}", e),
    }
    println!();

    println!("[4] Sending photonic signal...");
    let signal = PhotonicSignal {
        frequency: 193.414e12,
        amplitude: 1.0,
        phase: 0.0,
        wavelength: 1550.0,
        polarization: "TE".to_string(),
    };
    match bridge.send_photonic_signal(signal) {
        Ok(_) => println!("  ✓ Photonic signal transmitted at 1550nm"),
        Err(e) => println!("  ✗ Error: {}", e),
    }
    println!();

    println!("[5] Energy prediction...");
    let load_factor = 0.75;
    let prediction = bridge.predict_energy(load_factor);
    println!(
        "  → Predicted energy load at {:.0}%: {:.2} MW",
        load_factor * 100.0,
        prediction
    );
    println!();

    println!("[6] Distribution optimization...");
    let nodes = vec![1, 2, 3, 4, 5];
    let allocation = bridge.optimize_distribution(&nodes);
    println!("  → Optimized energy distribution:");
    for (node, energy) in &allocation {
        println!("    Node {}: {:.2} MW", node, energy);
    }
    println!();

    println!("[7] Bridge Status:");
    let status = bridge.get_bridge_status();
    for (key, value) in &status {
        println!("  • {}: {}", key, value);
    }
    println!();

    println!("╔═══════════════════════════════════════════════════════════╗");
    println!("║  QuantumBridge Ready - Prevention of Blackouts Active     ║");
    println!("╚═══════════════════════════════════════════════════════════╝");

    loop {
        thread::sleep(Duration::from_secs(60));
    }
}
