# 🔬 Mapa de Hardware Compatible — QuantumEnergyOS Kernel Fotónico

## Chips de silicio fotónico soportados

### QuiX Quantum (Países Bajos)
- **Material**: Si₃N₄ (nitruro de silicio)
- **Modos**: 12–20 modos (chips actuales 2025-26)
- **Pérdida waveguide**: ~0.1 dB/cm — el más bajo del mercado
- **Moduladores**: termo-ópticos, ~1 μs switching
- **Detección**: SNSPD integrado (eficiencia >95%)
- **Temperatura**: ambiente (300 K) — sin criogenia
- **Driver kernel**: `ChipType::QuiX12Mode`
- **Latencia Bridge**: ~1–2 ms (dominado por moduladores lentos)
- **Referencia**: *QuiX Quantum 12-mode chip, Nature Photonics 2022*

### Xanadu — X-Series / Borealis (Canadá)
- **Material**: GaAs + LiNbO₃ (niobato de litio)
- **Modos**: 216 modos (Borealis, 2022)
- **Moduladores**: electro-ópticos LiNbO₃, **40 GHz ancho de banda**
- **Recurso cuántico**: luz comprimida (squeezed), hasta 15 dB
- **Corrección de errores**: GKP implementada en hardware
- **Framework**: PennyLane compatible
- **Driver kernel**: `ChipType::XanaduBorealis`
- **Latencia Bridge**: **~0.1 ms** (moduladores EO ultra-rápidos)
- **Referencia**: *Xanadu Borealis, Nature 2022; X-Series architecture 2024*

### Photonic Inc. (Canadá/EE.UU.)
- **Material**: GaAs — compuerta cuántica vía interacción fotón-fotón
- **Arquitectmo**: Fusion-Based Quantum Computing (FBQC)
- **Recurso**: pares de Bell on-demand (SPDC en waveguide GaAs)
- **Ventaja**: escalabilidad vía fusión lineal — no necesita qubits de alta conectividad
- **Driver kernel**: `ChipType::PhotonicIncFBQC`
- **Referencia**: *Bombin et al., PRX Quantum 2021; Photonic Inc. roadmap 2024*

### Chips CMOS-PIC Híbridos
- **Material**: Si/SiO₂ + detectores Ge
- **Ventaja**: integración directa con CMOS standard — fabricación en foundry convencional
- **Pérdida**: ~2 dB/cm (mayor que Si₃N₄, pero más accesible)
- **Moduladores**: SiGe electro-óptico, ~10 GHz
- **Driver kernel**: `ChipType::CMOSPICHybrid`

---

## Componentes físicos del sistema

### Waveguides
| Material | Pérdida (dB/cm) | n_eff | BW (nm) | Ventaja |
|---|---|---|---|---|
| Si₃N₄ | 0.1 | 1.9 | 400–2400 | Baja pérdida, sin TPA |
| SiO₂ | 0.001 | 1.44 | 200–3000 | Ultra-baja pérdida |
| LiNbO₃ | 0.3 | 2.2 | 400–5000 | EO modulation |
| GaAs | 0.5 | 3.4 | 900–17000 | No-linealidad alta |
| Si | 2.0 | 3.48 | 1100–7000 | Integración CMOS |

### MZI (Mach-Zehnder Interferómetros)
- **Splitting ratio**: 50:50 (beam splitter ideal)
- **Extinction ratio**: >30 dB (chip comercial)
- **Throughput**: >95% por MZI (incluye acoplamiento)
- **Control**: DAC 16-bit, resolución ~0.1 mrad
- **Algoritmo de programación**: Descomposición de Clements (O(N²) MZIs para matriz N×N)

### Detección Homodyne
- **Componentes**: oscilador local (LO) + beam splitter 50:50 + detector balanceado InGaAs
- **Clearance electrónico**: >15 dB sobre shot noise (requisito mínimo para QC)
- **Ancho de banda**: DC–300 MHz (homodyne RF)
- **Eficiencia**: 97–99% (detectores InGaAs comerciales 2025)

### SNSPD (Superconducting Nanowire Single Photon Detector)
- **Eficiencia**: >99% (estado del arte 2026, Photon Spot / ID Quantique)
- **Jitter temporal**: <3 ps (resolución temporal ultra-fina)
- **Dark counts**: <1 Hz (criogénico a 0.8 K)
- **Temperatura de operación**: 0.8–4 K — requiere criogenia
- **Max count rate**: >1 GHz (limitado por dead time)
- **Nota para kernel**: las tareas que usan SNSPD tienen deadline más estricto
  (el criogénico tiene presupuesto de frio limitado)

---

## Topología del sistema fotónico

```
Fuente SPDC (GaAs/PPLN)
    ↓ pares entrelazados 1550 nm
Splitter de distribución (1×N)
    ↓ N modos independientes
Mesh MZI (N×N, algoritmo Clements)
    ↓ transformación unitaria U
Moduladores de fase (LiNbO₃/SiN)
    ↓ control óptico fino
  ┌─── Homodyne ──→ ADC 20 GHz ──→ PhotonicQ Bridge
  │    (X̂, P̂)                         ↑
  └─── SNSPD ────→ TDC 3 ps  ──→ GKP Corrector
```

---

## Compatibilidad con Nature Photonics 2025-26

Los diseños de este kernel son compatibles con:

- **Taballione et al. (QuiX)**: *20-mode universal linear optics*, nphoton 2023
- **Madsen et al. (Xanadu)**: *Quantum computational advantage with a programmable photonic processor*, Nature 2022
- **Arrazola et al. (Xanadu)**: *Quantum circuits with many photons*, Nature 2021
- **Bombin et al. (Photonic Inc.)**: *Interleaving: modular architectures for fault-tolerant photonic quantum computing*, PRX Quantum 2021
- **Chips CMOS-PIC**: AIM Photonics (EE.UU.) — foundry access para prototipos

---

## Roadmap de hardware (2025–2030)

| Año | Hito | Modos | Squeezing | Error lógico |
|---|---|---|---|---|
| 2025 | Estado actual (QuiX/Xanadu) | 12–216 | 12 dB | 10⁻³ |
| 2026 | GKP fault-tolerant demo | 50+ | 15 dB | 10⁻⁵ |
| 2027 | Braiding de Majorana fotónico | 100+ | 18 dB | 10⁻⁶ |
| 2028 | VQE molecular práctico (FeMoco) | 500+ | 20 dB | 10⁻⁸ |
| 2030 | Kardashev-ready photonic QC | 1000+ | 25 dB | 10⁻¹² |

---

*Desde Mexicali, BC — apuntando a cada uno de esos hitos. ⚡*
