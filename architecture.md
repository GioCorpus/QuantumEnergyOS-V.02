# 🏗️ Arquitectura Clásica de QuantumEnergyOS

```mermaid
graph TD
    Frontend[Frontend Web (HTML/JS)] --> Backend[Backend Simulado (API JS/Python)]
    Backend --> QuantumLayer[Quantum Layer (Q# + Qiskit)]
    QuantumLayer --> Cloud[Azure Quantum]
    Cloud --> Energia[Aplicaciones Energéticas]
    Energia --> Grid[Optimización de Red]
    Energia --> Fusion[Simulación de Fusión]
    Energia --> Storage[Almacenamiento Topológico 4D]


# 🔗 Arquitectura Híbrida End-to-End

```mermaid
flowchart TB
    Frontend[Frontend Web (HTML/JS)] --> Backend[Backend Simulado (API JS/Python)]
    Backend --> Qsharp[Q# Operaciones]
    Qsharp --> Simulador[Simulación Local (Python + Qiskit)]
    Simulador --> AzureQuantum[Azure Quantum]
    AzureQuantum --> Grid[Optimización de Red]
    AzureQuantum --> Fusion[Simulación de Fusión]
    AzureQuantum --> Storage[Almacenamiento Topológico 4D]
    Grid --> ImpactoRegional[Impacto: Baja California / Sonora / Chihuahua]
    Fusion --> ImpactoGlobal[Impacto: Energía limpia global]
- name: Run pip-audit
  run: pip-audit --requirement requirements-pinned.txt --skip-deps docs/
