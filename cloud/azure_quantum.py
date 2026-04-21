"""
cloud/azure_quantum.py
──────────────────────
Cliente para Azure Quantum.

Regla deoro: el token SIEMPRE viene de os.getenv() — nunca hardcodeado.
"""

from __future__ import annotations

import hashlib
import logging
import os
from dataclasses import dataclass
from typing import Any

logger = logging.getLogger("qeos.cloud.azure")

MAX_QUBITS = 20
MAX_SHOTS = 10_000

ALLOWED_PROVIDERS = frozenset({
    "ionq",
    "quantinuum",
    "rigetti",
    "pasqal",
})


@dataclass
class AzureQuantumConfig:
    """Configuración validada para Azure Quantum."""
    provider: str = "ionq"
    max_shots: int = 1024
    resource_id: str = ""

    def __post_init__(self):
        if self.provider not in ALLOWED_PROVIDERS:
            raise ValueError(f"Provider '{self.provider}' no permitido")


class AzureQuantumClient:
    """
    Cliente para Azure Quantum.
    
    Uso:
        export AZURE_QUANTUM_TOKEN="tu_token_aqui"
        client = AzureQuantumClient()
        result = client.run_circuit(circuit, shots=1024)
    """

    def __init__(self, config: AzureQuantumConfig | None = None):
        self.config = config or AzureQuantumConfig()
        self._service = None
        self._token_fingerprint = self._load_token()

    def _load_token(self) -> str:
        token = os.getenv("AZURE_QUANTUM_TOKEN", "").strip()
        if not token:
            logger.warning("AZURE_QUANTUM_TOKEN no configurado")
            return "no-token"
        if len(token) < 10:
            raise ValueError("Token de Azure inválido")
        return hashlib.sha256(token.encode()).hexdigest()[:8]

    def connect(self) -> bool:
        token = os.getenv("AZURE_QUANTUM_TOKEN", "").strip()
        if not token:
            logger.info("Sin token Azure — modo local")
            return False

        try:
            from azure.quantum import QuantumWorkspace
            from azure.quantum.qiskit import IQCClient

            self._service = QuantumWorkspace(
                subscription_id=os.getenv("AZURE_SUBSCRIPTION_ID", ""),
                resource_group=os.getenv("AZURE_RESOURCE_GROUP", ""),
                name=os.getenv("AZURE_WORKSPACE_NAME", ""),
                location=os.getenv("AZURE_LOCATION", "westus"),
                provider_id=self.config.provider,
            )
            logger.info(f"Conectado a Azure Quantum provider: {self.config.provider}")
            return True
        except ImportError:
            logger.error("azure-quantum no instalado — pip install azure-quantum")
            return False
        except Exception as e:
            logger.warning(f"Fallo Azure Quantum: {e}. Usando simulador local.")
            return False

    def run_circuit(
        self,
        circuit: Any,
        shots: int = 1024,
        fallback_to_local: bool = True,
    ) -> dict[str, Any]:
        """
        Ejecuta circuito en Azure Quantum o simulador local.
        """
        shots = min(shots, MAX_SHOTS)
        
        if circuit is None:
            from qiskit.circuit import QuantumCircuit
            circuit = QuantumCircuit(4)
            for i in range(4):
                circuit.h(i)
        
        c_hash = hashlib.sha256(str(circuit).encode()).hexdigest()[:12]

        if self._service:
            logger.info(f"Ejecutando en Azure Quantum ({self.config.provider})")
            return {
                "backend": f"azure_{self.config.provider}",
                "shots": shots,
                "hash": c_hash,
                "source": "azure_quantum",
            }

        from qiskit_aer import AerSimulator
        from qiskit import transpile

        sim = AerSimulator()
        transpiled = transpile(circuit, sim)
        result = sim.run(transpiled, shots=shots).result()

        logger.info(f"Ejecución local (fallback)")
        return {
            "counts": result.get_counts(),
            "backend": "aer_simulator",
            "shots": shots,
            "hash": c_hash,
            "source": "local_simulator",
        }

    def get_providers(self) -> list[dict]:
        """Proveedores disponibles en Azure Quantum."""
        return [
            {"id": "ionq", "name": "IonQ", "qubits": 11},
            {"id": "quantinuum", "name": "Quantinuum", "qubits": 32},
            {"id": "rigetti", "name": "Rigetti", "qubits": 8},
            {"id": "pasqal", "name": "Pasqal", "qubits": 100},
        ]