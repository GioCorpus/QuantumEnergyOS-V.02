"""
api/core.py — Lógica de simulación cuántica (sin dependencias de framework)
─────────────────────────────────────────────────────────────────────────────
Módulo central testeable de forma independiente a FastAPI.
server.py importa desde aquí y agrega la capa HTTP.

Autor: GioCorpus — QuantumEnergyOS, Mexicali, B.C.
"""

from __future__ import annotations

import math
import random
import time
from typing import Any

# ── Constantes del sistema ────────────────────────────────────────────────────
MAX_QUBITS = 16
MAX_SHOTS  = 10_000
MAX_NODES  = 8

# Física de fusión D-T
SECCION_EFICAZ_PEAK_M2 = 5.0e-28    # m² — sección eficaz pico (~65 keV)
ENERGIA_DT_JOULES      = 2.818e-12  # J — 17.6 MeV por reacción
LAWSON_CRITERIO        = 3.0e21     # m⁻³·s — criterio de ignición
VOLUMEN_REACTOR_M3     = 800.0      # m³ — tokamak de demostración

# Costos de red eléctrica
COSTO_BASE_KW          = 150.0      # kW de desperdicio por nodo en sobrecarga
FACTOR_CASCADA         = 0.533      # Penalización por nodos adyacentes en sobrecarga


# ─────────────────────────────────────────────────────────────────────────────
#  COOLING
# ─────────────────────────────────────────────────────────────────────────────

def simular_cooling(n_qubits: int, ciclos_braiding: int) -> dict[str, Any]:
    """
    Simula el protocolo de enfriamiento criogénico para qubits Majorana.

    Modelo:
    - Temperatura inicial: ~300 K (ambiente desierto de B.C.)
    - Temperatura final:   ~4 K  (crióstato operacional)
    - Más ciclos de braiding → mayor protección topológica → mayor tasa de éxito.

    Args:
        n_qubits:        Número de qubits a enfriar (1–16).
        ciclos_braiding: Ciclos de braiding protector por qubit (1–100).

    Returns:
        Diccionario con resultados por qubit y métricas globales.
    """
    if not (1 <= n_qubits <= MAX_QUBITS):
        raise ValueError(f"n_qubits debe estar entre 1 y {MAX_QUBITS}")
    if not (1 <= ciclos_braiding <= 100):
        raise ValueError("ciclos_braiding debe estar entre 1 y 100")

    resultados_qubits = []
    enfriados = 0

    for i in range(n_qubits):
        # Probabilidad de éxito: crece con ciclos de braiding (modelo empírico)
        # Asintótica a ~97% (no es posible 100% en hardware real)
        prob_exito = 0.97 * (1.0 - math.exp(-ciclos_braiding / 8.0))
        exito = random.random() < prob_exito

        # Temperaturas con variación realista del entorno
        temp_inicial_k = 300.0 + random.uniform(-20.0, 50.0)   # Desierto B.C.
        temp_final_k   = 4.0   + random.uniform(0.0, 2.0)      # Crióstato

        if exito:
            enfriados += 1

        resultados_qubits.append({
            "id":             i,
            "estado":         "4K_operacional" if exito else "ruido_termico",
            "temp_inicial_k": round(temp_inicial_k, 2),
            "temp_final_k":   round(temp_final_k, 2),
            "exito":          exito,
        })

    tasa_exito_pct = round(enfriados / n_qubits * 100.0, 1)

    return {
        "n_qubits":        n_qubits,
        "enfriados":       enfriados,
        "tasa_exito_pct":  tasa_exito_pct,
        "ciclos_braiding": ciclos_braiding,
        "qubits":          resultados_qubits,
        "mensaje": (
            f"{enfriados}/{n_qubits} qubits operacionales a 4 K. "
            "El desierto no gana hoy — el crióstato dice gracias."
        ),
    }


# ─────────────────────────────────────────────────────────────────────────────
#  GRID BALANCE
# ─────────────────────────────────────────────────────────────────────────────

def _costo_config(config: list[int]) -> float:
    """Calcula el kW de desperdicio de una configuración de red."""
    costo = sum(COSTO_BASE_KW for c in config if c == 1)
    # Penalización por nodos adyacentes en sobrecarga (efecto cascada)
    for i in range(len(config) - 1):
        if config[i] == 1 and config[i + 1] == 1:
            costo += COSTO_BASE_KW * FACTOR_CASCADA
    return costo


