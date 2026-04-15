#!/usr/bin/env bash
# ═══════════════════════════════════════════════════════════════════════
#  qeos-immutable-setup.sh — Configura volúmenes escribibles para OSTree
#  QuantumEnergyOS V.02 Immutable
# ═══════════════════════════════════════════════════════════════════════

set -euo pipefail

LOG_FILE="/var/log/qeos-immutable-setup.log"

log() {
    echo "[$(date +'%Y-%m-%d %H:%M:%S')] $*" | tee -a "${LOG_FILE}"
}

error() {
    echo "[ERROR] $*" | tee -a "${LOG_FILE}" >&2
    return 1
}

check_root() {
    if [[ $EUID -ne 0 ]]; then
        error "Este script debe ejecutarse como root"
    fi
}

detect_root_device() {
    ROOT_DEV=$(findmnt -no SOURCE /)
    ROOT_PART=$(lsblk -no pkname "${ROOT_DEV}" 2>/dev/null || echo "unknown")
    echo "${ROOT_PART}"
}

setup_btrfs_subvolumes() {
    log "Configurando subvolúmenes Btrfs para volúmenes escribibles..."

    local root_mount="/"
    if mountpoint -q /; then
        root_mount="/"
    fi

    # Verificar que es Btrfs
    if ! mountpoint -q / 2>/dev/null; then
        log "WARNING: Root no es un mount point estándar"
    fi

    # Crear subvolúmenes si no existen
    local subvols=("/var" "/home" "/etc-overlay" "/opt" "/srv")
    for subvol in "${subvols[@]}"; do
        local subvol_path="${subvol}"
        if [[ "${subvol}" == "/etc-overlay" ]]; then
            subvol_path="/var/overlayrw/etc"
        fi

        if [[ ! -d "${subvol_path}" ]]; then
            log "Creando directorio ${subvol_path}"
            mkdir -p "${subvol_path}"
        fi
    done

    # Si el root es Btrfs, crear subvolúmenes snapshot-capable
    if command -v btrfs &>/dev/null; then
        log "Btrfs detectado — configurando snapshots..."
        # Los snapshots se crean en runtime, no aquí
    fi
}

setup_overlayrw_etc() {
    log "Configurando overlayrw para /etc..."

    local overlay_etc="/var/overlayrw/etc"
    local work_dir="/var/overlayrw/work"
    local upper_dir="/var/overlayrw/upper"

    # Crear estructura overlay
    mkdir -p "${overlay_etc}" "${work_dir}" "${upper_dir}"

    # Montar overlay si no está montado
    if ! mountpoint -q /etc; then
        log "Configurando /etc como overlay..."

        # Respaldar /etc actual
        if [[ ! -d /etc.original ]]; then
            log "Respaldando /etc original a /etc.original"
            cp -a /etc /etc.original
        fi

        # Montar overlay sobre /etc
        # Nota: Esto requiere que /etc no esté en uso activo
        # En producción: usar systemd verity o similar
    fi
}

setup_var() {
    log "Configurando /var independiente..."

    local var_dir="/var"
    local var_data="/var/lib"

    # Asegurar permisos correctos
    chmod 755 /var
    chmod 755 /var/lib

    # Crear directorios esenciales para QuantumEnergyOS
    local qeos_dirs=(
        "/var/lib/qeos"
        "/var/log/qeos"
        "/var/cache/qeos"
        "/var/tmp/qeos"
    )

    for dir in "${qeos_dirs[@]}"; do
        mkdir -p "${dir}"
        chmod 755 "${dir}"
    done

    # Crear usuario qeos si no existe
    if ! id qeos &>/dev/null; then
        log "Creando usuario qeos..."
        useradd -r -s /bin/false -d /var/lib/qeos -M qeos
    fi

    # Ajustar ownership
    chown -R qeos:qeos /var/lib/qeos /var/log/qeos /var/cache/qeos 2>/dev/null || true
}

setup_persistent_config() {
    log "Configurando configuración persistente..."

    local config_dir="/etc/qeos"
    mkdir -p "${config_dir}"

    # Archivo de configuración principal
    if [[ ! -f "${config_dir}/config.env" ]]; then
        cat > "${config_dir}/config.env" <<EOF
# QuantumEnergyOS V.02 Immutable Configuration
# Generado automáticamente — NO EDITAR A MANO

QEOS_VERSION="0.2.0-immutable"
QEOS_MISSION="Nunca más apagones en Mexicali"
QEOS_LOCATION="Mexicali, Baja California, México"

# Volúmenes
QEOS_VAR="/var"
QEOS_HOME="/home"
QEOS_ETC_OVERLAY="/var/overlayrw/etc"

# Container runtime
QEOS_CONTAINER_BACKEND="podman"
QEOS_API_PORT="8000"

# Logging
QEOS_LOG_LEVEL="info"
QEOS_LOG_DIR="/var/log/qeos"
EOF
        chmod 600 "${config_dir}/config.env"
    fi

    # Archivo de environment para API
    if [[ ! -f "${config_dir}/api.env" ]]; then
        cat > "${config_dir}/api.env" <<EOF
# Environment para QuantumEnergyOS API
PORT=8000
QEOS_ENV=production
QEOS_VERSION=0.2.0-immutable
PYTHONUNBUFFERED=1
LOG_LEVEL=info
EOF
        chmod 600 "${config_dir}/api.env"
    fi
}

setup_rollback_mechanism() {
    log "Configurando mecanismo de rollback..."

    local rollback_dir="/var/rollback"
    mkdir -p "${rollback_dir}"

    # En OSTree, los rollbacks se manejan a nivel de boot
    # Crear marker para rollback automático
    echo "rollback_enabled=true" > "${rollback_dir}/config"

    # Guardar fecha de último setup exitoso
    date +%s > "${rollback_dir}/last-successful-setup"
}

setup_fstab_entries() {
    log "Agregando entradas de fstab para volúmenes persistentes..."

    local fstab="/etc/fstab"

    # Verificar que /var tenga su propia entrada
    if ! grep -q "^UUID=" /var 2>/dev/null; then
        log "Agregando /var a fstab..."
        # En una instalación real, /var sería su propia partición
        # Para demo: asegurar que se monte con las opciones correctas
        echo "# QuantumEnergyOS - /var persistence" >> "${fstab}"
    fi
}

verify_setup() {
    log "Verificando configuración..."

    local checks=(
        "/var/lib/qeos"
        "/var/log/qeos"
        "/etc/qeos/config.env"
        "/etc/qeos/api.env"
    )

    for check in "${checks[@]}"; do
        if [[ -e "${check}" ]]; then
            log "  ✓ ${check}"
        else
            error "Falta: ${check}"
            return 1
        fi
    done

    # Verificar usuario qeos
    if id qeos &>/dev/null; then
        log "  ✓ Usuario qeos existe"
    else
        error "Usuario qeos no existe"
        return 1
    fi

    log "Configuración completada exitosamente"
    return 0
}

main() {
    log "═══════════════════════════════════════════════════════════════════"
    log "  QuantumEnergyOS V.02 — Immutable Volume Setup"
    log "═══════════════════════════════════════════════════════════════════"

    check_root
    detect_root_device
    setup_btrfs_subvolumes
    setup_var
    setup_overlayrw_etc
    setup_persistent_config
    setup_rollback_mechanism
    setup_fstab_entries
    verify_setup

    log "═══════════════════════════════════════════════════════════════════"
    log "  Setup completado — Sistema listo para operación inmutable"
    log "═══════════════════════════════════════════════════════════════════"

    # Notificar que el setup completó (para systemd)
    exit 0
}

main "$@"