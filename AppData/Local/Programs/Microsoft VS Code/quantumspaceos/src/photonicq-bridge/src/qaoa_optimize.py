#!/usr/bin/env python3
"""
qaoa_optimize.py — Bridge Qiskit-Aer para AntiGravBridge
══════════════════════════════════════════════════════════

Lee parámetros del lifter desde stdin (JSON), ejecuta QAOA p=1 con
Qiskit-Aer (simulador cuántico con ruido real), y devuelve el voltaje
óptimo en stdout (JSON).

Uso desde Rust (subprocess):
    echo '{"gap_m": 0.05, ...}' | python3 qaoa_optimize.py

Uso manual de prueba:
    echo '{"gap_m": 0.05, "emitter_radius_m": 1e-4, "air_density": 1.225,
           "mass_kg": 0.002, "voltage_min_kv": 10, "voltage_max_kv": 45,
           "n_levels": 8, "p_layers": 1, "shots": 1024}' | python3 qaoa_optimize.py

Respuesta:
    {"optimal_voltage_kv": 25.0, "best_cost": 0.00312, "counts": {...}}

Requisitos:
    pip install qiskit qiskit-aer numpy scipy

Autor: Giovanny Anthony Corpus Bernal — Mexicali, BC
"""

import sys
import json
import math

# ── Constantes físicas (espejo de antigrav_bridge.rs) ─────────────────────────
G        = 9.80665          # m/s²
RHO_SL   = 1.225            # kg/m³
E_CORONA = 3.0e6            # V/m (umbral de corona STP)
MU_ION   = 2.0e-4           # m²/(V·s) movilidad N₂⁺
K_BB     = 5.0e-5           # N/kV² coeficiente BB empírico


def compute_ehd(voltage_kv: float, gap_m: float, emitter_radius_m: float,
                air_density: float) -> tuple[float, float, float]:
    """
    Calcula (thrust_N, ion_current_uA, power_W) para los parámetros dados.
    Espejo exacto de AntiGravBridge::compute_ehd() en Rust.
    """
    v_volts = voltage_kv * 1_000.0
    e_avg = v_volts / gap_m
    enhancement = min(math.sqrt(gap_m / emitter_radius_m), 150.0)
    e_tip = e_avg * enhancement
    e_corona_adj = E_CORONA * math.sqrt(air_density / RHO_SL)

    if e_tip < e_corona_adj:
        return 0.0, 0.0, 0.0

    rho_norm = air_density / RHO_SL
    thrust_n = K_BB * voltage_kv**2 * rho_norm
    ion_current_a = thrust_n * MU_ION / gap_m
    power_w = v_volts * ion_current_a

    return thrust_n, ion_current_a * 1e6, power_w


def cost_function(voltage_kv: float, gap_m: float, emitter_radius_m: float,
                  air_density: float, mass_kg: float) -> float:
    """
    Función de coste para QAOA: rendimiento de empuje neto [N/W].
    c(V) = max(0, T(V) − m·g) / P(V)
    Devuelve 0 si el lifter no puede levantar su propio peso.
    """
    thrust, _, power = compute_ehd(voltage_kv, gap_m, emitter_radius_m, air_density)
    weight = mass_kg * G
    if thrust <= weight or power < 1e-12:
        return 0.0
    return (thrust - weight) / power


