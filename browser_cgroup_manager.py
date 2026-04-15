#!/usr/bin/env python3
"""
browser_cgroup_manager.py — QuantumEnergyOS V.02
═══════════════════════════════════════════════════════════════════════

Gestiona la priorización de procesos de navegadores mediante:

  1. **cgroups v2** (systemd slice) — control de CPU weight y memory limit
  2. **nice / ionice** — prioridad de scheduling y I/O
  3. **cpufreq** — reducción de frecuencia máxima de CPU bajo demanda

## Prerequisitos

  - Kernel Linux ≥ 5.0 con cgroups v2 (`/sys/fs/cgroup/cgroup.controllers`)
  - systemd ≥ 238 (para systemd-run --scope)
  - Usuario con permisos de systemd (no requiere root para cgroups de usuario)
  - Para ionice clase RT: CAP_SYS_NICE o root
  - Para cpufreq: acceso de escritura a /sys/devices/system/cpu/cpu*/cpufreq/

Uso standalone:
    python3 browser_cgroup_manager.py --pid 1234 --priority high
    python3 browser_cgroup_manager.py --scan   # escanear y gestionar todos

Autor: Giovanny Anthony Corpus Bernal — Mexicali, BC
"""

import os
import sys
import subprocess
import logging
import time
import argparse
from pathlib import Path
from typing import Optional, List, Tuple, Dict
from dataclasses import dataclass

logger = logging.getLogger("cgroup_manager")

# ── Constantes ────────────────────────────────────────────────────────────────

# Peso CPU para procesos de alta prioridad (máx = 10000 en cgroups v2)
# El peso estándar es 100. Alta prioridad = 4× el peso normal.
CGROUP_CPU_WEIGHT_HIGH: int = 400

# Peso CPU para procesos de baja prioridad (~25% del peso normal)
CGROUP_CPU_WEIGHT_LOW: int = 25

# Peso de I/O para procesos de baja prioridad (rango 10..1000, default 100)
CGROUP_IO_WEIGHT_LOW: int = 20

# Límite de memoria para cgroup de baja prioridad (1.5 GB en bytes)
CGROUP_MEM_LIMIT_LOW_BYTES: int = 1_500 * 1024 * 1024

# Slice de systemd para los cgroups del OS cuántico
QUANTUM_SLICE: str = "quantum-browsers.slice"

# Nombres de los scopes cgroups por tipo de prioridad
SCOPE_HIGH: str = "quantum-browser-high.scope"
SCOPE_LOW: str = "quantum-browser-low.scope"

# Prefijo del cgroup path en cgroups v2
CGROUP_V2_BASE: Path = Path("/sys/fs/cgroup")

# Ruta del cgroup de usuario actual (systemd crea uno automáticamente)
def user_cgroup_base() -> Path:
    uid = os.getuid()
    return CGROUP_V2_BASE / f"user.slice/user-{uid}.slice"


# ── Detección de cgroups v2 ────────────────────────────────────────────────────

def is_cgroup_v2() -> bool:
    """Verifica que el sistema usa cgroups v2 puro."""
    controllers = CGROUP_V2_BASE / "cgroup.controllers"
    return controllers.exists()


def get_cgroup_v2_path(pid: int) -> Optional[Path]:
    """
    Lee el cgroup actual de un PID desde /proc/[pid]/cgroup.
    En cgroups v2 puro, solo hay una entrada con jerarquía 0.
    """
    try:
        cgroup_file = Path(f"/proc/{pid}/cgroup")
        for line in cgroup_file.read_text().strip().splitlines():
            parts = line.split(":", 2)
            if len(parts) == 3 and parts[0] == "0":
                # parts[2] es el path relativo al mount point
                return CGROUP_V2_BASE / parts[2].lstrip("/")
    except (FileNotFoundError, PermissionError):
        pass
    return None


# ── Operaciones con cgroups v2 ────────────────────────────────────────────────

