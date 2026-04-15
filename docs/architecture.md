[architecture.md](https://github.com/user-attachments/files/26388373/architecture.md)
# ⚙️ Arquitectura de QuantumEnergyOS

```mermaid
flowchart LR
    Qsharp[Q# Operaciones] --> Simulador[Simulador local / Qiskit]
    Simulador --> AzureQuantum[Azure Quantum]
    AzureQuantum --> Grid[Optimización de red eléctrica]
    AzureQuantum --> Fusion[Simulación de fusión]
    AzureQuantum --> Storage[Almacenamiento topológico 4D]
