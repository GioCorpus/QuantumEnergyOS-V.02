#!/usr/bin/env bash
# ══════════════════════════════════════════════════════════════════════
#  QuantumEnergyOS — Build ISO Bootable
#  Genera una ISO live basada en Ubuntu 24.04 LTS con QEOS preinstalado.
#  La ISO bootea directamente en un entorno cuántico listo para usar.
#
#  Requisitos (correr en Ubuntu 22.04+ o 24.04+):
#    sudo apt-get install -y live-build squashfs-tools genisoimage xorriso
#
#  Uso:
#    chmod +x build-iso.sh
#    sudo ./build-iso.sh
#
#  Output: QuantumEnergyOS-v1.0.0-live.iso (~2.5 GB)
#  Arquitecturas: amd64 (x86_64) | arm64 (Apple M-series via UTM/QEMU)
# ══════════════════════════════════════════════════════════════════════
set -euo pipefail

QEOS_VERSION="1.0.0"
ISO_NAME="QuantumEnergyOS-v${QEOS_VERSION}-live"
BUILD_DIR="/tmp/qeos-iso-build"
ARCH="${1:-amd64}"   # amd64 o arm64
UBUNTU_VERSION="noble"  # 24.04 LTS

RED='\033[0;31m'; GREEN='\033[0;32m'; YELLOW='\033[1;33m'
CYAN='\033[0;36m'; BOLD='\033[1m'; RESET='\033[0m'

log()     { echo -e "${GREEN}[✓]${RESET} $*"; }
warn()    { echo -e "${YELLOW}[⚠]${RESET} $*"; }
error()   { echo -e "${RED}[✗]${RESET} $*"; exit 1; }
section() { echo -e "\n${CYAN}${BOLD}── $* ──${RESET}"; }