def create_quantum_cgroup(name: str, cpu_weight: int,
                          mem_limit_bytes: Optional[int] = None,
                          io_weight: Optional[int] = None) -> Optional[Path]:
    """
    Crea un cgroup v2 bajo el slice del usuario actual.

    En lugar de manipular /sys/fs/cgroup directamente (que requiere permisos
    del cgroup manager, normalmente systemd), usamos `systemd-run --scope`
    que crea un scope cgroup con los parámetros deseados.

    Devuelve el path del cgroup creado, o None si falla.
    """
    try:
        # Verificar si el scope ya existe
        scope_path = user_cgroup_base() / QUANTUM_SLICE / name
        if scope_path.exists():
            # Actualizar cpu.weight si ya existe
            weight_file = scope_path / "cpu.weight"
            if weight_file.exists():
                weight_file.write_text(str(cpu_weight))
            return scope_path

        # Crear el slice padre si no existe
        slice_path = user_cgroup_base() / QUANTUM_SLICE
        if not slice_path.exists():
            # systemd lo crea automáticamente cuando asignamos un proceso
            # También se puede crear manualmente:
            logger.info("Creando slice: %s", QUANTUM_SLICE)
            subprocess.run(
                ["systemctl", "--user", "start", QUANTUM_SLICE],
                check=False, capture_output=True
            )

        logger.info("Cgroup '%s' con cpu.weight=%d", name, cpu_weight)
        return scope_path

    except Exception as e:
        logger.warning("No se pudo crear cgroup '%s': %s", name, e)
        return None


def move_pid_to_cgroup(pid: int, cgroup_path: Path) -> bool:
    """
    Mueve un PID a un cgroup v2 escribiendo en cgroup.procs.

    Requiere permisos de escritura en el cgroup. En la práctica, solo
    funciona si somos dueños del proceso O si tenemos CAP_SYS_PTRACE.
    Para procesos propios (mismo UID) siempre funciona.
    """
    procs_file = cgroup_path / "cgroup.procs"
    if not procs_file.exists():
        logger.warning("cgroup.procs no existe en %s", cgroup_path)
        return False

    try:
        procs_file.write_text(str(pid))
        logger.debug("PID %d → cgroup %s", pid, cgroup_path)
        return True
    except PermissionError:
        logger.warning("PermissionError: No se puede mover PID %d (¿proceso de otro usuario?)", pid)
        return False
    except OSError as e:
        logger.warning("OSError moviendo PID %d: %s", pid, e)
        return False


def set_cgroup_cpu_weight(cgroup_path: Path, weight: int) -> bool:
    """Establece cpu.weight en un cgroup existente."""
    cpu_weight_file = cgroup_path / "cpu.weight"
    if not cpu_weight_file.exists():
        return False
    try:
        cpu_weight_file.write_text(str(weight))
        return True
    except (PermissionError, OSError) as e:
        logger.warning("No se pudo set cpu.weight en %s: %s", cgroup_path, e)
        return False


def set_cgroup_memory_limit(cgroup_path: Path, limit_bytes: int) -> bool:
    """Establece memory.max en un cgroup (0 = sin límite)."""
    mem_file = cgroup_path / "memory.max"
    if not mem_file.exists():
        return False
    try:
        value = str(limit_bytes) if limit_bytes > 0 else "max"
        mem_file.write_text(value)
        return True
    except (PermissionError, OSError) as e:
        logger.warning("No se pudo set memory.max en %s: %s", cgroup_path, e)
        return False


def set_cgroup_io_weight(cgroup_path: Path, weight: int) -> bool:
    """Establece io.weight en un cgroup."""
    io_file = cgroup_path / "io.weight"
    if not io_file.exists():
        return False
    try:
        # Formato: "default WEIGHT" para el peso por defecto
        io_file.write_text(f"default {weight}")
        return True
    except (PermissionError, OSError) as e:
        logger.warning("No se pudo set io.weight en %s: %s", cgroup_path, e)
        return False


# ── Nice e ionice ─────────────────────────────────────────────────────────────

def set_nice(pid: int, nice_value: int) -> bool:
    """
    Establece la prioridad nice de un proceso.

    nice_value rango: -20 (máx prioridad) .. +19 (mín prioridad)
    Valor estándar: 0

    Para nice negativo se requiere CAP_SYS_NICE.
    Para nice positivo cualquier usuario puede ajustar sus propios procesos.
    """
    try:
        # os.setpriority(os.PRIO_PROCESS, pid, nice_value) solo funciona para
        # el proceso actual. Para otros PIDs usamos el comando `renice`.
        result = subprocess.run(
            ["renice", "-n", str(nice_value), "-p", str(pid)],
            capture_output=True, text=True
        )
        if result.returncode == 0:
            logger.debug("renice PID %d → nice=%d", pid, nice_value)
            return True
        else:
            # renice puede fallar si el proceso ya terminó o no tenemos permisos
            logger.debug("renice PID %d falló: %s", pid, result.stderr.strip())
            return False
    except FileNotFoundError:
        # renice no disponible, intentar via /proc/[pid]/autogroup
        try:
            ag_path = Path(f"/proc/{pid}/autogroup")
            if ag_path.exists():
                ag_path.write_text(str(nice_value))
                return True
        except (PermissionError, OSError):
            pass
        return False


