"""
cloud/ibm_quantum.py
─────────────────────
Conexión segura con IBM Quantum Runtime.

Regla de oro: el token SIEMPRE viene de os.getenv() — nunca hardcodeado,
nunca en el código fuente, nunca en logs.
"""

from __future__ import annotations

import hashlib
import logging
import os
from dataclasses import dataclass, field
from typing import Any

logger = logging.getLogger("qeos.cloud.ibm")

# ── Constantes de seguridad ───────────────────────────────────
MAX_QUBITS    = 32           # Límite duro — previene DoS exponencial
MAX_SHOTS     = 65_536       # Límite de shots por job
ALLOWED_BACKENDS = frozenset({
    "ibm_brisbane",
    "ibm_kyoto",
    "ibm_osaka",
    "ibm_sherbrooke",
    "simulator_statevector",  # simulador gratuito
    "simulator_mps",
})


@dataclass
class IBMQuantumConfig:
    """Configuración validada para IBM Quantum — sin secretos en campos."""
    backend_name: str = "simulator_statevector"
    max_shots:    int = 1024
    instance:     str = "ibm-q/open/main"
    channel:      str = "ibm_quantum"

    def __post_init__(self) -> None:
        if self.backend_name not in ALLOWED_BACKENDS:
            raise ValueError(
                f"Backend '{self.backend_name}' no permitido. "
                f"Permitidos: {sorted(ALLOWED_BACKENDS)}"
            )
        if not (1 <= self.max_shots <= MAX_SHOTS):
            raise ValueError(f"shots debe estar entre 1 y {MAX_SHOTS}")