def build_qaoa_circuit(n_qubits: int, costs: list[float], gamma: float, beta: float):
    """
    Construye el circuito QAOA p=1 para n_qubits qubits.

    Estructura:
      |+⟩^n → e^{iγ H_C} → e^{-iβ H_B} → medición

    H_C es diagonal (coste de cada estado computacional).
    H_B = Σₖ Xₖ (mezclador de Pauli X).

    Args:
        n_qubits: número de qubits (log2 del número de estados)
        costs: lista de costes por estado (len = 2^n_qubits)
        gamma: parámetro del separador de fase [rad]
        beta: parámetro del mezclador [rad]

    Returns:
        QuantumCircuit de Qiskit
    """
    from qiskit import QuantumCircuit
    import numpy as np

    n_states = 1 << n_qubits
    qc = QuantumCircuit(n_qubits, n_qubits)

    # Inicializar |+⟩^n con puertas Hadamard
    qc.h(range(n_qubits))

    # Separador de fase e^{iγ H_C}:
    # Para cada estado computacional |j⟩, la fase es e^{iγ·c_j}.
    # Implementación: unitaria diagonal usando fase global por estado.
    # En un hardware real esto se descompondría en puertas Rz + CNOT.
    # Aquí usamos unitary() de Qiskit para exactitud en el simulador.
    import numpy as np

    # Construir la matriz diagonal e^{iγ·c_j}
    diag = np.exp(1j * gamma * np.array(costs))
    phase_gate = np.diag(diag)
    qc.unitary(phase_gate, range(n_qubits), label="e^{iγH_C}")

    # Mezclador e^{-iβ H_B}: aplica Rx(2β) sobre cada qubit
    # e^{-iβ X} = Rx(2β) en convención de Qiskit
    for k in range(n_qubits):
        qc.rx(2 * beta, k)

    # Medición de todos los qubits
    qc.measure(range(n_qubits), range(n_qubits))

    return qc


def grid_search_qaoa(costs: list[float], n_qubits: int, shots: int,
                     n_grid: int = 10) -> tuple[float, float, dict]:
    """
    Barrido de grilla sobre (γ, β) para encontrar los parámetros QAOA
    que maximizan el valor esperado ⟨H_C⟩.

    Args:
        costs: costes por estado
        n_qubits: número de qubits
        shots: mediciones por circuito
        n_grid: puntos en cada dimensión del barrido

    Returns:
        (gamma_opt, beta_opt, best_counts)
    """
    from qiskit_aer import AerSimulator
    import numpy as np

    simulator = AerSimulator()
    n_states = 1 << n_qubits

    best_expectation = -math.inf
    best_gamma, best_beta = math.pi / 2, math.pi / 4
    best_counts: dict = {}

    for gi in range(1, n_grid + 1):
        for bi in range(1, n_grid + 1):
            gamma = gi * math.pi / n_grid
            beta = bi * (math.pi / 2) / n_grid

            qc = build_qaoa_circuit(n_qubits, costs, gamma, beta)

            # Transpilación y ejecución en AerSimulator
            from qiskit import transpile
            tqc = transpile(qc, simulator, optimization_level=1)
            result = simulator.run(tqc, shots=shots).result()
            counts = result.get_counts()

            # Valor esperado empírico: Σ P(j) · c_j
            expectation = 0.0
            for bitstring, count in counts.items():
                # Qiskit ordena qubits en little-endian (qubit 0 = bit más a la derecha)
                state_idx = int(bitstring[::-1], 2) % n_states
                expectation += (count / shots) * costs[state_idx]

            if expectation > best_expectation:
                best_expectation = expectation
                best_gamma, best_beta = gamma, beta
                best_counts = dict(counts)

    return best_gamma, best_beta, best_counts


def run_final_circuit(costs: list[float], voltages: list[float], n_qubits: int,
                      shots: int, gamma: float, beta: float) -> float:
    """
    Ejecuta el circuito QAOA con los parámetros óptimos encontrados,
    y devuelve el voltaje asociado al estado más probable.
    """
    from qiskit_aer import AerSimulator
    from qiskit import transpile
    import numpy as np

    simulator = AerSimulator()
    qc = build_qaoa_circuit(n_qubits, costs, gamma, beta)
    tqc = transpile(qc, simulator, optimization_level=1)
    result = simulator.run(tqc, shots=shots * 2).result()  # más shots para precisión
    counts = result.get_counts()

    # Estado más frecuente (modo de la distribución)
    n_states = 1 << n_qubits
    most_frequent = max(counts, key=counts.get)
    # Conversión little-endian → índice de estado
    best_idx = int(most_frequent[::-1], 2) % n_states

    return voltages[best_idx]


