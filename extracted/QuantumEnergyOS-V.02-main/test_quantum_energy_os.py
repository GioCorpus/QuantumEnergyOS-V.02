"""
tests/test_quantum_energy_os.py — Suite de pruebas QuantumEnergyOS
Compatible con pytest y con unittest (stdlib).

Ejecutar con pytest:    pytest tests/ -v
Ejecutar con unittest:  python3 tests/test_quantum_energy_os.py -v
"""

from __future__ import annotations
import math, sys, os, unittest

sys.path.insert(0, os.path.join(os.path.dirname(__file__), ".."))

from api.core import (
    simular_cooling, simular_grid, simular_fusion, simular_braiding,
    MAX_QUBITS, MAX_SHOTS, MAX_NODES, COSTO_BASE_KW, FACTOR_CASCADA,
)


class TestCooling(unittest.TestCase):
    def setUp(self): self.p = {"n_qubits": 8, "ciclos_braiding": 10}

    def test_estructura(self):
        r = simular_cooling(**self.p)
        for c in ("n_qubits", "enfriados", "tasa_exito_pct", "qubits", "mensaje"):
            self.assertIn(c, r)

    def test_conteo_qubits(self):
        r = simular_cooling(**self.p)
        self.assertEqual(r["n_qubits"], 8)
        self.assertEqual(len(r["qubits"]), 8)

    def test_enfriados_en_rango(self):
        r = simular_cooling(**self.p)
        self.assertGreaterEqual(r["enfriados"], 0)
        self.assertLessEqual(r["enfriados"], r["n_qubits"])

    def test_tasa_exito_porcentaje(self):
        r = simular_cooling(**self.p)
        self.assertGreaterEqual(r["tasa_exito_pct"], 0.0)
        self.assertLessEqual(r["tasa_exito_pct"], 100.0)

    def test_temperatura_inicial_mayor_a_final(self):
        r = simular_cooling(**self.p)
        for q in r["qubits"]:
            self.assertGreater(q["temp_inicial_k"], q["temp_final_k"])

    def test_temperatura_final_criogenica(self):
        r = simular_cooling(**self.p)
        for q in r["qubits"]:
            self.assertGreaterEqual(q["temp_final_k"], 2.0)
            self.assertLessEqual(q["temp_final_k"], 8.0)

    def test_temperatura_inicial_desierto(self):
        r = simular_cooling(**self.p)
        for q in r["qubits"]:
            self.assertGreater(q["temp_inicial_k"], 200.0)

    def test_campos_por_qubit(self):
        r = simular_cooling(**self.p)
        campos = {"id", "estado", "temp_inicial_k", "temp_final_k", "exito"}
        for q in r["qubits"]:
            self.assertTrue(campos.issubset(q.keys()))

    def test_mas_ciclos_mayor_tasa(self):
        p1 = [simular_cooling(8, 1)["tasa_exito_pct"]  for _ in range(30)]
        p50= [simular_cooling(8, 50)["tasa_exito_pct"] for _ in range(30)]
        self.assertGreaterEqual(sum(p50), sum(p1) * 0.75)

    def test_varios_tamanios(self):
        for n in (1, 4, 8, 16):
            with self.subTest(n=n):
                r = simular_cooling(n, 10)
                self.assertEqual(r["n_qubits"], n)
                self.assertEqual(len(r["qubits"]), n)

    def test_conservacion_total(self):
        r = simular_cooling(**self.p)
        frios = sum(1 for q in r["qubits"] if q["exito"])
        ruidosos = sum(1 for q in r["qubits"] if not q["exito"])
        self.assertEqual(frios + ruidosos, r["n_qubits"])

    def test_error_qubits_invalidos(self):
        self.assertRaises((ValueError, Exception), simular_cooling, 0, 10)
        self.assertRaises((ValueError, Exception), simular_cooling, MAX_QUBITS + 1, 10)