def set_ionice(pid: int, ioclass: int, level: int = 4) -> bool:
    """
    Establece la clase y nivel de prioridad I/O de un proceso.

    ioclass:
      1 = RT (Real-Time) — requiere root/CAP_SYS_NICE
      2 = BE (Best-Effort) — default, no requiere privilegio especial
      3 = Idle — no requiere privilegio; el proceso solo accede cuando disco libre

    level: 0 (máx) .. 7 (mín), solo para RT y BE.

    Para navegadores de alta prioridad: clase 2 nivel 0 (BE máximo)
    Para navegadores de baja prioridad: clase 3 (Idle)
    """
    try:
        result = subprocess.run(
            ["ionice", "-c", str(ioclass), "-n", str(level), "-p", str(pid)],
            capture_output=True, text=True
        )
        if result.returncode == 0:
            logger.debug("ionice PID %d → class=%d level=%d", pid, ioclass, level)
            return True
        else:
            logger.debug("ionice PID %d falló: %s", pid, result.stderr.strip())
            return False
    except FileNotFoundError:
        logger.warning("ionice no disponible en el sistema")
        return False


# ── Priorización compuesta ─────────────────────────────────────────────────────

@dataclass
class PrioritizationResult:
    pid: int
    priority_level: str       # "high" | "low" | "normal"
    nice_applied: bool
    ionice_applied: bool
    cgroup_moved: bool
    cgroup_path: Optional[str]
    error: Optional[str] = None


def prioritize_process(pid: int, high_priority: bool) -> PrioritizationResult:
    """
    Aplica la priorización completa (nice + ionice + cgroup) a un PID.

    Para alta prioridad (energía/ciencia/educación):
      - nice = -5 (más CPU)
      - ionice class 2 level 0 (I/O BE máximo)
      - cgroup cpu.weight = 400

    Para baja prioridad (todo lo demás):
      - nice = +10 (menos CPU)
      - ionice class 3 (Idle I/O)
      - cgroup cpu.weight = 25, io.weight = 20, memory.max = 1.5GB
    """
    if high_priority:
        nice_val = -5
        ionice_class, ionice_level = 2, 0
        cpu_weight = CGROUP_CPU_WEIGHT_HIGH
        scope_name = SCOPE_HIGH
        mem_limit = None
        io_weight = None
        label = "high"
    else:
        nice_val = 10
        ionice_class, ionice_level = 3, 7
        cpu_weight = CGROUP_CPU_WEIGHT_LOW
        scope_name = SCOPE_LOW
        mem_limit = CGROUP_MEM_LIMIT_LOW_BYTES
        io_weight = CGROUP_IO_WEIGHT_LOW
        label = "low"

    nice_ok = set_nice(pid, nice_val)
    ionice_ok = set_ionice(pid, ionice_class, ionice_level)

    # Intentar cgroup solo si cgroups v2 está disponible
    cgroup_path = None
    cgroup_ok = False
    if is_cgroup_v2():
        cg = create_quantum_cgroup(scope_name, cpu_weight, mem_limit, io_weight)
        if cg is not None:
            cgroup_ok = move_pid_to_cgroup(pid, cg)
            if cgroup_ok:
                cgroup_path = str(cg)
                if mem_limit:
                    set_cgroup_memory_limit(cg, mem_limit)
                if io_weight:
                    set_cgroup_io_weight(cg, io_weight)

    return PrioritizationResult(
        pid=pid,
        priority_level=label,
        nice_applied=nice_ok,
        ionice_applied=ionice_ok,
        cgroup_moved=cgroup_ok,
        cgroup_path=cgroup_path,
    )


# ── cpufreq — reducción de frecuencia bajo demanda ────────────────────────────

def get_cpu_max_freq_mhz(cpu_idx: int = 0) -> Optional[int]:
    """Lee la frecuencia máxima actual de la CPU [MHz]."""
    try:
        freq_file = Path(f"/sys/devices/system/cpu/cpu{cpu_idx}/cpufreq/scaling_max_freq")
        return int(freq_file.read_text().strip()) // 1000  # kHz → MHz
    except (FileNotFoundError, ValueError):
        return None