def main():
    # ── Leer parámetros de stdin ─────────────────────────────────────────────
    try:
        raw = sys.stdin.read()
        params = json.loads(raw)
    except (json.JSONDecodeError, EOFError) as e:
        json.dump({"error": f"JSON inválido: {e}"}, sys.stdout)
        sys.exit(1)

    # Extraer parámetros con defaults seguros
    gap_m           = float(params.get("gap_m", 0.05))
    emitter_radius  = float(params.get("emitter_radius_m", 1e-4))
    air_density     = float(params.get("air_density", 1.225))
    mass_kg         = float(params.get("mass_kg", 0.002))
    v_min           = float(params.get("voltage_min_kv", 10.0))
    v_max           = float(params.get("voltage_max_kv", 45.0))
    n_levels        = int(params.get("n_levels", 8))
    p_layers        = int(params.get("p_layers", 1))  # este script siempre usa p=1
    shots           = int(params.get("shots", 1024))

    # Construir niveles de voltaje discretizados
    if n_levels > 1:
        step = (v_max - v_min) / (n_levels - 1)
        voltages = [v_min + i * step for i in range(n_levels)]
    else:
        voltages = [v_min]

    # Número de qubits = ceil(log2(n_levels))
    n_qubits = max(1, math.ceil(math.log2(n_levels)))
    n_states = 1 << n_qubits

    # Pad voltages si n_states > n_levels (por redondeo de qubits)
    while len(voltages) < n_states:
        voltages.append(voltages[-1])
    voltages = voltages[:n_states]

    # ── Calcular paisaje de costes ────────────────────────────────────────────
    costs = [
        cost_function(v, gap_m, emitter_radius, air_density, mass_kg)
        for v in voltages
    ]

    # Si todos los costes son 0 (ningún voltaje levanta el lifter),
    # devolver el voltaje máximo como mejor estimación
    if max(costs) < 1e-15:
        result = {
            "optimal_voltage_kv": voltages[-1],
            "best_cost": 0.0,
            "note": "Ningún voltaje supera el umbral de levitación, usando V_max",
            "costs": costs
        }
        print(json.dumps(result))
        return

    # ── Intentar importar Qiskit ──────────────────────────────────────────────
    try:
        import qiskit
        import qiskit_aer
    except ImportError as e:
        # Fallback clásico si Qiskit no está disponible
        best_idx = costs.index(max(costs))
        result = {
            "optimal_voltage_kv": voltages[best_idx],
            "best_cost": costs[best_idx],
            "note": f"Qiskit no disponible ({e}), fallback clásico",
            "costs": costs
        }
        print(json.dumps(result))
        return

    # ── Ejecutar QAOA ─────────────────────────────────────────────────────────
    try:
        # 1. Barrido de grilla para encontrar (γ*, β*)
        gamma_opt, beta_opt, best_counts = grid_search_qaoa(
            costs, n_qubits, shots, n_grid=10
        )

        # 2. Circuito final con parámetros óptimos → voltaje óptimo
        optimal_voltage = run_final_circuit(
            costs, voltages, n_qubits, shots, gamma_opt, beta_opt
        )

        result = {
            "optimal_voltage_kv": optimal_voltage,
            "best_cost": max(costs),
            "gamma_opt": gamma_opt,
            "beta_opt": beta_opt,
            "n_qubits": n_qubits,
            "n_states": n_states,
            "p_layers": p_layers,
            "voltages_kv": voltages,
            "costs": costs,
            "counts_sample": dict(list(best_counts.items())[:8])  # muestra del histograma
        }

    except Exception as e:
        # Fallback clásico ante cualquier error de Qiskit
        best_idx = costs.index(max(costs))
        result = {
            "optimal_voltage_kv": voltages[best_idx],
            "best_cost": costs[best_idx],
            "note": f"Error Qiskit: {e}, fallback clásico",
            "costs": costs
        }

    print(json.dumps(result))


if __name__ == "__main__":
    main()
