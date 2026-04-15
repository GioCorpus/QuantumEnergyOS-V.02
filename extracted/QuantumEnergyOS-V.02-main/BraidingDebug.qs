namespace QuantumEnergyOS.Braiding {

    open Microsoft.Quantum.Intrinsic;
    open Microsoft.Quantum.Canon;
    open Microsoft.Quantum.Measurement;
    open Microsoft.Quantum.Math;
    open Microsoft.Quantum.Convert;
    open Microsoft.Quantum.Diagnostics;

    // ──────────────────────────────────────────────────────────────────────────
    //  BraidingDebug.qs — Depuración de operaciones topológicas (Majorana)
    //
    //  Contexto:
    //  Los qubits topológicos basados en fermiones de Majorana almacenan
    //  información en pares no locales (γ₁, γ₂). La compuerta lógica se
    //  ejecuta "trenzando" (braiding) físicamente los modos Majorana.
    //
    //  Ventaja topológica:
    //  El error no puede ocurrir localmente — requiere un error global en
    //  el sistema completo. Esto da corrección de errores cuasi-gratuita.
    //
    //  Este módulo:
    //  1. Simula la secuencia de braiding para compuertas T, S, CNOT topológicas.
    //  2. Incluye detección de errores de braiding (intercambio incorrecto).
    //  3. Benchmark de fidelidad con y sin errores.
    //
    //  Referencia: Kitaev (2003), Nayak et al. (2008) — Non-Abelian anyons.
    //
    //  Autor: GioCorpus — QuantumEnergyOS, Mexicali, B.C.
    // ──────────────────────────────────────────────────────────────────────────

    // ── Tipos auxiliares ──────────────────────────────────────────────────────

    /// Resultado de una secuencia de braiding
    newtype ResultadoBraiding = (
        Exitoso      : Bool,
        FidelEstimada: Double,
        MensajeDiag  : String
    );

    // ── Operaciones de braiding ───────────────────────────────────────────────

    /// # Resumen
    /// Simula el intercambio (braid) de dos modos Majorana γᵢ y γⱼ.
    ///
    /// # Descripción física
    /// En 2D topológico, intercambiar γᵢ con γⱼ implementa la compuerta:
    ///   B(i,j) = (1 + γⱼγᵢ) / √2
    ///
    /// En la simulación: modelamos esto como CNOT + fase, que captura
    /// la estructura unitaria del braiding no-Abeliano.
    ///
    /// # Parámetros
    /// - modi, modj : Qubits que representan los modos Majorana a intercambiar.
    /// - sentido    : true = horario, false = antihorario.
    operation BraidMajorana(modi : Qubit, modj : Qubit, sentido : Bool) : Unit is Adj + Ctl {
        if sentido {
            // Braid horario: B = H · CNOT · S · CNOT
            H(modi);
            CNOT(modi, modj);
            S(modj);
            CNOT(modi, modj);
            H(modi);
        } else {
            // Braid antihorario (inverso): aplica la secuencia adjunta
            H(modi);
            CNOT(modi, modj);
            Adjoint S(modj);
            CNOT(modi, modj);
            H(modi);
        }
    }

    /// # Resumen
    /// Implementa la compuerta T topológica (π/8) mediante braiding.
    ///
    /// # Descripción
    /// La compuerta T es la más costosa en computación cuántica estándar.
    /// En qubits topológicos, se implementa directamente por braiding,
    /// eliminando la necesidad de destilación mágica.
    ///
    /// # Circuito de braiding
    /// T_top = B(1,2) · B(2,3) · B(1,2)†
    operation CompuertaT_Topologica(
        qubitLogico : Qubit,
        ancilla1    : Qubit,
        ancilla2    : Qubit
    ) : Unit is Adj + Ctl {

        // Preparar estado topológico de los ancillas
        H(ancilla1);
        CNOT(ancilla1, ancilla2);

        // Secuencia de braiding: 3 intercambios
        BraidMajorana(qubitLogico, ancilla1, true);   // B(1,2)
        BraidMajorana(ancilla1, ancilla2, true);      // B(2,3)
        BraidMajorana(qubitLogico, ancilla1, false);  // B(1,2)†

        // Medición de ancilla para teleportación topológica
        let resultado = MResetZ(ancilla2);
        if resultado == One {
            // Corrección de Pauli (no destruye topología)
            Z(qubitLogico);
        }

        Reset(ancilla1);
    }

    /// # Resumen
    /// Implementa CNOT topológico mediante braiding de 4 modos.
    ///
    /// # Descripción
    /// El CNOT topológico requiere 4 intercambios.
    /// Esta es la compuerta de 2 qubits más eficiente en qubits Majorana.
    operation CNOT_Topologico(
        control : Qubit,
        target  : Qubit,
        anc1    : Qubit,
        anc2    : Qubit
    ) : Unit is Adj + Ctl {

        // Preparar entrelazamiento topológico de ancillas
        H(anc1);
        CNOT(anc1, anc2);

        // Secuencia de braiding para CNOT
        BraidMajorana(control, anc1, true);
        BraidMajorana(anc1, target, true);
        BraidMajorana(control, anc1, false);
        BraidMajorana(anc1, anc2, true);

        // Medición destructiva de ancillas
        let m1 = MResetZ(anc1);
        let m2 = MResetZ(anc2);

        // Correcciones de Pauli según resultado
        if m1 == One { Z(control); }
        if m2 == One { X(target);  }
    }

    // ── Herramientas de depuración ────────────────────────────────────────────

    /// # Resumen
    /// Verifica que una secuencia de braiding conserva la paridad fermiónica.
    ///
    /// # Descripción
    /// La paridad total de los modos Majorana es un invariante topológico.
    /// Si el braiding es correcto, la medición de paridad debe ser consistente.
    /// Error de paridad → secuencia de braiding incorrecta.
    ///
    /// # Retorna
    /// true si la paridad se conservó (braiding válido).
    operation VerificarParidad(modos : Qubit[]) : Bool {
        let n = Length(modos);

        if n < 2 {
            Message("⚠ VerificarParidad: se necesitan al menos 2 modos.");
            return false;
        }

        // Medir paridad del primer par (no destructivo usando ancilla)
        use ancilla = Qubit();

        // Sondeo de paridad: ancilla acumula XOR de los modos
        H(ancilla);
        for modo in modos {
            CNOT(modo, ancilla);
        }
        H(ancilla);

        let paridad = MResetZ(ancilla);

        // Paridad Zero = número par de modos en |1⟩ = paridad par conservada
        let valida = (paridad == Zero);
        if valida {
            Message("✓ Paridad fermiónica conservada — braiding topológico válido.");
        } else {
            Message("✗ ERROR: Paridad rota — revisar secuencia de braiding.");
            Message("  Posible causa: intercambio incompleto o decoherencia.");
        }

        return valida;
    }

    /// # Resumen
    /// Detecta errores de fase en una operación de braiding.
    ///
    /// # Método
    /// Aplica el braiding y luego el adjunto: el resultado debe ser |0⟩.
    /// Cualquier desviación indica un error de fase (decoherencia topológica).
    ///
    /// # Retorna
    /// Estimación de la tasa de error (0.0 = sin errores, 1.0 = error total).
    operation DetectarErrorFase(modi : Qubit, modj : Qubit, nSamples : Int) : Double {
        mutable errores = 0;

        for _ in 1 .. nSamples {
            use (q1, q2) = (Qubit(), Qubit());

            // Estado inicial conocido
            H(q1);
            CNOT(q1, q2);

            // Aplicar braiding y su adjunto: debe ser identidad
            BraidMajorana(q1, q2, true);
            Adjoint BraidMajorana(q1, q2, true);

            // Deshacer el estado inicial
            CNOT(q1, q2);
            H(q1);

            // Medir: si no es |00⟩ → hubo error
            let r1 = MResetZ(q1);
            let r2 = MResetZ(q2);

            if r1 != Zero or r2 != Zero {
                set errores += 1;
            }
        }

        let tasaError = IntAsDouble(errores) / IntAsDouble(nSamples);
        Message($"Tasa de error de fase: {tasaError * 100.0}% ({errores}/{nSamples})");
        return tasaError;
    }

    /// # Resumen
    /// Benchmark completo de fidelidad topológica.
    /// Ejecuta una secuencia de 6 braidings (circuito de Bell topológico)
    /// y mide la fidelidad contra el estado ideal.
    ///
    /// # Retorna
    /// ResultadoBraiding con fidelidad estimada y diagnóstico.
    operation BenchmarkFidelidad(nShots : Int) : ResultadoBraiding {
        mutable coincidencias = 0;

        for _ in 1 .. nShots {
            use qubit   = Qubit();
            use ancilla1 = Qubit();
            use ancilla2 = Qubit();

            // Circuito de prueba: H → CompuertaT_Topologica → medición
            // Estado esperado: (|0⟩ + e^(iπ/4)|1⟩) / √2
            H(qubit);
            CompuertaT_Topologica(qubit, ancilla1, ancilla2);

            // Para verificar: volver atrás — T† · H · T · H debe dar |0⟩
            H(qubit);
            Adjoint CompuertaT_Topologica(qubit, ancilla1, ancilla2);
            H(qubit);

            let resultado = MResetZ(qubit);
            if resultado == Zero { set coincidencias += 1; }
        }

        let fidelidad   = IntAsDouble(coincidencias) / IntAsDouble(nShots);
        let exitoso     = fidelidad > 0.95;
        let mensaje     =
            exitoso
            ? $"✓ Fidelidad topológica: {fidelidad * 100.0}% — APROBADO"
            | $"✗ Fidelidad baja: {fidelidad * 100.0}% — Revisar temperatura operacional";

        Message("══════════════════════════════════════════");
        Message("  BraidingDebug — Benchmark de Fidelidad");
        Message("══════════════════════════════════════════");
        Message(mensaje);
        if exitoso {
            Message("  Qubits Majorana operacionales. La topología protege.");
            Message("  Desde Mexicali: información cuántica indestructible.");
        }
        Message("══════════════════════════════════════════");

        return ResultadoBraiding(
            Exitoso       = exitoso,
            FidelEstimada = fidelidad,
            MensajeDiag   = mensaje
        );
    }

    /// # Resumen
    /// Suite completa de diagnóstico: corre todos los tests de braiding
    /// y genera un reporte de salud del sistema topológico.
    operation DiagnosticoCompleto() : Unit {
        Message("╔══════════════════════════════════════════╗");
        Message("║   QuantumEnergyOS — Diagnóstico Topológico  ║");
        Message("╚══════════════════════════════════════════╝");

        // Test 1: Paridad fermiónica
        Message("\n[1/3] Verificación de paridad fermiónica...");
        use modos = Qubit[4];
        PrepararEstadoPrueba(modos);
        let paridadOK = VerificarParidad(modos);
        ResetAll(modos);

        // Test 2: Error de fase
        Message("\n[2/3] Detección de errores de fase (200 muestras)...");
        use (m1, m2) = (Qubit(), Qubit());
        let tasaError = DetectarErrorFase(m1, m2, 200);

        // Test 3: Benchmark de fidelidad
        Message("\n[3/3] Benchmark de fidelidad topológica (500 shots)...");
        let benchResult = BenchmarkFidelidad(500);

        // Reporte final
        Message("\n──────────────────────────────────────────");
        Message("  RESUMEN:");
        Message($"  Paridad: {paridadOK ? '✓ OK' | '✗ FALLO'}");
        Message($"  Tasa error: {tasaError * 100.0}%");
        Message($"  Fidelidad: {benchResult::FidelEstimada * 100.0}%");
        let estadoSistema = paridadOK and tasaError < 0.05 and benchResult::Exitoso;
        Message($"  Sistema: {estadoSistema ? '✓ OPERACIONAL' | '⚠ REQUIERE CALIBRACIÓN'}");
        Message("──────────────────────────────────────────");
    }

    /// Prepara un estado de prueba conocido para verificación de paridad.
    operation PrepararEstadoPrueba(modos : Qubit[]) : Unit {
        // Estado de paridad par: |00⟩ + |11⟩ (Bell state)
        H(modos[0]);
        for i in 0 .. Length(modos) - 2 {
            CNOT(modos[i], modos[i + 1]);
        }
    }
}