class IBMQuantumClient:
    """
    Cliente seguro para IBM Quantum Runtime.

    El token se lee de la variable de entorno IBM_QUANTUM_TOKEN.
    Nunca se almacena en texto plano — solo se guarda un hash parcial
    para logs de auditoría.

    Uso:
        export IBM_QUANTUM_TOKEN="tu_token_aqui"
        client = IBMQuantumClient()
        result = client.run_circuit(qiskit_circuit, shots=1024)
    """

    def __init__(self, config: IBMQuantumConfig | None = None) -> None:
        self.config = config or IBMQuantumConfig()
        self._service = None
        self._token_fingerprint = self._load_and_validate_token()

    def _load_and_validate_token(self) -> str:
        """
        Carga el token desde la variable de entorno.
        Devuelve un fingerprint (primeros 8 chars del SHA-256) para logs.
        Nunca expone el token completo.
        """
        token = os.getenv("IBM_QUANTUM_TOKEN", "").strip()

        if not token:
            logger.warning(
                "IBM_QUANTUM_TOKEN no configurado — "
                "usando solo simuladores locales"
            )
            return "no-token"

        if len(token) < 20:
            raise ValueError(
                "IBM_QUANTUM_TOKEN parece inválido (muy corto). "
                "Verifica la variable de entorno."
            )

        # Fingerprint para logs — NO el token completo
        fingerprint = hashlib.sha256(token.encode()).hexdigest()[:8]
        logger.info(f"Token IBM Quantum cargado (fingerprint: {fingerprint}...)")
        return fingerprint

    def connect(self) -> bool:
        """
        Establece la conexión con IBM Quantum Runtime.
        Retorna True si exitoso, False si no hay token configurado.
        """
        token = os.getenv("IBM_QUANTUM_TOKEN", "").strip()
        if not token:
            logger.info("Sin token IBM — operando en modo simulador local")
            return False

        try:
            # Import tardío — no requerido si se usa solo simulador local
            from qiskit_ibm_runtime import QiskitRuntimeService  # type: ignore

            self._service = QiskitRuntimeService(
                channel=self.config.channel,
                token=token,             # token inyectado desde env, no hardcodeado
                instance=self.config.instance,
            )
            logger.info(
                f"Conectado a IBM Quantum — backend: {self.config.backend_name} "
                f"(token fingerprint: {self._token_fingerprint}...)"
            )
            return True

        except ImportError:
            logger.error(
                "qiskit-ibm-runtime no instalado — "
                "pip install qiskit-ibm-runtime"
            )
            return False
        except Exception as e:
            # Log del error SIN incluir el token
            safe_msg = str(e).replace(
                os.getenv("IBM_QUANTUM_TOKEN", ""), "[REDACTED]"
            )
            logger.error(f"Error al conectar con IBM Quantum: {safe_msg}")
            return False

    def validate_circuit(self, circuit: Any) -> None:
        """
        Valida un circuito cuántico antes de enviarlo a hardware real.
        Aplica MAX_QUBITS y otras restricciones de seguridad.

        Args:
            circuit: Objeto QuantumCircuit de Qiskit.

        Raises:
            ValueError: Si el circuito viola alguna restricción.
        """
        n_qubits = getattr(circuit, "num_qubits", 0)

        if n_qubits > MAX_QUBITS:
            raise ValueError(
                f"Circuito con {n_qubits} qubits excede MAX_QUBITS={MAX_QUBITS}. "
                f"Hardware IBM solo soporta hasta {MAX_QUBITS} qubits en este tier."
            )

        if n_qubits < 1:
            raise ValueError("El circuito debe tener al menos 1 qubit.")

        depth = getattr(circuit, "depth", lambda: 0)()
        if depth > 5000:
            raise ValueError(
                f"Profundidad del circuito ({depth}) demasiado alta para hardware real. "
                f"Considera reducir a < 5000 compuertas."
            )

        logger.debug(
            f"Circuito validado — qubits: {n_qubits}, "
            f"profundidad: {depth}, "
            f"hash: {self.circuit_hash(circuit)[:12]}..."
        )

    def circuit_hash(self, circuit: Any) -> str:
        """SHA-256 del circuito para auditoría e idempotencia."""
        try:
            qasm = circuit.qasm() if hasattr(circuit, "qasm") else str(circuit)
        except Exception:
            qasm = str(circuit)
        return hashlib.sha256(qasm.encode()).hexdigest()

    def run_circuit(
        self,
        circuit: Any,
        shots: int = 1024,
        fallback_to_local: bool = True,
    ) -> dict[str, Any]:
        """
        Ejecuta un circuito cuántico en IBM Quantum o en simulador local.

        Args:
            circuit:           QuantumCircuit de Qiskit.
            shots:             Número de repeticiones.
            fallback_to_local: Si True, usa Aer si no hay conexión IBM.

        Returns:
            Diccionario con counts, backend usado y hash del circuito.
        """
        shots = min(shots, MAX_SHOTS)
        self.validate_circuit(circuit)
        c_hash = self.circuit_hash(circuit)

        # Intentar hardware real
        if self._service is not None:
            try:
                from qiskit_ibm_runtime import SamplerV2 as Sampler  # type: ignore
                from qiskit import transpile

                backend = self._service.backend(self.config.backend_name)
                transpiled = transpile(circuit, backend)
                sampler = Sampler(backend)
                job = sampler.run([transpiled], shots=shots)
                result = job.result()
                counts = result[0].data.meas.get_counts()

                logger.info(
                    f"Job IBM completado — "
                    f"backend: {self.config.backend_name}, "
                    f"shots: {shots}, hash: {c_hash[:12]}..."
                )
                return {
                    "counts":   counts,
                    "backend":  self.config.backend_name,
                    "shots":    shots,
                    "hash":     c_hash,
                    "source":   "ibm_quantum",
                }
            except Exception as e:
                safe_msg = str(e).replace(
                    os.getenv("IBM_QUANTUM_TOKEN", ""), "[REDACTED]"
                )
                logger.warning(
                    f"Fallo en IBM Quantum: {safe_msg}. "
                    f"{'Usando simulador local.' if fallback_to_local else 'Sin fallback.'}"
                )
                if not fallback_to_local:
                    raise

        # Fallback: simulador local Qiskit Aer
        try:
            from qiskit_aer import AerSimulator  # type: ignore
            from qiskit import transpile

            sim = AerSimulator()
            transpiled = transpile(circuit, sim)
            result = sim.run(transpiled, shots=shots).result()
            counts = result.get_counts()

            logger.info(
                f"Simulación local completada — "
                f"shots: {shots}, hash: {c_hash[:12]}..."
            )
            return {
                "counts":  counts,
                "backend": "aer_simulator",
                "shots":   shots,
                "hash":    c_hash,
                "source":  "local_simulator",
            }
        except ImportError:
            raise RuntimeError(
                "qiskit-aer no instalado — "
                "pip install qiskit-aer"
            )

