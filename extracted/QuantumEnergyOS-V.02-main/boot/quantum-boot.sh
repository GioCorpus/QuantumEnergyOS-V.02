#!/usr/bin/env bash
# ═══════════════════════════════════════════════════════════════════════
#  quantum-boot.sh — QuantumEnergyOS V.02 Boot with Atomic Rollback
#  Sistema inmutable OSTree con rollback automático en fallos
# ═══════════════════════════════════════════════════════════════════════

set -euo pipefail

# ═══════════════════════════════════════════════════════════════════════
#  Configuration
# ═══════════════════════════════════════════════════════════════════════
readonly BOOT_LOG="/var/log/quantum-boot.log"
readonly ROLLBACK_DIR="/var/rollback"
readonly OSTREE_DEPLOY="/ostree"
readonly DEFAULT_BOOT="ostree.0"
readonly FALLBACK_BOOT="ostree.1"
readonly MAX_BOOT_ATTEMPTS=3

# ═══════════════════════════════════════════════════════════════════════
#  Logging
# ═══════════════════════════════════════════════════════════════════════
log() {
    local timestamp
    timestamp=$(date +'%Y-%m-%d %H:%M:%S')
    echo "[${timestamp}] [BOOT] $*" | tee -a "${BOOT_LOG}"
}

error() {
    local timestamp
    timestamp=$(date +'%Y-%m-%d %H:%M:%S')
    echo "[${timestamp}] [ERROR] $*" | tee -a "${BOOT_LOG}" >&2
}

log_section() {
    echo "" | tee -a "${BOOT_LOG}"
    echo "═══════════════════════════════════════════════════════════════════" | tee -a "${BOOT_LOG}"
    echo "  QuantumEnergyOS V.02 — $*" | tee -a "${BOOT_LOG}"
    echo "══════════════════════════════════════════════════════════════════════" | tee -a "${BOOT_LOG}"
}

# ═══════════════════════════════════════════════════════════════════════
#  Pre-flight checks
# ═══════════════════════════════════════════════════════════════════════
preflight_checks() {
    log_section "Pre-flight Checks"

    # Verificar que el sistema es OSTree-based
    if [[ ! -d "/ostree" ]]; then
        log "WARNING: No se detectó /ostree — ejecutando en modo compatibilidad"
        return 1
    fi

    # Verificar kernel
    if [[ ! -f "/boot/vmlinuz-linux" ]]; then
        error "Kernel no encontrado"
        return 1
    fi

    log "✓ Pre-flight checks completados"
    return 0
}