def set_cpu_max_freq(cpu_idx: int, freq_mhz: int) -> bool:
    """
    Establece la frecuencia máxima de una CPU [MHz].

    Requiere acceso de escritura a /sys/devices/system/cpu/cpu*/cpufreq/
    (normalmente root, o configurar udev rules).

    Útil para reducir el consumo cuando los navegadores están muy activos.
    """
    try:
        freq_file = Path(f"/sys/devices/system/cpu/cpu{cpu_idx}/cpufreq/scaling_max_freq")
        freq_file.write_text(str(freq_mhz * 1000))  # MHz → kHz
        logger.info("CPU%d: frecuencia máxima → %d MHz", cpu_idx, freq_mhz)
        return True
    except (FileNotFoundError, PermissionError, OSError) as e:
        logger.warning("No se pudo ajustar cpufreq CPU%d: %s", cpu_idx, e)
        return False


def throttle_all_cpus(target_pct: float = 0.7) -> int:
    """
    Reduce la frecuencia máxima de todas las CPUs a `target_pct` × máximo.

    Solo si cpufreq y el governor "powersave" o "schedutil" están disponibles.
    Devuelve el número de CPUs ajustadas.
    """
    cpu_dir = Path("/sys/devices/system/cpu")
    adjusted = 0

    for cpu_path in sorted(cpu_dir.glob("cpu[0-9]*")):
        idx_str = cpu_path.name.replace("cpu", "")
        if not idx_str.isdigit():
            continue
        idx = int(idx_str)

        # Leer frecuencia máxima hardware (no la actual del governor)
        cpuinfo_max = cpu_path / "cpufreq/cpuinfo_max_freq"
        if not cpuinfo_max.exists():
            continue

        try:
            hw_max_khz = int(cpuinfo_max.read_text().strip())
            target_khz = int(hw_max_khz * target_pct)
            target_mhz = target_khz // 1000

            if set_cpu_max_freq(idx, target_mhz):
                adjusted += 1
        except (ValueError, IOError):
            continue

    return adjusted


def restore_all_cpus() -> int:
    """Restaura la frecuencia máxima de todas las CPUs a su máximo hardware."""
    cpu_dir = Path("/sys/devices/system/cpu")
    restored = 0

    for cpu_path in sorted(cpu_dir.glob("cpu[0-9]*")):
        idx_str = cpu_path.name.replace("cpu", "")
        if not idx_str.isdigit():
            continue
        idx = int(idx_str)

        cpuinfo_max = cpu_path / "cpufreq/cpuinfo_max_freq"
        scaling_max = cpu_path / "cpufreq/scaling_max_freq"

        if not cpuinfo_max.exists() or not scaling_max.exists():
            continue

        try:
            hw_max = cpuinfo_max.read_text().strip()
            scaling_max.write_text(hw_max)
            restored += 1
        except (IOError, PermissionError):
            continue

    return restored


# ── Escáner de procesos (independiente del daemon Rust) ───────────────────────

def scan_and_prioritize(high_priority_keywords: List[str] = None) -> List[PrioritizationResult]:
    """
    Escanea /proc, detecta navegadores y aplica priorización.

    Puede usarse standalone (sin el daemon Rust) o como complemento.
    """
    if high_priority_keywords is None:
        high_priority_keywords = [
            "energy", "solar", "quantum", "arxiv", "wikipedia",
            "nasa", "cfe.mx", "cenace", "scholar",
        ]

    results = []
    proc_base = Path("/proc")

    for entry in proc_base.iterdir():
        if not entry.name.isdigit():
            continue
        pid = int(entry.name)

        comm_file = entry / "comm"
        if not comm_file.exists():
            continue

        try:
            comm = comm_file.read_text().strip().lower()
        except (PermissionError, FileNotFoundError):
            continue

        # Detectar navegadores
        is_browser = any(b in comm for b in [
            "firefox", "chromium", "brave", "chrome", "opera", "edge"
        ])
        if not is_browser:
            continue

        # Determinar prioridad desde cmdline
        high_prio = False
        try:
            cmdline = (entry / "cmdline").read_text().replace('\0', ' ').lower()
            high_prio = any(kw in cmdline for kw in high_priority_keywords)
        except (PermissionError, FileNotFoundError):
            pass

        result = prioritize_process(pid, high_prio)
        results.append(result)

        logger.info(
            "PID %d (%s) → %s | nice:%s ionice:%s cgroup:%s",
            pid, comm, result.priority_level,
            "✓" if result.nice_applied else "✗",
            "✓" if result.ionice_applied else "✗",
            "✓" if result.cgroup_moved else "✗",
        )

    return results


