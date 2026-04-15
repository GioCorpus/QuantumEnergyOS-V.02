#!/usr/bin/env python3
"""
server_patch.py — QuantumEnergyOS V.02
Parchea api/server.py para agregar la integración de browsers.

USO:
    python3 server_patch.py              # Parchear api/server.py automáticamente
    python3 server_patch.py --check      # Solo verificar si ya está integrado
    python3 server_patch.py --revert     # Revertir el parche

O ejecutar el server ya parchado directamente:
    python3 server_patch.py --run        # Parchear y correr
"""

import argparse
import os
import shutil
import subprocess
import sys
from pathlib import Path

SERVER_PATH = Path("api/server.py")
BACKUP_PATH = Path("api/server.py.backup")

INTEGRATION_IMPORT = "from browser_integration import integrate_browsers"
INTEGRATION_CALL   = "    integrate_browsers(app)"
INTEGRATION_COMMENT = "    # ── Browser integration (navegadores + QAOA) ──────────────"

MARKER = "# [QEOS-BROWSER-INTEGRATION]"


def is_patched(content: str) -> bool:
    return MARKER in content


def patch_server() -> bool:
    """Agrega la integración de browsers a server.py."""
    if not SERVER_PATH.exists():
        print(f"✗ No se encontró {SERVER_PATH}")
        return False

    content = SERVER_PATH.read_text()

    if is_patched(content):
        print(f"✓ {SERVER_PATH} ya está parchado")
        return True

    # Crear backup
    shutil.copy(SERVER_PATH, BACKUP_PATH)
    print(f"✓ Backup creado: {BACKUP_PATH}")

    # Buscar el punto de inserción: antes de `if __name__ == '__main__':`
    insert_point = content.rfind("\nif __name__")
    if insert_point == -1:
        # Alternativa: al final del archivo
        insert_point = len(content)

    patch_block = f"""

{MARKER}
# ── Browser Integration — QuantumEnergyOS V.02 ─────────────────────
# Agrega endpoints /api/v1/browser/* y QAOA worker en background
try:
    {INTEGRATION_IMPORT}
    integrate_browsers(app)
    import logging
    logging.getLogger("qeos.api").info(
        "✅ Browser integration activa — /api/v1/browser/"
    )
except ImportError:
    pass  # browser_integration.py no disponible — continuar sin él
# ────────────────────────────────────────────────────────────────────
"""

    new_content = content[:insert_point] + patch_block + content[insert_point:]
    SERVER_PATH.write_text(new_content)
    print(f"✓ {SERVER_PATH} parchado exitosamente")
    print(f"  Endpoints agregados: /api/v1/browser/*")
    return True


def revert_patch() -> bool:
    """Revierte el parche desde el backup."""
    if not BACKUP_PATH.exists():
        print(f"✗ No se encontró backup: {BACKUP_PATH}")
        return False
    shutil.copy(BACKUP_PATH, SERVER_PATH)
    BACKUP_PATH.unlink()
    print(f"✓ {SERVER_PATH} revertido desde backup")
    return True


def check_patch() -> None:
    """Verifica el estado del parche."""
    if not SERVER_PATH.exists():
        print(f"✗ {SERVER_PATH} no encontrado")
        return

    content = SERVER_PATH.read_text()
    if is_patched(content):
        print(f"✓ {SERVER_PATH} está parchado con browser integration")
    else:
        print(f"ℹ {SERVER_PATH} NO está parchado (ejecutar: python3 server_patch.py)")


def run_server() -> None:
    """Parchea y corre el servidor."""
    patch_server()

    # Verificar que browser_integration.py está disponible
    if not Path("browser_integration.py").exists():
        print("⚠ browser_integration.py no encontrado en el directorio actual")
        print("  Cópialo desde setup/browser_integration.py")

    print("\n⚡ Iniciando QuantumEnergyOS API con browser integration...")
    print(f"   Dashboard: http://localhost:{os.environ.get('PORT', 8000)}")
    print(f"   Browsers:  http://localhost:{os.environ.get('PORT', 8000)}/api/v1/browser/status\n")

    os.execlp(sys.executable, sys.executable, str(SERVER_PATH))


if __name__ == "__main__":
    parser = argparse.ArgumentParser(
        description="QuantumEnergyOS V.02 — Server patch para browser integration"
    )
    parser.add_argument("--check",  action="store_true", help="Verificar estado del parche")
    parser.add_argument("--revert", action="store_true", help="Revertir el parche")
    parser.add_argument("--run",    action="store_true", help="Parchear y correr el servidor")
    args = parser.parse_args()

    if args.check:
        check_patch()
    elif args.revert:
        revert_patch()
    elif args.run:
        run_server()
    else:
        success = patch_server()
        if success:
            print("\n  Para correr el servidor:")
            print(f"  python3 {SERVER_PATH}")
            print("\n  O en un paso:")
            print("  python3 server_patch.py --run")