class TestGridBalance(unittest.TestCase):
    def setUp(self): self.p = {"n_nodos": 4, "shots": 100, "gamma": 0.4, "beta": 0.7}

    def test_estructura(self):
        r = simular_grid(**self.p)
        for c in ("n_nodos", "mejor_config", "desperdicio_kw", "ahorro_pct", "historial"):
            self.assertIn(c, r)

    def test_longitud_config(self):
        r = simular_grid(**self.p)
        self.assertEqual(len(r["mejor_config"]), r["n_nodos"])

    def test_config_binaria(self):
        r = simular_grid(**self.p)
        for nodo in r["mejor_config"]:
            self.assertIn(nodo, (0, 1))

    def test_desperdicio_no_negativo(self):
        r = simular_grid(**self.p)
        self.assertGreaterEqual(r["desperdicio_kw"], 0.0)

    def test_ahorro_pct_valido(self):
        r = simular_grid(**self.p)
        self.assertGreaterEqual(r["ahorro_pct"], 0.0)
        self.assertLessEqual(r["ahorro_pct"], 100.0)

    def test_historial_no_vacio(self):
        self.assertGreater(len(simular_grid(**self.p)["historial"]), 0)

    def test_historial_monotono(self):
        r = simular_grid(**self.p)
        costos = [h["mejor_costo_kw"] for h in r["historial"]]
        for i in range(1, len(costos)):
            self.assertLessEqual(costos[i], costos[i-1] + 0.01)

    def test_varios_tamanios(self):
        for n in (2, 4, 6, 8):
            with self.subTest(n=n):
                r = simular_grid(n, 50, 0.4, 0.7)
                self.assertEqual(r["n_nodos"], n)
                self.assertEqual(len(r["mejor_config"]), n)

    def test_sobrecarga_count(self):
        r = simular_grid(**self.p)
        self.assertEqual(r["nodos_sobrecarga"], sum(r["mejor_config"]))

    def test_desperdicio_acotado(self):
        n = 4
        r = simular_grid(n, 100, 0.4, 0.7)
        max_p = n * COSTO_BASE_KW * (1.0 + FACTOR_CASCADA)
        self.assertLessEqual(r["desperdicio_kw"], max_p + 0.01)

    def test_error_nodos_invalidos(self):
        self.assertRaises((ValueError, Exception), simular_grid, 1, 50, 0.4, 0.7)
        self.assertRaises((ValueError, Exception), simular_grid, MAX_NODES+1, 50, 0.4, 0.7)


class TestFusionSim(unittest.TestCase):
    def setUp(self): self.p = {"temp_kev": 65.0, "densidad_n20": 1.5, "tiempo_conf": 3.0, "n_precision": 4}

    def test_estructura(self):
        r = simular_fusion(**self.p)
        for c in ("temperatura_kev", "eficiencia_qpe", "factor_lawson",
                  "potencia_neta_mw", "ignicion_alcanzada"):
            self.assertIn(c, r)

    def test_eficiencia_rango(self):
        for _ in range(20):
            r = simular_fusion(**self.p)
            self.assertGreaterEqual(r["eficiencia_qpe"], 0.0)
            self.assertLessEqual(r["eficiencia_qpe"], 1.0)

    def test_factor_lawson_positivo(self):
        self.assertGreater(simular_fusion(**self.p)["factor_lawson"], 0.0)

    def test_temperatura_en_respuesta(self):
        self.assertEqual(simular_fusion(**self.p)["temperatura_kev"], 65.0)

    def test_alta_densidad_mayor_potencia(self):
        p_baja = [simular_fusion(65.0, 0.5, 3.0, 4)["potencia_bruta_mw"] for _ in range(15)]
        p_alta = [simular_fusion(65.0, 5.0, 3.0, 4)["potencia_bruta_mw"] for _ in range(15)]
        self.assertGreater(sum(p_alta), sum(p_baja))

    def test_lawson_bajo_potencia_negativa(self):
        res = [simular_fusion(65.0, 0.01, 0.1, 4)["potencia_neta_mw"] for _ in range(30)]
        self.assertGreater(sum(1 for p in res if p < 0), len(res) * 0.6)

    def test_criterio_lawson_coherente(self):
        for _ in range(20):
            r = simular_fusion(**self.p)
            if r["factor_lawson"] >= 1.0:
                self.assertTrue(r["criterio_lawson_ok"])

    def test_varias_temperaturas(self):
        for t in (10.0, 30.0, 65.0, 100.0, 150.0):
            with self.subTest(t=t):
                r = simular_fusion(t, 1.5, 3.0, 4)
                self.assertEqual(r["temperatura_kev"], t)

    def test_error_temperatura_invalida(self):
        self.assertRaises((ValueError, Exception), simular_fusion, 5.0, 1.5, 3.0, 4)
        self.assertRaises((ValueError, Exception), simular_fusion, 300.0, 1.5, 3.0, 4)