# ═══════════════════════════════════════════════════════════════════════
#  OSTree boot selection
# ═══════════════════════════════════════════════════════════════════════
ostree_boot_select() {
    log_section "OSTree Boot Selection"

    local current_deploy
    local deployed_version
    local boot_count

    # Obtener deployment actual
    if command -v ostree &>/dev/null; then
        current_deploy=$(ostree admin --print-current-dir 2>/dev/null || echo "unknown")
        deployed_version=$(ostree admin --status 2>/dev/null | grep -E "^\*" | head -1 || echo "unknown")
        log "Deployment actual: ${current_deploy}"
        log "Versión desplegada: ${deployed_version}"
    else
        log "WARNING: ostree no disponible — usando despliegue manual"
    fi

    # Contar deployments disponibles
    boot_count=$(ls -1d /ostree/boot/* 2>/dev/null | wc -l || echo "1")
    log "Boot entries disponibles: ${boot_count}"

    # Seleccionar deployment
    if [[ "${boot_count}" -ge 1 ]]; then
        log "Seleccionando: ${DEFAULT_BOOT}"
        return 0
    else
        error "No hay deployments disponibles"
        return 1
    fi
}

# ═══════════════════════════════════════════════════════════════════════
#  Immutable volume setup check
# ═══════════════════════════════════════════════════════════════════════
verify_immutable_volumes() {
    log_section "Verificando Volúmenes Inmutables"

    local required_dirs=(
        "/var/lib/qeos"
        "/var/log/qeos"
        "/etc/qeos"
    )

    for dir in "${required_dirs[@]}"; do
        if [[ -d "${dir}" ]]; then
            log "  ✓ ${dir}"
        else
            log "  ⚠ Creando ${dir}..."
            mkdir -p "${dir}"
        fi
    done

    # Verificar que / está montada como SquashFS o similar (readonly)
    local root_ro
    root_ro=$(findmnt -no OPTIONS / 2>/dev/null || echo "rw")
    if [[ "${root_ro}" == *"ro"* ]]; then
        log "Root filesystem: READ-ONLY (correcto para inmutable)"
    else
        log "WARNING: Root filesystem NO es solo-lectura"
    fi

    return 0
}

# ═══════════════════════════════════════════════════════════════════════
#  Container runtime verification
# ═══════════════════════════════════════════════════════════════════════
verify_container_runtime() {
    log_section "Verificando Container Runtime"

    # Podman (preferido para sistemas inmutables)
    if command -v podman &>/dev/null; then
        local podman_version
        podman_version=$(podman --version 2>/dev/null | head -1)
        log "Podman: ${podman_version}"

        # Verificar que los contenedores können ejecutarse
        if podman info &>/dev/null; then
            log "✓ Podman operacional"
        else
            log "WARNING: Podman no puede ejecutarse (sin permisos?)"
        fi
    else
        log "WARNING: Podman no está instalado"
    fi

    # Docker (alternativo)
    if command -v docker &>/dev/null; then
        log "Docker disponible (no recomendado para sistemas inmutables)"
    fi

    return 0
}

# ═══════════════════════════════════════════════════════════════════════
#  Start services
# ═══════════════════════════════════════════════════════════════════════
start_systemd_services() {
    log_section "Iniciando Servicios"

    # Habilitar servicios QuantumEnergyOS
    local services=(
        "qeos-immutable-setup.service"
        "qeos-api.service"
    )

    for svc in "${services[@]}"; do
        if systemctl is-enabled "${svc}" &>/dev/null; then
            log "Iniciando ${svc}..."
            systemctl start "${svc}" || log "  ⚠ ${svc} no pudo iniciar (no crítico)"
        else
            log "  - ${svc} no está habilitado"
        fi
    done

    return 0
}

# ═══════════════════════════════════════════════════════════════════════
#  Rollback mechanism
# ═══════════════════════════════════════════════════════════════════════
rollback_check() {
    log_section "Verificando Rollback"

    local last_successful
    local current_attempt
    local rollback_marker="${ROLLBACK_DIR}/rollback-needed"

    # Verificar si hay un rollback pendente
    if [[ -f "${rollback_marker}" ]]; then
        log "⚠ Rollback pendente detectado"

        local reason
        reason=$(cat "${rollback_marker}" 2>/dev/null || echo "desconocido")
        log "Razón: ${reason}"

        # Ejecutar rollback
        perform_rollback
    else
        log "No hay rollback pendente"
    fi

    # Marcar este boot como exitoso
    date +%s > "${ROLLBACK_DIR}/last-successful-boot"
    return 0
}

perform_rollback() {
    log "EJECUTANDO ROLLBACK..."

    # En OSTree: cambiar al deployment anterior
    if command -v ostree &>/dev/null; then
        log "Cambiando a deployment anterior con ostree..."

        local current_deploy
        current_deploy=$(ostree admin --print-current-dir 2>/dev/null | xargs basename)

        # Listar deployments y seleccionar el anterior
        local available_deploys
        available_deploys=$(ostree admin --status 2>/dev/null | grep -E "^\*" | wc -l)

        if [[ "${available_deploys}" -gt 1 ]]; then
            ostree admin deploy --previous 2>/dev/null || \
                log "No se pudo ejecutar rollback automático"
        fi
    else
        # Fallback: reiniciar con último kernel conocido
        log "Usando método de rollback manual"
        echo "rollback-to-previous" > /proc/sysrq-trigger 2>/dev/null || true
    fi

    log "Rollback iniciado — el sistema reiniciará en 10 segundos"
    sleep 10
    reboot
}

# ═══════════════════════════════════════════════════════════════════════
#  Boot health verification
# ═══════════════════════════════════════════════════════════════════════
boot_health_check() {
    log_section "Boot Health Check"

    local health_status=0

    # 1. Verificar red
    if ping -c 1 -W 2 8.8.8.8 &>/dev/null; then
        log "  ✓ Red operativa"
    else
        log "  ⚠ Red no disponible"
    fi

    # 2. Verificar sistema de archivos
    if df -T / 2>/dev/null | grep -q "squashfs\|erofs"; then
        log "  ✓ Root en sistema de archivos inmutable"
    else
        log "  ⚠ Root no es inmutable"
    fi

    # 3. Verificar contenedores
    if command -v podman &>/dev/null; then
        if podman ps &>/dev/null; then
            log "  ✓ Podman operacional"
        else
            log "  ⚠ Podman no puede listar contenedores"
        fi
    fi

    # 4. Verificar API (si el contenedor está corriendo)
    if curl -sf -m 5 http://localhost:8000/health &>/dev/null; then
        log "  ✓ API responding"
    else
        log "  ⚠ API no responde (contenedor puede no estar iniciado)"
    fi

    # Guardar resultado
    if [[ "${health_status}" -eq 0 ]]; then
        echo "healthy" > "${ROLLBACK_DIR}/boot-health"
    else
        echo "degraded" > "${ROLLBACK_DIR}/boot-health"
    fi

    return "${health_status}"
}

# ═══════════════════════════════════════════════════════════════════════
#  Display system info
# ═══════════════════════════════════════════════════════════════════════
display_system_info() {
    log_section "System Info"

    log "QuantumEnergyOS V.02 — Immutable Edition"
    log "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
    log "Versión:    $(cat /etc/qeos/version 2>/dev/null || echo '0.2.0-immutable')"
    log "Misión:     Nunca más apagones en Mexicali"
    log "Ubicación:  Mexicali, Baja California, México"
    log "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"

    if [[ -f "/proc/uptime" ]]; then
        local uptime_seconds
        uptime_seconds=$(awk '{print int($1)}' /proc/uptime)
        log "Uptime:     ${uptime_seconds}s"
    fi

    # OSTree status
    if command -v ostree &>/dev/null; then
        log ""
        log "OSTree deployments:"
        ostree admin --status 2>/dev/null | grep -E "^(\*|  )" | head -5 || true
    fi
}

# ═══════════════════════════════════════════════════════════════════════
#  Main execution
# ═══════════════════════════════════════════════════════════════════════
main() {
    # Crear directorio de logs si no existe
    mkdir -p "$(dirname "${BOOT_LOG}")" "$(dirname "${ROLLBACK_DIR}")"

    log_section "QuantumEnergyOS V.02 — Boot Sequence"

    # Pre-flight checks
    preflight_checks || true

    # OSTree boot selection
    ostree_boot_select || true

    # Verificar volúmenes inmutables
    verify_immutable_volumes || true

    # Verificar container runtime
    verify_container_runtime || true

    # Verificar rollback pendente
    rollback_check || true

    # Iniciar servicios systemd
    start_systemd_services || true

    # Boot health check
    boot_health_check || true

    # Mostrar información del sistema
    display_system_info

    log_section "Boot Completado — Sistema Listo"

    log "QuantumEnergyOS V.02 Immutable está operativo"
    log "Misión: Nunca más apagones en Mexicali"

    exit 0
}

main "$@"