# ── Verificar root y dependencias ────────────────────────────
check_deps() {
    [ "$(id -u)" = "0" ] || error "Ejecutar como root: sudo ./build-iso.sh"

    section "Verificando dependencias"
    DEPS=(live-build squashfs-tools xorriso grub-common grub-pc-bin grub-efi-amd64-bin)
    MISSING=()

    for dep in "${DEPS[@]}"; do
        if ! dpkg -l "$dep" &>/dev/null; then
            MISSING+=("$dep")
        fi
    done

    if [ ${#MISSING[@]} -gt 0 ]; then
        warn "Instalando dependencias faltantes: ${MISSING[*]}"
        apt-get update -qq
        apt-get install -y "${MISSING[@]}"
    fi
    log "Dependencias: OK"
}

# ── Configurar live-build ─────────────────────────────────────
configure_livebuild() {
    section "Configurando live-build (Ubuntu ${UBUNTU_VERSION} ${ARCH})"

    rm -rf "$BUILD_DIR"
    mkdir -p "$BUILD_DIR"
    cd "$BUILD_DIR"

    lb config \
        --architecture "$ARCH" \
        --distribution "$UBUNTU_VERSION" \
        --archive-areas "main restricted universe multiverse" \
        --debian-installer none \
        --binary-images iso-hybrid \
        --bootloader grub-pc \
        --iso-volume "QuantumEnergyOS-${QEOS_VERSION}" \
        --iso-publisher "GioCorpus <github.com/GioCorpus>" \
        --iso-application "QuantumEnergyOS Live" \
        --memtest none \
        --apt-recommends false \
        --compression xz

    log "live-build configurado"
}

# ── Paquetes del sistema a incluir ────────────────────────────
configure_packages() {
    section "Configurando lista de paquetes"

    mkdir -p config/package-lists

    cat > config/package-lists/qeos-base.list.chroot << 'PKGS'
# ── Base del sistema ──
live-boot
live-config
systemd-sysv
network-manager
wget
curl
git
ca-certificates
gnupg
apt-transport-https
software-properties-common
build-essential
libffi-dev
libssl-dev
openssl
zlib1g-dev

# ── Python 3.11+ ──
python3.11
python3.11-venv
python3.11-dev
python3-pip
python3-setuptools
python3-wheel

# ── Jupyter / notebooks ──
jupyter-notebook

# ── Desktop mínimo (para demo gráfica) ──
xfce4
lightdm
xfce4-terminal
firefox-esr

# ── Herramientas de desarrollo ──
vim
nano
htop
tree

# ── Networking ──
openssh-client
net-tools
iputils-ping
PKGS

    log "Lista de paquetes: OK"
}

# ── Script de instalación automática de QEOS en el chroot ─────
configure_chroot_hooks() {
    section "Configurando hooks chroot"

    mkdir -p config/hooks/live

    # Hook: instalar QEOS en el sistema live
    cat > config/hooks/live/0100-install-qeos.hook.chroot << 'HOOK'
#!/bin/bash
set -euo pipefail

QEOS_DIR="/opt/QuantumEnergyOS"
QEOS_VENV="${QEOS_DIR}/venv"

echo "[QEOS] Instalando QuantumEnergyOS en el sistema live..."

# Clonar repo
git clone https://github.com/GioCorpus/QuantumEnergyOS.git "$QEOS_DIR" --depth 1 || {
    # Fallback: copiar desde /tmp si no hay internet en el build
    cp -r /tmp/QuantumEnergyOS-src "$QEOS_DIR" 2>/dev/null || mkdir -p "$QEOS_DIR"
}

# Crear venv global
python3.11 -m venv "$QEOS_VENV"
"${QEOS_VENV}/bin/pip" install --upgrade pip --quiet

# Instalar dependencias
if [ -f "${QEOS_DIR}/requirements-pinned.txt" ]; then
    "${QEOS_VENV}/bin/pip" install -r "${QEOS_DIR}/requirements-pinned.txt" --quiet
fi

# QDK
"${QEOS_VENV}/bin/pip" install qsharp --quiet 2>/dev/null || true

# Comando global
cat > /usr/local/bin/qeos << 'CMD'
#!/bin/bash
source /opt/QuantumEnergyOS/venv/bin/activate
cd /opt/QuantumEnergyOS
exec "$@"
CMD
chmod +x /usr/local/bin/qeos

echo "[QEOS] Instalación completada ✓"
HOOK

    chmod +x config/hooks/live/0100-install-qeos.hook.chroot

    # Hook: configurar autostart en el desktop
    cat > config/hooks/live/0200-configure-desktop.hook.chroot << 'HOOK'
#!/bin/bash
# Usuario live por defecto: user
LIVE_USER="user"
DESKTOP_DIR="/home/${LIVE_USER}/Desktop"
mkdir -p "$DESKTOP_DIR"

# Acceso directo QuantumEnergyOS
cat > "${DESKTOP_DIR}/QuantumEnergyOS.desktop" << DESKTOP
[Desktop Entry]
Version=1.0
Type=Application
Name=⚡ Quantum Energy OS
Comment=Energía cuántica para el mundo
Exec=/usr/local/bin/qeos uvicorn api.server:app --host 0.0.0.0 --port 8000
Icon=/opt/QuantumEnergyOS/assets/icon.png
Terminal=true
Categories=Science;Education;
DESKTOP
chmod +x "${DESKTOP_DIR}/QuantumEnergyOS.desktop"

# Auto-abrir navegador en localhost:8000
mkdir -p "/home/${LIVE_USER}/.config/autostart"
cat > "/home/${LIVE_USER}/.config/autostart/qeos-api.desktop" << AUTOSTART
[Desktop Entry]
Type=Application
Name=QuantumEnergyOS API
Exec=bash -c "sleep 5 && /usr/local/bin/qeos uvicorn api.server:app --host 0.0.0.0 --port 8000 & sleep 8 && xdg-open http://localhost:8000"
Hidden=false
NoDisplay=false
X-GNOME-Autostart-enabled=true
AUTOSTART

echo "[QEOS] Desktop configurado ✓"
HOOK
    chmod +x config/hooks/live/0200-configure-desktop.hook.chroot
    log "Hooks configurados"
}

# ── Personalizar boot splash ──────────────────────────────────
configure_boot() {
    section "Configurando boot GRUB"

    mkdir -p config/includes.binary/boot/grub

    cat > config/includes.binary/boot/grub/grub.cfg << 'GRUB'
set default=0
set timeout=5

insmod all_video
insmod gfxterm
set gfxmode=1920x1080,auto
terminal_output gfxterm

# Fondo QEOS
if background_image /boot/grub/qeos-splash.png; then
    set color_normal=white/black
    set color_highlight=cyan/black
fi

menuentry "⚡ QuantumEnergyOS Live — Iniciar" {
    linux /live/vmlinuz boot=live quiet splash noswap nopersistence hostname=QuantumEnergyOS username=quantum
    initrd /live/initrd
}

menuentry "⚡ QuantumEnergyOS Live — Modo seguro" {
    linux /live/vmlinuz boot=live noswap nopersistence hostname=QuantumEnergyOS username=quantum
    initrd /live/initrd
}

menuentry "🔧 Instalar en disco duro" {
    linux /live/vmlinuz boot=live quiet splash only-ubiquity
    initrd /live/initrd
}

menuentry "Apagar" {
    halt
}
GRUB

    log "Boot GRUB configurado"
}

# ── Construir ISO ─────────────────────────────────────────────
build_iso() {
    section "Construyendo ISO (esto puede tardar 30-60 minutos)"
    cd "$BUILD_DIR"

    log "Iniciando lb build..."
    lb build 2>&1 | tee /tmp/qeos-iso-build.log

    # Renombrar ISO final
    FINAL_ISO="${ISO_NAME}-${ARCH}.iso"
    mv live-image-${ARCH}.hybrid.iso "$FINAL_ISO" 2>/dev/null || \
    mv *.iso "$FINAL_ISO" 2>/dev/null || \
    error "No se encontró la ISO generada. Ver /tmp/qeos-iso-build.log"

    log "ISO generada: ${BOLD}${BUILD_DIR}/${FINAL_ISO}${RESET}"
}

# ── Generar checksums ─────────────────────────────────────────
generate_checksums() {
    section "Generando checksums"
    cd "$BUILD_DIR"
    FINAL_ISO="${ISO_NAME}-${ARCH}.iso"

    sha256sum "$FINAL_ISO" > "${FINAL_ISO}.sha256"
    sha512sum "$FINAL_ISO" > "${FINAL_ISO}.sha512"

    log "SHA-256: $(cat ${FINAL_ISO}.sha256 | awk '{print $1}')"
    log "SHA-512: OK"

    # Firmar con GPG si está disponible
    if command -v gpg &>/dev/null && gpg --list-secret-keys &>/dev/null; then
        gpg --armor --detach-sign "$FINAL_ISO"
        log "Firma GPG: ${FINAL_ISO}.asc"
    else
        warn "GPG no configurado — firma omitida (recomendado para releases públicos)"
    fi
}

# ── Main ──────────────────────────────────────────────────────
main() {
    echo ""
    echo -e "${CYAN}${BOLD}╔══════════════════════════════════════════════════════════╗"
    echo -e "║     ⚡  QuantumEnergyOS — Build ISO Live             ║"
    echo -e "║     Arquitectura: ${ARCH}  |  Ubuntu ${UBUNTU_VERSION}              ║"
    echo -e "║     Desde Mexicali, BC — para el mundo entero        ║"
    echo -e "╚══════════════════════════════════════════════════════════╝${RESET}"
    echo ""
    warn "Este proceso requiere ~30-60 minutos y ~10 GB de espacio en disco."
    read -r -p "¿Continuar? (s/n): " resp
    [[ "$resp" =~ ^[Ss]$ ]] || exit 0

    check_deps
    configure_livebuild
    configure_packages
    configure_chroot_hooks
    configure_boot
    build_iso
    generate_checksums

    echo ""
    echo -e "${GREEN}${BOLD}╔══════════════════════════════════════════════════════════╗"
    echo -e "║          ✅  ISO construida exitosamente              ║"
    echo -e "╚══════════════════════════════════════════════════════════╝${RESET}"
    echo ""
    echo -e "  ISO:     ${BOLD}${BUILD_DIR}/${ISO_NAME}-${ARCH}.iso${RESET}"
    echo -e "  SHA-256: ${BUILD_DIR}/${ISO_NAME}-${ARCH}.iso.sha256"
    echo ""
    echo -e "  ${BOLD}Grabar en USB (Linux/macOS):${RESET}"
    echo -e "    sudo dd if=${ISO_NAME}-${ARCH}.iso of=/dev/sdX bs=4M status=progress"
    echo -e "    o usar: balenaEtcher, Rufus (Windows)"
    echo ""
    echo -e "  ${BOLD}Correr en VM (QEMU):${RESET}"
    echo -e "    qemu-system-x86_64 -m 4G -cdrom ${ISO_NAME}-${ARCH}.iso -boot d"
    echo ""
    echo -e "  ${BOLD}Apple Silicon M1/M2/M3/M4 (UTM):${RESET}"
    echo -e "    Usar la ISO arm64 en UTM (https://mac.getutm.app/)"
    echo ""
    echo -e "  ${CYAN}⚡ Kardashev 0→1 — desde Mexicali para el mundo${RESET}"
    echo ""
}

main "$@"