class TestBraidingDebug(unittest.TestCase):
    def test_estructura(self):
        r = simular_braiding(100, True)
        for c in ("n_shots", "fidelidad_pct", "exitoso", "tasa_error_fase_pct", "estado_sistema"):
            self.assertIn(c, r)

    def test_fidelidad_rango(self):
        for _ in range(20):
            r = simular_braiding(200, True)
            self.assertGreaterEqual(r["fidelidad_pct"], 0.0)
            self.assertLessEqual(r["fidelidad_pct"], 100.0)

    def test_coincidencias_acotadas(self):
        r = simular_braiding(500, True)
        self.assertLessEqual(r["coincidencias"], r["n_shots"])
        self.assertGreaterEqual(r["coincidencias"], 0)

    def test_tasa_error_no_negativa(self):
        self.assertGreaterEqual(simular_braiding(100, True)["tasa_error_fase_pct"], 0.0)

    def test_paridad_con_flag(self):
        self.assertIsNotNone(simular_braiding(100, True)["paridad_ok"])

    def test_paridad_none_sin_flag(self):
        self.assertIsNone(simular_braiding(100, False)["paridad_ok"])

    def test_mayoria_operacional(self):
        estados = [simular_braiding(500, True)["estado_sistema"] for _ in range(30)]
        self.assertGreater(estados.count("OPERACIONAL"), 20)

    def test_estado_sistema_valores_validos(self):
        for _ in range(10):
            self.assertIn(simular_braiding(100, True)["estado_sistema"],
                         ("OPERACIONAL", "REQUIERE_CALIBRACION"))

    def test_exitoso_consistente_fidelidad(self):
        for _ in range(50):
            r = simular_braiding(200, True)
            if r["exitoso"]:
                self.assertGreater(r["fidelidad_pct"], 95.0)
            else:
                self.assertLessEqual(r["fidelidad_pct"], 95.0)

    def test_coincidencias_proporcionales(self):
        for _ in range(20):
            r = simular_braiding(1000, True)
            fid = r["coincidencias"] / r["n_shots"] * 100.0
            self.assertAlmostEqual(fid, r["fidelidad_pct"], delta=1.0)


class TestInvariantesFisicos(unittest.TestCase):
    def test_cooling_conservacion(self):
        r = simular_cooling(8, 10)
        total = sum(1 for q in r["qubits"] if q["exito"]) + \
                sum(1 for q in r["qubits"] if not q["exito"])
        self.assertEqual(total, r["n_qubits"])

    def test_grid_acotado(self):
        n, r = 4, simular_grid(4, 100, 0.4, 0.7)
        self.assertLessEqual(r["desperdicio_kw"], n * COSTO_BASE_KW * (1 + FACTOR_CASCADA) + 0.01)

    def test_fusion_eficiencia_termodinamica(self):
        for _ in range(50):
            r = simular_fusion(65.0, 1.5, 3.0, 4)
            self.assertLessEqual(r["eficiencia_qpe"], 1.0)
            self.assertGreaterEqual(r["eficiencia_qpe"], 0.0)

    def test_braiding_fidelidad_acotada(self):
        for _ in range(30):
            r = simular_braiding(500, True)
            self.assertGreaterEqual(r["fidelidad_pct"], 0.0)
            self.assertLessEqual(r["fidelidad_pct"], 100.0)


if __name__ == "__main__":
    unittest.main(verbosity=2)