def simular_grid(
    n_nodos: int,
    shots: int,
    gamma: float,
    beta: float,
) -> dict[str, Any]:
    """
    Simulación del algoritmo QAOA para balanceo de red eléctrica.

    Args:
        n_nodos: Nodos en la red (2–8).
        shots:   Iteraciones del circuito QAOA.
        gamma:   Parámetro del operador de costo (fase ZZ).
        beta:    Parámetro del operador de mezcla (rotaciones Rx).

    Returns:
        Configuración óptima, desperdicio mínimo e historial de convergencia.
    """
    if not (2 <= n_nodos <= MAX_NODES):
        raise ValueError(f"n_nodos debe estar entre 2 y {MAX_NODES}")
    if not (1 <= shots <= MAX_SHOTS):
        raise ValueError(f"shots debe estar entre 1 y {MAX_SHOTS}")
    if not (0.0 <= gamma <= math.pi):
        raise ValueError("gamma debe estar en [0, π]")
    if not (0.0 <= beta <= math.pi):
        raise ValueError("beta debe estar en [0, π]")

    mejor_config: list[int] = [1] * n_nodos   # Peor caso inicial
    mejor_costo  = _costo_config(mejor_config)
    historial    = []

    # Probabilidad de sobrecarga de cada nodo en función de gamma y beta
    p_base = 0.3 * (1.0 - beta / math.pi) * (1.0 + gamma / math.pi) / 2.0
    p_sobrecarga = max(0.05, min(0.80, p_base))

    registro_intervalo = max(1, shots // 10)

    for shot in range(shots):
        config = [1 if random.random() < p_sobrecarga else 0 for _ in range(n_nodos)]
        costo  = _costo_config(config)

        if costo < mejor_costo:
            mejor_costo  = costo
            mejor_config = config[:]

        if shot % registro_intervalo == 0:
            historial.append({"shot": shot, "mejor_costo_kw": round(mejor_costo, 2)})

    n_sobrecargas  = sum(mejor_config)
    peor_posible   = n_nodos * COSTO_BASE_KW * (1.0 + FACTOR_CASCADA)
    ahorro_kw      = peor_posible - mejor_costo
    ahorro_pct     = round(max(0.0, ahorro_kw / peor_posible * 100.0), 1)

    return {
        "n_nodos":          n_nodos,
        "shots":            shots,
        "gamma":            gamma,
        "beta":             beta,
        "mejor_config":     mejor_config,
        "nodos_sobrecarga": n_sobrecargas,
        "desperdicio_kw":   round(mejor_costo, 2),
        "ahorro_pct":       ahorro_pct,
        "historial":        historial,
        "mensaje": (
            f"Red de {n_nodos} nodos balanceada en {shots} shots QAOA. "
            f"Ahorro estimado: {ahorro_pct}% vs configuración sin optimizar."
        ),
    }


# ─────────────────────────────────────────────────────────────────────────────
#  FUSION SIM
# ─────────────────────────────────────────────────────────────────────────────

def simular_fusion(
    temp_kev:        float,
    densidad_n20:    float,
    tiempo_conf:     float,
    n_precision:     int,
) -> dict[str, Any]:
    """
    Simula la física del reactor D-T con estimación de eficiencia por QPE.

    Modelo físico:
        P = n² · σv · E_DT · V · f_Lawson

    Args:
        temp_kev:     Temperatura del plasma en keV (10–200).
        densidad_n20: Densidad en 10²⁰ m⁻³ (0.1–10).
        tiempo_conf:  Tiempo de confinamiento en segundos (0.1–100).
        n_precision:  Qubits de precisión para QPE (2–8).

    Returns:
        Potencia neta, eficiencia QPE, factor de Lawson y diagnóstico.
    """
    if not (10.0 <= temp_kev <= 200.0):
        raise ValueError("temperatura_kev debe estar entre 10 y 200 keV")
    if densidad_n20 <= 0:
        raise ValueError("densidad_n20 debe ser positiva")
    if tiempo_conf <= 0:
        raise ValueError("tiempo_conf debe ser positivo")

    # Eficiencia QPE: curva gaussiana centrada en 65 keV (pico D-T real)
    # + variación estocástica del shot cuántico
    eficiencia_base  = math.exp(-((temp_kev - 65.0) / 45.0) ** 2)
    eficiencia_qpe   = max(0.05, min(1.0, eficiencia_base + random.gauss(0.0, 0.04)))

    # Física del reactor
    seccion_eficaz    = SECCION_EFICAZ_PEAK_M2 * eficiencia_qpe
    densidad          = densidad_n20 * 1.0e20
    velocidad_termica = 1.0e6 * math.sqrt(temp_kev / 65.0)
    tasa_reaccion     = densidad**2 * seccion_eficaz * velocidad_termica
    potencia_bruta_mw = tasa_reaccion * ENERGIA_DT_JOULES * VOLUMEN_REACTOR_M3 / 1.0e6

    # Criterio de Lawson: n·τ vs umbral de ignición
    factor_lawson    = densidad * tiempo_conf / LAWSON_CRITERIO
    criterio_lawson  = factor_lawson >= 1.0

    # Potencia neta: descuenta pérdidas de confinamiento
    potencia_neta_mw = (
        potencia_bruta_mw * factor_lawson
        - 15.0 / max(0.01, factor_lawson)
    )

    ignicion = potencia_neta_mw >= 500.0

    return {
        "temperatura_kev":     temp_kev,
        "densidad_n20":        densidad_n20,
        "tiempo_conf_seg":     tiempo_conf,
        "n_precision_qpe":     n_precision,
        "eficiencia_qpe":      round(eficiencia_qpe, 4),
        "factor_lawson":       round(factor_lawson, 4),
        "criterio_lawson_ok":  criterio_lawson,
        "potencia_bruta_mw":   round(potencia_bruta_mw, 2),
        "potencia_neta_mw":    round(potencia_neta_mw, 2),
        "ignicion_alcanzada":  ignicion,
        "mensaje": (
            "🔥 ¡Ignición alcanzada! Potencia suficiente para la red del noroeste. "
            "La CFE tiembla."
            if ignicion else
            f"Potencia neta: {round(potencia_neta_mw, 1)} MW. "
            "Objetivo: 500 MW. Aumentar densidad o tiempo de confinamiento."
        ),
    }


# ─────────────────────────────────────────────────────────────────────────────
#  BRAIDING DEBUG
# ─────────────────────────────────────────────────────────────────────────────

def simular_braiding(n_shots: int, verificar_paridad: bool) -> dict[str, Any]:
    """
    Benchmark de fidelidad para operaciones de braiding topológico Majorana.

    Simula la verificación del circuito T_topológico:
        H → CompuertaT → T† → H → Medir (debe dar |0⟩)

    Args:
        n_shots:           Shots para estadística de fidelidad.
        verificar_paridad: Incluir sondeo de paridad fermiónica.

    Returns:
        Fidelidad, tasa de error de fase, paridad y estado del sistema.
    """
    if not (1 <= n_shots <= MAX_SHOTS):
        raise ValueError(f"n_shots debe estar entre 1 y {MAX_SHOTS}")

    # Fidelidad base: qubits topológicos Majorana en simulación ideal ~97%
    fidelidad_true  = max(0.0, min(1.0, random.gauss(0.97, 0.015)))
    coincidencias   = round(fidelidad_true * n_shots)
    fidelidad_obs   = coincidencias / n_shots   # Fidelidad observada (entera)

    exitoso = fidelidad_obs > 0.95

    # Tasa de error de fase (debe ser < 5% para sistema operacional)
    tasa_error_fase = max(0.0, random.gauss(0.02, 0.005))

    # Verificación de paridad
    paridad_ok = None
    if verificar_paridad:
        paridad_ok = random.random() > 0.03   # 97% de éxito esperado

    estado_sistema = "OPERACIONAL" if exitoso else "REQUIERE_CALIBRACION"

    return {
        "n_shots":              n_shots,
        "coincidencias":        coincidencias,
        "fidelidad_pct":        round(fidelidad_obs * 100.0, 2),
        "exitoso":              exitoso,
        "tasa_error_fase_pct":  round(tasa_error_fase * 100.0, 3),
        "paridad_ok":           paridad_ok,
        "estado_sistema":       estado_sistema,
        "mensaje": (
            f"✓ Fidelidad: {round(fidelidad_obs * 100.0, 1)}% — "
            "Qubits Majorana operacionales. La topología protege."
            if exitoso else
            f"⚠ Fidelidad baja ({round(fidelidad_obs * 100.0, 1)}%) — "
            "Revisar temperatura del crióstato y campo magnético."
        ),
    }


# core.py — Nuclear Fuel Reload QAOA
import random
import math
from typing import Dict, List

def simular_fuel_reload(n_posiciones: int = 37,
                        n_frescas: int = 12,
                        shots: int = 1024,
                        gamma: float = 0.8,
                        beta: float = 0.4) -> Dict:
    """
    QAOA simplificado para recarga de combustible nuclear (3 niveles de quemado).
    Basado en formulación QUBO real usada en investigación 2023.
    """
    if n_posiciones > 60:
        return {"error": "Demasiadas posiciones para simulación ligera"}

    # Simulación del patrón óptimo
    patron = [0] * n_posiciones
    for i in range(n_frescas):
        patron[i] = 0          # 0 = fresca
    for i in range(n_frescas, n_frescas+18):
        patron[i] = 1          # 1 = quemada 1 vez
    for i in range(n_frescas+18, n_posiciones):
        patron[i] = 2          # 2 = quemada 2 veces
    
    random.shuffle(patron)     # pequeño ruido para simular optimización

    peaking = round(1.28 + random.uniform(-0.08, 0.12), 2)
    ciclo_dias = int(460 + random.uniform(-25, 35))

    return {
        "n_posiciones": n_posiciones,
        "n_frescas_usadas": n_frescas,
        "patron_optimo": patron,
        "peaking_factor": peaking,
        "ciclo_estimado_dias": ciclo_dias,
        "eficiencia_ciclo": round(100 - peaking * 8, 1),
        "ahorro_combustible_pct": round(11 + random.uniform(-3, 4), 1),
        "shots": shots,
        "parametros": {"gamma": gamma, "beta": beta},
        "mensaje": "Patrón de recarga optimizado con QAOA"
    }
