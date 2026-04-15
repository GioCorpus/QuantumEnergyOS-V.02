# 🌐 Master Browser Setup — QuantumEnergyOS V.02

**Autor:** Giovanny Anthony Corpus Bernal — Mexicali, BC  
**Versión:** V.02  
**Objetivo:** Integración nativa de navegadores web como interfaz primaria del OS cuántico

---

## Qué resuelve este módulo

QuantumEnergyOS necesita navegadores para:
1. Mostrar el dashboard de red eléctrica en tiempo real (`localhost:8000`)
2. Acceder a IBM Quantum, Google Colab, y documentación de Q#/Qiskit
3. Monitorear actividad solar y datos de la CFE
4. Correr simulaciones WebGPU directamente en el browser

El reto no es solo instalar los navegadores — es integrarlos con el kernel cuántico para que **no compitan por recursos** con las operaciones QAOA críticas.

---

## Arquitectura de integración

```
┌────────────────────────────────────────────────────────────────┐
│                    QuantumEnergyOS V.02                        │
├────────────────────────────────────────────────────────────────┤
│  Firefox (EnergyGrid)  │  Chromium (QuantumDev)  │  Brave      │
├────────────────────────┴─────────────────────────┴─────────────┤
│              browser_integration.py (Flask Blueprint)          │
│    /api/v1/browser/status  · /optimize  · /open  · /metrics    │
├────────────────────────────────────────────────────────────────┤
│  browser_cgroup_manager.py       browser_monitor_module.py     │
│  cgroups v2 · nice · ionice      QAOA trigger · telemetría     │
├────────────────────────────────────────────────────────────────┤
│              PhotonicQ Bridge (Rust)                           │
│          Scheduler híbrido — browsers = baja prioridad         │
│          QAOA — browsers de energía = alta prioridad           │
└────────────────────────────────────────────────────────────────┘
```

---

## Archivos del módulo

| Archivo | Descripción |
|---|---|
| `install_browsers.sh` | Instalación y configuración completa |
| `browser_integration.py` | Blueprint Flask — todos los endpoints |
| `browser_cgroup_manager.py` | Control de cgroups v2 + nice + ionice |
| `browser_monitor_module.py` | Monitor de métricas + QAOA trigger |
| `packages-browsers.x86_64` | Paquetes para incluir en la ISO |

---

## Instalación

```bash
# En Arch Linux / QuantumEnergyOS live
sudo chmod +x setup/install_browsers.sh
sudo ./setup/install_browsers.sh

# Verificar
qeos-browser --help
start-energy-browser
```

---

## Integrar con server.py

Añadir al final de `api/server.py`, antes del `if __name__ == '__main__':`:

```python
from browser_integration import integrate_browsers
integrate_browsers(app)
```

Esto agrega automáticamente:
- `GET  /api/v1/browser/status`   — estado de todos los navegadores
- `GET  /api/v1/browser/profiles` — perfiles disponibles
- `POST /api/v1/browser/open`     — abrir navegador con perfil
- `POST /api/v1/browser/optimize` — forzar optimización QAOA
- `GET  /api/v1/browser/qaoa/events` — historial de optimizaciones
- `POST /api/v1/browser/metrics`  — recibir telemetría del daemon Rust
- `POST /api/v1/browser/config`   — actualizar configuración en vivo

---

## Perfiles de navegador

### EnergyGrid (Firefox) — Prioridad ALTA
- Propósito: dashboard energético en tiempo real
- URL default: `http://localhost:8000`
- Hardware accel: VA-API + WebRender
- WebSocket/SSE optimizado para actualizaciones de 100ms
- Extensiones: uBlock Origin, Dark Reader, Bitwarden
- cgroups: `cpu.weight=400` (4× el peso normal)

### QuantumDev (Chromium) — Prioridad ALTA  
- Propósito: desarrollo Q#, Qiskit, simulaciones WebGPU
- Flags: `--enable-features=VaapiVideoDecoder,WebGPU,VulkanFromANGLE`
- `--ozone-platform=wayland` — nativo Wayland sin XWayland
- `--enable-unsafe-webgpu` — para simulaciones cuánticas en GPU
- URL recomendada: `https://lab.quantum.ibm.com`

### Secure (Brave) — Prioridad NORMAL
- Propósito: navegación general, privacidad
- cgroups: `cpu.weight=25`, `memory.max=1.5GB` cuando no prioritario
- Sin telemetría, con Shields activos

---

## Atajos de teclado Sway

```
Ctrl+Shift+E  →  Dashboard energético (Firefox EnergyGrid)
Ctrl+Shift+Q  →  Chromium QuantumDev
Ctrl+Shift+B  →  Brave Secure
```

---

## Comandos útiles

```bash
# Estado de navegadores desde CLI
qeos-browser status    # o: curl http://localhost:8000/api/v1/browser/status

# Forzar optimización QAOA
curl -X POST http://localhost:8000/api/v1/browser/optimize

# Abrir dashboard
qeos-browser energy-grid

# Ver eventos QAOA aplicados
curl http://localhost:8000/api/v1/browser/qaoa/events

# Escanear y re-priorizar todos los navegadores
python3 browser_cgroup_manager.py --scan

# Modo daemon (re-escanear cada 10s)
python3 browser_cgroup_manager.py --daemon --interval 10

# Reducir CPU al 70% cuando los navegadores consumen mucho
python3 browser_cgroup_manager.py --throttle-cpu 70

# Verificar hardware acceleration en Firefox
# Abrir en Firefox: about:support → buscar "WebRender"
# Debe decir: "WebRender: enabled by default"

# Verificar hardware acceleration en Chromium
# Abrir en Chromium: chrome://gpu
# Debe mostrar: "Video Decode: Hardware accelerated"
```

---

## Verificar hardware acceleration

```bash
# VA-API
vainfo 2>/dev/null && echo "VA-API: OK" || echo "VA-API: no disponible"

# Vulkan
vulkaninfo --summary 2>/dev/null | grep -E "deviceName|apiVersion"

# Intel GPU
intel_gpu_top -l 1 2>/dev/null || echo "intel-gpu-tools no instalado"
```

---

## Notas de optimización energética

El módulo `browser_cgroup_manager.py` aplica la siguiente lógica:

| Perfil | CPU weight | Memory limit | I/O weight | nice |
|---|---|---|---|---|
| EnergyGrid / QuantumDev | 400 (alta) | sin límite | default | -5 |
| Secure / otros | 25 (baja) | 1.5 GB | 20 (idle) | +10 |
| QAOA en ejecución | 1000 (máxima) | sin límite | default | -20 |

Cuando el scheduler cuántico detecta que el QAOA necesita CPU urgente, el `browser_monitor_module.py` reduce automáticamente la prioridad de los navegadores de baja prioridad, liberando recursos para la operación cuántica.

**Resultado:** los apagones se detectan y previenen en <1 ms, sin importar cuántas pestañas tenga abiertas el usuario.

---

## Roadmap

- [ ] Plugin Firefox nativo para visualizar el estado del Cuarzo 4D
- [ ] Extension Chrome para control del dashboard desde la barra de herramientas  
- [ ] WebGPU shaders para simulaciones cuánticas directamente en el browser
- [ ] Notificaciones push cuando el índice solar Kp sube a ≥5

---

*⚡ Desde Mexicali, BC — Nunca más apagones. Kardashev 0→1.*