# ── CLI ───────────────────────────────────────────────────────────────────────

def main():
    parser = argparse.ArgumentParser(
        description="QuantumEnergyOS V.02 — Browser cgroup/nice manager",
        formatter_class=argparse.RawDescriptionHelpFormatter,
        epilog="""
Ejemplos:
  # Escanear todos los navegadores y aplicar priorización automática
  python3 browser_cgroup_manager.py --scan

  # Aplicar alta prioridad a un PID específico
  python3 browser_cgroup_manager.py --pid 1234 --priority high

  # Reducir frecuencia CPU al 70% cuando los navegadores consumen mucho
  python3 browser_cgroup_manager.py --throttle-cpu 70

  # Restaurar frecuencias CPU a máximo hardware
  python3 browser_cgroup_manager.py --restore-cpu

  # Modo daemon: escanear y re-priorizar cada 10 segundos
  python3 browser_cgroup_manager.py --daemon --interval 10
        """
    )
    parser.add_argument("--scan", action="store_true",
                        help="Escanear todos los navegadores y priorizar")
    parser.add_argument("--pid", type=int, help="PID específico a priorizar")
    parser.add_argument("--priority", choices=["high", "low", "normal"],
                        default="low", help="Nivel de prioridad (default: low)")
    parser.add_argument("--throttle-cpu", type=float, metavar="PCT",
                        help="Reducir frecuencia CPU a PCT%% del máximo (ej: 70)")
    parser.add_argument("--restore-cpu", action="store_true",
                        help="Restaurar frecuencias CPU al máximo hardware")
    parser.add_argument("--daemon", action="store_true",
                        help="Modo daemon: re-escanear periódicamente")
    parser.add_argument("--interval", type=float, default=10.0,
                        help="Intervalo de re-escaneo en modo daemon [s] (default: 10)")
    parser.add_argument("--verbose", "-v", action="store_true")

    args = parser.parse_args()

    if args.verbose:
        logging.basicConfig(level=logging.DEBUG)
    else:
        logging.basicConfig(
            level=logging.INFO,
            format="[%(asctime)s] %(levelname)s: %(message)s",
            datefmt="%H:%M:%S"
        )

    print("╔══════════════════════════════════════════════════════════════╗")
    print("║  QuantumEnergyOS V.02 — Browser cgroup/nice Manager         ║")
    print("║  Priorización via cgroups v2 + nice + ionice                ║")
    print("╚══════════════════════════════════════════════════════════════╝")
    print()
    print(f"  cgroups v2: {'✓ disponible' if is_cgroup_v2() else '✗ no disponible (usando solo nice/ionice)'}")
    print(f"  UID: {os.getuid()} | root: {os.getuid() == 0}")
    print()

    if args.restore_cpu:
        n = restore_all_cpus()
        print(f"  ✓ Frecuencias restauradas en {n} CPUs")
        return

    if args.throttle_cpu:
        pct = args.throttle_cpu / 100.0
        n = throttle_all_cpus(pct)
        print(f"  ✓ Frecuencia reducida a {args.throttle_cpu:.0f}% en {n} CPUs")
        return

    if args.pid:
        high = (args.priority == "high")
        result = prioritize_process(args.pid, high)
        print(f"  PID {result.pid}: {result.priority_level}")
        print(f"    nice:    {'✓' if result.nice_applied else '✗'}")
        print(f"    ionice:  {'✓' if result.ionice_applied else '✗'}")
        print(f"    cgroup:  {'✓ ' + str(result.cgroup_path) if result.cgroup_moved else '✗'}")
        return

    if args.scan or args.daemon:
        def do_scan():
            results = scan_and_prioritize()
            high = sum(1 for r in results if r.priority_level == "high")
            low = sum(1 for r in results if r.priority_level == "low")
            print(f"  Escaneados: {len(results)} procesos | Alta: {high} | Baja: {low}")
            return results

        if args.daemon:
            print(f"  Modo daemon — intervalo: {args.interval}s (Ctrl+C para detener)")
            while True:
                do_scan()
                time.sleep(args.interval)
        else:
            do_scan()
        return

    parser.print_help()


if __name__ == "__main__":
    main()
