namespace QuantumEnergyOS.Cooling {

    open Microsoft.Quantum.Intrinsic;
    open Microsoft.Quantum.Canon;
    open Microsoft.Quantum.Measurement;
    open Microsoft.Quantum.Math;

    // ──────────────────────────────────────────────────────────────────────────
    //  Cooling.qs — Enfriamiento criogénico simulado para qubits Majorana
    //
    //  Contexto real: los qubits topológicos operan a ~4 mK (milikelvin).
    //  Esta simulación modela el proceso de enfriamiento como una secuencia
    //  de pulsos de reinicio térmico + braiding protector contra decoherencia.
    //
    //  Autor: GioCorpus — QuantumEnergyOS, Mexicali, B.C.
    // ──────────────────────────────────────────────────────────────────────────

    /// # Resumen
    /// Simula el enfriamiento criogénico de un qubit Majorana.
    /// Modela la reducción de ruido térmico de ~300 K (desierto) a ~4 K (crióstato).
    ///
    /// # Parámetros
    /// - qubit : El qubit a enfriar (representando un modo Majorana).
    ///
    /// # Descripción del protocolo
    /// 1. Reset térmico: lleva el qubit al estado base |0⟩.
    /// 2. Braiding protector: serie de operaciones H+CNOT que construyen
    ///    el entrelazamiento topológico que protege contra decoherencia.
    /// 3. Medición de temperatura lógica: mide el estado final.
    operation EnfriarMajorana(qubit : Qubit, auxiliar : Qubit) : Result {

        // Paso 1 — Reset térmico: fuerza el estado base (equivalente a enfriar)
        Reset(qubit);
        Reset(auxiliar);

        // Paso 2 — Braiding protector (10 ciclos)
        // Cada ciclo: H crea superposición, CNOT entrelaza con auxiliar.
        // Topológicamente: simula el trenzado de modos Majorana que suprime
        // la decoherencia local sin destruir la información cuántica.
        for _ in 1 .. 10 {
            H(qubit);
            CNOT(qubit, auxiliar);   // FIX: dos qubits distintos, no self-CNOT
            H(auxiliar);
            CNOT(auxiliar, qubit);
        }

        // Paso 3 — Medición de temperatura lógica
        // |0⟩ → "frío" (4 K operacional)
        // |1⟩ → "caliente" (ruido térmico residual, aplicar reset adicional)
        let temperatura = M(qubit);  // FIX: M(qubit) en lugar de Measure( , )

        if temperatura == Zero {
            Message("✓ Qubit enfriado: 4 K operacional. ¡El desierto de Mexicali no gana hoy!");
        } else {
            Reset(qubit);
            Reset(auxiliar);
            Message("⚠ Ruido térmico detectado — reset aplicado. Reintentar braiding.");
        }

        return temperatura;
    }

    /// # Resumen
    /// Ejecuta el protocolo de enfriamiento en una red de 8 qubits Majorana.
    /// Modela el crióstato de un procesador cuántico topológico pequeño.
    ///
    /// # Retorna
    /// Número de qubits que alcanzaron el estado operacional (4 K lógico).
    operation EnfriarRed() : Int {
        use qubits    = Qubit[8];
        use auxiliares = Qubit[8];

        mutable enfriados = 0;

        for i in 0 .. 7 {
            let resultado = EnfriarMajorana(qubits[i], auxiliares[i]);
            if resultado == Zero {
                set enfriados += 1;
            }
        }

        let porcentaje = (enfriados * 100) / 8;
        Message($"Red de 8 qubits enfriada: {enfriados}/8 operacionales ({porcentaje}%)");
        Message("Sistema listo para braiding. Samsung agradece, la CFE tiembla.");

        // Limpiar estado antes de liberar
        ResetAll(qubits);
        ResetAll(auxiliares);

        return enfriados;
    }

    /// # Resumen
    /// Benchmark de fidelidad del protocolo de enfriamiento.
    /// Ejecuta N repeticiones y reporta la tasa de éxito.
    operation BenchmarkEnfriamiento(repeticiones : Int) : Double {
        mutable exitos = 0;

        for _ in 1 .. repeticiones {
            use q  = Qubit();
            use qa = Qubit();
            let r = EnfriarMajorana(q, qa);
            if r == Zero { set exitos += 1; }
        }

        let fidelidad = IntAsDouble(exitos) / IntAsDouble(repeticiones);
        Message($"Fidelidad de enfriamiento: {fidelidad * 100.0}% ({exitos}/{repeticiones})");
        return fidelidad;
    }
}
