//! ╔══════════════════════════════════════════════════════════════════════════╗
//! ║  AntiGravBridge — Demo 200 Ticks                                        ║
//! ║  QuantumEnergyOS V.02 — Mexicali, BC                                    ║
//! ╚══════════════════════════════════════════════════════════════════════════╝
//!
//! Ejecutar:
//!   cargo run --bin antigrav-demo
//!   cargo run --bin antigrav-demo --features dashboard   (con POST HTTP)
//!   cargo run --bin antigrav-demo --release

// Importar el módulo de la librería
use antigrav_bridge::{AntiGravBridge, Electrode};

fn main() {
    print_banner();

    // ── Configuración del lifter ───────────────────────────────────────────
    //
    // Usamos el constructor estándar de 2g con voltaje inicial de 30 kV.
    // El QAOA ajustará automáticamente el voltaje cada 50 ticks hacia
    // el nivel óptimo de eficiencia.
    let mut bridge = AntiGravBridge::new(30.0);

    // Mostrar configuración inicial
    println!("  Voltaje inicial  : {:.0} kV", bridge.voltage_kv);
    println!("  Masa del lifter  : {:.0} g", bridge.mass_kg * 1000.0);
    println!("  Gap inter-electr.: {:.0} mm", bridge.gap_m * 1000.0);
    println!("  Radio emisor     : {:.2} mm", bridge.emitter_radius_m * 1000.0);
    println!("  QAOA cada        : {} ticks", bridge.qaoa_interval_ticks);
    println!("  dt = 50 ms | 200 ticks = 10 s de simulación\n");
    println!("{}", "─".repeat(90));
    println!(
        "{:<6} | {:<7} | {:<10} | {:<9} | {:<8} | {:<8} | {:<7} | {:<7} | {}",
        "Tick", "V [kV]", "Thrust [N]", "ΔF [N]", "Alt [m]", "Vel[m/s]",
        "I [μA]", "P [W]", "Estado"
    );
    println!("{}", "─".repeat(90));

    let dt = 0.05; // 50 ms por tick = 20 Hz

    for _tick in 0..200 {
        bridge.update(dt);

        // ── Mostrar estado cada 5 ticks ────────────────────────────────
        if bridge.tick % 5 == 0 {
            let t = bridge.telemetry();
            println!(
                "{:<6} | {:>6.1}  | {:>9.5} | {:>+8.5} | {:>7.4} | {:>7.4} | {:>6.2} | {:>6.2} | {}",
                t.tick,
                t.voltage_kv,
                t.thrust_magnitude_n,
                t.net_force_n,
                t.altitude_m,
                t.vel_ms[2],  // componente vertical
                t.ion_current_ua,
                t.power_w,
                t.status,
            );
        }

        // ── Eventos especiales en puntos clave ─────────────────────────

        // Tick 50: primer QAOA + MZI + Grid
        if bridge.tick == 50 {
            println!("\n{}", "═".repeat(90));
            println!("  ⚡ PRIMER CICLO QAOA (tick 50)");
            println!("     Voltaje QAOA óptimo : {:.0} kV", bridge.qaoa_voltage_kv);
            let mzi = bridge.pulse_control();
            println!(
                "  🔬 Control MZI: φ={:.4} rad | Trans={:.4} | P_óptica={:.3} mW | f={:.2e} Hz",
                mzi.phase_shift_rad, mzi.transmittance, mzi.optical_power_mw, mzi.pulse_freq_hz
            );
            let grid = bridge.balance_grid();
            println!(
                "  ⚖️  Grid balancer: consumo={:.3} W | efic={:.5} N/W | prioridad_QAOA={:.3} | P_hover={:.3} W",
                grid.consumption_w, grid.conversion_efficiency_n_per_w,
                grid.qaoa_priority, grid.min_hover_power_w
            );
            // Enviar al dashboard (stub JSON si no hay feature dashboard)
            match bridge.send_to_dashboard() {
                Ok(_) => println!("  📡 Telemetría enviada al dashboard"),
                Err(e) => println!("  📡 Dashboard (stub): {}", e),
            }
            println!("{}", "═".repeat(90));
        }

        // Tick 100: snapshot a media simulación + test de electrodo personalizado
        if bridge.tick == 100 {
            println!("\n{}", "═".repeat(90));
            println!("  📊 SNAPSHOT TICK 100 — MEDIA SIMULACIÓN");
            let tel = bridge.telemetry();
            println!("     Altitud acumulada: {:.4} m", tel.altitude_m);
            println!("     Velocidad Z: {:.4} m/s", tel.vel_ms[2]);
            println!("     Eficiencia: {:.3} mN/W", tel.efficiency_mn_per_w);
            println!("     Densidad aire: {:.5} kg/m³", tel.air_density_kgm3);
            println!("     Voltaje actual (post-QAOA): {:.0} kV", tel.voltage_kv);
            println!("{}", "═".repeat(90));
        }

        // Tick 150: segundo QAOA + análisis de convergencia
        if bridge.tick == 150 {
            println!("\n{}", "═".repeat(90));
            println!("  ⚡ SEGUNDO CICLO QAOA (tick 150)");
            println!("     Voltaje QAOA óptimo : {:.0} kV", bridge.qaoa_voltage_kv);
            let mzi = bridge.pulse_control();
            println!(
                "  🔬 Control MZI: φ={:.4} rad | Trans={:.4} | P_óptica={:.3} mW",
                mzi.phase_shift_rad, mzi.transmittance, mzi.optical_power_mw
            );
            let grid = bridge.balance_grid();
            println!(
                "  ⚖️  Grid: consumo={:.3} W | nodo=#{} | prioridad_QAOA={:.3}",
                grid.consumption_w, grid.node_id, grid.qaoa_priority
            );
            println!("{}", "═".repeat(90));
        }
    }

    println!("{}", "─".repeat(90));

    // ── Telemetría final ──────────────────────────────────────────────────
    println!("\n{}", "═".repeat(90));
    println!("  🏁 TELEMETRÍA FINAL — 200 TICKS COMPLETADOS");
    println!("{}", "═".repeat(90));
    let final_tel = bridge.telemetry();
    let json = serde_json::to_string_pretty(&final_tel)
        .unwrap_or_else(|_| "{}".to_string());
    println!("{}", json);

    // ── Demo de electrodo personalizado ───────────────────────────────────
    println!("\n{}", "═".repeat(90));
    println!("  🔧 DEMO: Lifter con geometría personalizada (gap 8cm, 1.5g)");
    println!("{}", "═".repeat(90));

    let em2 = Electrode::emitter([0.0, 0.0, 0.0]);
    let col2 = Electrode::collector([0.0, 0.0, 0.08]); // gap 8 cm
    let mut custom = AntiGravBridge::with_electrodes(
        35.0,
        vec![em2, col2],
        0.0015, // 1.5g
    );
    custom.qaoa_interval_ticks = 25; // QAOA más frecuente

    for _ in 0..50 {
        custom.update(0.05);
    }
    println!("{}", custom.get_status());
    let mzi_c = custom.pulse_control();
    println!(
        "  MZI: φ={:.4}rad | Trans={:.4} | P_opt={:.3}mW | f={:.3e}Hz",
        mzi_c.phase_shift_rad, mzi_c.transmittance,
        mzi_c.optical_power_mw, mzi_c.pulse_freq_hz
    );

    println!("\n{}", "═".repeat(90));
    println!("  ✅ Simulación completada — QuantumEnergyOS V.02 AntiGravBridge");
    println!("  Mexicali, BC — Rompiendo límites. Kardashev 0→1");
    println!("{}", "═".repeat(90));
}

fn print_banner() {
    println!();
    println!("  ╔══════════════════════════════════════════════════════════════╗");
    println!("  ║   QuantumEnergyOS V.02 — AntiGravBridge Demo               ║");
    println!("  ║   Simulación EHD Biefeld-Brown + QAOA + MZI Fotónico       ║");
    println!("  ║   200 ticks × 50ms = 10 segundos de vuelo                  ║");
    println!("  ╚══════════════════════════════════════════════════════════════╝");
    println!();
    println!("  Física:");
    println!("    · Empuje EHD: T = K_BB · V² · ρ  [Biefeld-Brown empírico]");
    println!("    · Corona: E_tip = V/d · √(d/r) > 3 MV/m · √(ρ/ρ₀)");
    println!("    · QAOA p=1 sobre 3 qubits → 8 niveles de voltaje [10..45 kV]");
    println!("    · MZI: φ = π·T/T_max → Trans = cos²(φ/2) [1550 nm]");
    println!("    · Grid: carga virtual para balanceador energético QAOA");
    println!();
}
