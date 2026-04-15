#!/bin/bash
# ═══════════════════════════════════════════════════════════════════════
#  QuantumEnergyOS — ISO Builder with Wayland Compositor
#  Builds bootable ISO with compositor in initramfs
#
#  Boot sequence: GRUB → kernel → initramfs → shell → tinywl → desktop
#  Splash: "QuantumEnergyOS – Mexicali no se apaga"
#
#  Desde Mexicali, BC — para el mundo. Kardashev 0→1
# ═══════════════════════════════════════════════════════════════════════

set -e

# ── Colors ───────────────────────────────────────────────────────────
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
CYAN='\033[0;36m'
NC='\033[0m'

# ── Configuration ────────────────────────────────────────────────────
ISO_NAME="QuantumEnergyOS-1.0.0-amd64.iso"
ISO_LABEL="QEnergOS"
WORK_DIR="build/iso"
ROOTFS_DIR="${WORK_DIR}/rootfs"
INITRAMFS_DIR="${WORK_DIR}/initramfs"
ISO_DIR="${WORK_DIR}/iso"

# Kernel (adjust to your kernel path)
KERNEL_PATH="/boot/vmlinuz-linux"
INITRD_PATH="/boot/initramfs-linux.img"

# Compositor
COMPOSITOR_BIN="build/quantum-compositor"
ENERGY_DAEMON="src/energy-daemon.py"

# ── Functions ────────────────────────────────────────────────────────
print_header() {
    echo -e "${CYAN}═══════════════════════════════════════════════════════════════${NC}"
    echo -e "${CYAN}  QuantumEnergyOS — ISO Builder${NC}"
    echo -e "${CYAN}  Building bootable ISO with Wayland compositor${NC}"
    echo -e "${CYAN}═══════════════════════════════════════════════════════════════${NC}"
    echo ""
}

print_step() {
    echo -e "${BLUE}[*]${NC} $1"
}

print_success() {
    echo -e "${GREEN}[✓]${NC} $1"
}

print_warning() {
    echo -e "${YELLOW}[!]${NC} $1"
}

print_error() {
    echo -e "${RED}[✗]${NC} $1"
}

check_dependencies() {
    print_step "Checking dependencies..."
    
    local missing=()
    
    for cmd in mksquashfs grub-mkrescue xorriso cpio gzip; do
        if ! command -v $cmd &> /dev/null; then
            missing+=($cmd)
        fi
    done
    
    if [ ${#missing[@]} -ne 0 ]; then
        print_error "Missing dependencies: ${missing[*]}"
        echo "Install with: sudo apt-get install squashfs-tools grub-pc-bin xorriso"
        exit 1
    fi
    
    print_success "All dependencies found"
}

clean_work_dir() {
    print_step "Cleaning work directory..."
    rm -rf ${WORK_DIR}
    mkdir -p ${ROOTFS_DIR} ${INITRAMFS_DIR} ${ISO_DIR}
    print_success "Work directory ready"
}

build_compositor() {
    print_step "Building compositor..."
    
    if [ ! -f "${COMPOSITOR_BIN}" ]; then
        make clean
        make
    fi
    
    if [ ! -f "${COMPOSITOR_BIN}" ]; then
        print_error "Compositor build failed"
        exit 1
    fi
    
    print_success "Compositor built: ${COMPOSITOR_BIN}"
}

create_rootfs() {
    print_step "Creating root filesystem..."
    
    # Create basic directory structure
    mkdir -p ${ROOTFS_DIR}/{bin,sbin,etc,proc,sys,dev,tmp,var,run,usr/{bin,sbin,lib,share}}
    
    # Copy compositor
    install -m 755 ${COMPOSITOR_BIN} ${ROOTFS_DIR}/usr/bin/quantum-compositor
    install -m 755 ${ENERGY_DAEMON} ${ROOTFS_DIR}/usr/bin/quantum-energy-daemon
    
    # Copy theme
    mkdir -p ${ROOTFS_DIR}/etc/quantum-compositor
    install -m 644 themes/quantum-neon.ini ${ROOTFS_DIR}/etc/quantum-compositor/theme.ini
    
    # Copy assets
    mkdir -p ${ROOTFS_DIR}/usr/share/quantum-compositor
    if [ -f "assets/wallpaper.png" ]; then
        install -m 644 assets/wallpaper.png ${ROOTFS_DIR}/usr/share/quantum-compositor/
    fi
    
    # Copy splash screen
    install -m 755 src/splash.sh ${ROOTFS_DIR}/usr/bin/quantum-splash
    
    # Copy init script
    install -m 755 src/init ${ROOTFS_DIR}/sbin/init
    
    # Copy required libraries (wlroots, wayland, etc.)
    print_step "Copying system libraries..."
    
    # wlroots
    if [ -f "/usr/local/lib/libwlroots.so" ]; then
        cp -a /usr/local/lib/libwlroots.so* ${ROOTFS_DIR}/usr/lib/
    fi
    
    # wayland-server
    if [ -f "/usr/lib/libwayland-server.so" ]; then
        cp -a /usr/lib/libwayland-server.so* ${ROOTFS_DIR}/usr/lib/
    fi
    
    # xkbcommon
    if [ -f "/usr/lib/libxkbcommon.so" ]; then
        cp -a /usr/lib/libxkbcommon.so* ${ROOTFS_DIR}/usr/lib/
    fi
    
    # pixman
    if [ -f "/usr/lib/libpixman-1.so" ]; then
        cp -a /usr/lib/libpixman-1.so* ${ROOTFS_DIR}/usr/lib/
    fi
    
    # libinput
    if [ -f "/usr/lib/libinput.so" ]; then
        cp -a /usr/lib/libinput.so* ${ROOTFS_DIR}/usr/lib/
    fi
    
    # cairo
    if [ -f "/usr/lib/libcairo.so" ]; then
        cp -a /usr/lib/libcairo.so* ${ROOTFS_DIR}/usr/lib/
    fi
    
    # Copy shell (bash or busybox)
    if command -v busybox &> /dev/null; then
        cp $(which busybox) ${ROOTFS_DIR}/bin/
        cd ${ROOTFS_DIR}/bin && ln -sf busybox sh && cd -
    else
        cp /bin/bash ${ROOTFS_DIR}/bin/
        # Copy bash dependencies
        ldd /bin/bash | grep -o '/[^ ]*' | while read lib; do
            if [ -f "$lib" ]; then
                mkdir -p ${ROOTFS_DIR}$(dirname $lib)
                cp -a $lib ${ROOTFS_DIR}$lib
            fi
        done
    fi
    
    # Copy essential utilities
    for cmd in ls cat echo mount umount mkdir rm cp mv; do
        if command -v busybox &> /dev/null; then
            cd ${ROOTFS_DIR}/bin && ln -sf busybox $cmd && cd -
        else
            cp $(which $cmd) ${ROOTFS_DIR}/bin/ 2>/dev/null || true
        fi
    done
    
    print_success "Root filesystem created"
}

create_initramfs() {
    print_step "Creating initramfs with compositor..."
    
    # Create initramfs directory structure
    mkdir -p ${INITRAMFS_DIR}/{bin,sbin,etc,proc,sys,dev,tmp,usr/{bin,lib}}
    
    # Copy init script
    install -m 755 src/init ${INITRAMFS_DIR}/init
    
    # Copy compositor to initramfs
    install -m 755 ${COMPOSITOR_BIN} ${INITRAMFS_DIR}/usr/bin/quantum-compositor
    install -m 755 ${ENERGY_DAEMON} ${INITRAMFS_DIR}/usr/bin/quantum-energy-daemon
    install -m 755 src/splash.sh ${INITRAMFS_DIR}/usr/bin/quantum-splash
    
    # Copy theme
    mkdir -p ${INITRAMFS_DIR}/etc/quantum-compositor
    install -m 644 themes/quantum-neon.ini ${INITRAMFS_DIR}/etc/quantum-compositor/theme.ini
    
    # Copy shell
    if command -v busybox &> /dev/null; then
        cp $(which busybox) ${INITRAMFS_DIR}/bin/
        cd ${INITRAMFS_DIR}/bin && ln -sf busybox sh && cd -
    else
        cp /bin/bash ${INITRAMFS_DIR}/bin/
    fi
    
    # Copy essential libraries for compositor
    for lib in libwlroots.so libwayland-server.so libxkbcommon.so libpixman-1.so libinput.so libcairo.so; do
        if [ -f "/usr/local/lib/${lib}" ]; then
            cp -a /usr/local/lib/${lib}* ${INITRAMFS_DIR}/usr/lib/
        elif [ -f "/usr/lib/${lib}" ]; then
            cp -a /usr/lib/${lib}* ${INITRAMFS_DIR}/usr/lib/
        fi
    done
    
    # Copy splash screen assets
    mkdir -p ${INITRAMFS_DIR}/usr/share/quantum-compositor
    if [ -f "assets/splash.png" ]; then
        install -m 644 assets/splash.png ${INITRAMFS_DIR}/usr/share/quantum-compositor/
    fi
    
    # Copy audio file for splash (Guilty Gear style rock)
    if [ -f "assets/quantum-rock.ogg" ]; then
        install -m 644 assets/quantum-rock.ogg ${INITRAMFS_DIR}/usr/share/quantum-compositor/
    fi
    
    # Create initramfs archive
    cd ${INITRAMFS_DIR}
    find . | cpio -o -H newc | gzip > ${WORK_DIR}/initramfs.gz
    cd -
    
    print_success "Initramfs created with compositor"
}

create_splash_screen() {
    print_step "Creating splash screen..."
    
    cat > src/splash.sh << 'EOF'
#!/bin/sh
# ═══════════════════════════════════════════════════════════════════════
#  QuantumEnergyOS — Splash Screen
#  "QuantumEnergyOS – Mexicali no se apaga"
#  Guilty Gear-style rock vibes (Daisuke Ishiwatari)
# ═══════════════════════════════════════════════════════════════════════

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
CYAN='\033[0;36m'
MAGENTA='\033[0;35m'
NC='\033[0m'

# Clear screen
clear

# Play rock music in background (if available)
if [ -f "/usr/share/quantum-compositor/quantum-rock.ogg" ]; then
    if command -v aplay &> /dev/null; then
        aplay /usr/share/quantum-compositor/quantum-rock.ogg 2>/dev/null &
    elif command -v paplay &> /dev/null; then
        paplay /usr/share/quantum-compositor/quantum-rock.ogg 2>/dev/null &
    fi
fi

# Splash animation
echo ""
echo -e "${CYAN}═══════════════════════════════════════════════════════════════${NC}"
echo -e "${CYAN}                                                               ${NC}"
echo -e "${CYAN}   ██████╗ ██╗   ██╗ █████╗ ███╗   ██╗████████╗██╗   ██╗███╗ ${NC}"
echo -e "${CYAN}  ██╔═══██╗██║   ██║██╔══██╗████╗  ██║╚══██╔══╝██║   ██║████╗${NC}"
echo -e "${CYAN}  ██║   ██║██║   ██║███████║██╔██╗ ██║   ██║   ██║   ██║██╔██╗${NC}"
echo -e "${CYAN}  ██║▄▄ ██║██║   ██║██╔══██║██║╚██╗██║   ██║   ██║   ██║██║╚██╗${NC}"
echo -e "${CYAN}  ╚██████╔╝╚██████╔╝██║  ██║██║ ╚████║   ██║   ╚██████╔╝██║ ╚████╗${NC}"
echo -e "${CYAN}   ╚══▀▀═╝  ╚═════╝ ╚═╝  ╚═╝╚═╝  ╚═══╝   ╚═╝    ╚═════╝ ╚═╝  ╚═══╝${NC}"
echo -e "${CYAN}                                                               ${NC}"
echo -e "${CYAN}  ███████╗███╗   ██╗███████╗██████╗  ██████╗ ███████╗          ${NC}"
echo -e "${CYAN}  ██╔════╝████╗  ██║██╔════╝██╔══██╗██╔═══██╗██╔════╝          ${NC}"
echo -e "${CYAN}  █████╗  ██╔██╗ ██║█████╗  ██████╔╝██║   ██║███████╗          ${NC}"
echo -e "${CYAN}  ██╔══╝  ██║╚██╗██║██╔══╝  ██╔══██╗██║   ██║╚════██║          ${NC}"
echo -e "${CYAN}  ███████╗██║ ╚████║███████╗██║  ██║╚██████╔╝███████║          ${NC}"
echo -e "${CYAN}  ╚══════╝╚═╝  ╚═══╝╚══════╝╚═╝  ╚═╝ ╚═════╝ ╚══════╝          ${NC}"
echo -e "${CYAN}                                                               ${NC}"
echo -e "${CYAN}═══════════════════════════════════════════════════════════════${NC}"
echo ""

# Animated message
echo -e "${YELLOW}  ╔═══════════════════════════════════════════════════════════╗${NC}"
echo -e "${YELLOW}  ║                                                           ║${NC}"
echo -e "${YELLOW}  ║   ${MAGENTA}QuantumEnergyOS – Mexicali no se apaga${YELLOW}              ║${NC}"
echo -e "${YELLOW}  ║                                                           ║${NC}"
echo -e "${YELLOW}  ║   ${CYAN}Desde el desierto, para el mundo${YELLOW}                    ║${NC}"
echo -e "${YELLOW}  ║   ${CYAN}Kardashev 0 → 1${YELLOW}                                    ║${NC}"
echo -e "${YELLOW}  ║                                                           ║${NC}"
echo -e "${YELLOW}  ╚═══════════════════════════════════════════════════════════╝${NC}"
echo ""

# Loading animation
echo -ne "${GREEN}  Iniciando sistema cuántico${NC}"
for i in 1 2 3 4 5; do
    echo -ne "${GREEN}.${NC}"
    sleep 0.3
done
echo ""

echo -ne "${GREEN}  Cargando compositor Wayland${NC}"
for i in 1 2 3 4 5; do
    echo -ne "${GREEN}.${NC}"
    sleep 0.3
done
echo ""

echo -ne "${GREEN}  Conectando predictor de energía${NC}"
for i in 1 2 3 4 5; do
    echo -ne "${GREEN}.${NC}"
    sleep 0.3
done
echo ""

echo ""
echo -e "${CYAN}  ════════════════════════════════════════════════════════════${NC}"
echo -e "${GREEN}  ✓ Sistema listo. Iniciando escritorio...${NC}"
echo -e "${CYAN}  ════════════════════════════════════════════════════════════${NC}"
echo ""

sleep 2
EOF
    
    chmod +x src/splash.sh
    print_success "Splash screen created"
}

create_init_script() {
    print_step "Creating init script..."
    
    cat > src/init << 'EOF'
#!/bin/sh
# ═══════════════════════════════════════════════════════════════════════
#  QuantumEnergyOS — Init Script
#  Boot sequence: shell → splash → tinywl → desktop
#
#  Desde Mexicali, BC — para el mundo. Kardashev 0→1
# ═══════════════════════════════════════════════════════════════════════

export PATH=/usr/bin:/bin:/sbin:/usr/sbin

# Mount essential filesystems
mount -t proc proc /proc
mount -t sysfs sysfs /sys
mount -t devtmpfs devtmpfs /dev
mount -t tmpfs tmpfs /tmp
mount -t tmpfs tmpfs /run

# Create necessary directories
mkdir -p /dev/pts /dev/shm
mount -t devpts devpts /dev/pts
mount -t tmpfs tmpfs /dev/shm

# Set up console
echo "QuantumEnergyOS — Iniciando..." > /dev/console

# Show splash screen
/usr/bin/quantum-splash

# Start energy daemon in background
/usr/bin/quantum-energy-daemon &

# Wait a moment for daemon to start
sleep 2

# Start compositor with terminal
echo "Iniciando compositor Wayland..." > /dev/console
/usr/bin/quantum-compositor -s /bin/sh

# If compositor exits, drop to shell
echo "Compositor terminado. Shell disponible." > /dev/console
exec /bin/sh
EOF
    
    chmod +x src/init
    print_success "Init script created"
}

build_iso() {
    print_step "Building ISO..."
    
    # Create ISO directory structure
    mkdir -p ${ISO_DIR}/boot/grub
    mkdir -p ${ISO_DIR}/EFI/BOOT
    
    # Copy kernel
    if [ -f "${KERNEL_PATH}" ]; then
        cp ${KERNEL_PATH} ${ISO_DIR}/boot/vmlinuz
    else
        print_warning "Kernel not found at ${KERNEL_PATH}, using placeholder"
        echo "placeholder" > ${ISO_DIR}/boot/vmlinuz
    fi
    
    # Copy initramfs
    cp ${WORK_DIR}/initramfs.gz ${ISO_DIR}/boot/initramfs.gz
    
    # Create GRUB config
    cat > ${ISO_DIR}/boot/grub/grub.cfg << EOF
set timeout=5
set default=0

menuentry "QuantumEnergyOS — Mexicali no se apaga" {
    linux /boot/vmlinuz quiet splash
    initrd /boot/initramfs.gz
}

menuentry "QuantumEnergyOS — Modo texto" {
    linux /boot/vmlinuz
    initrd /boot/initramfs.gz
}
EOF
    
    # Create UEFI boot
    if [ -f "/usr/lib/grub/x86_64-efi/monolithic/grub.efi" ]; then
        cp /usr/lib/grub/x86_64-efi/monolithic/grub.efi ${ISO_DIR}/EFI/BOOT/BOOTX64.EFI
    fi
    
    # Build ISO
    grub-mkrescue \
        --output=${ISO_NAME} \
        --volid=${ISO_LABEL} \
        --compress=xz \
        ${ISO_DIR}
    
    # Verify BIOS boot sector exists
    if [ -f "${ISO_DIR}/boot/grub/i386-pc/core.img" ]; then
        print_success "BIOS boot sector verified"
    else
        print_warning "BIOS boot sector not found—ISO may not boot on legacy systems"
    fi
    
    print_success "ISO built: ${ISO_NAME}"
}

print_summary() {
    echo ""
    echo -e "${CYAN}═══════════════════════════════════════════════════════════════${NC}"
    echo -e "${GREEN}  ISO Build Complete!${NC}"
    echo -e "${CYAN}═══════════════════════════════════════════════════════════════${NC}"
    echo ""
    echo -e "  ISO: ${ISO_NAME}"
    echo -e "  Size: $(du -h ${ISO_NAME} | cut -f1)"
    echo ""
    echo -e "${YELLOW}  Boot Sequence:${NC}"
    echo -e "    1. GRUB menu"
    echo -e "    2. Kernel + initramfs"
    echo -e "    3. Splash: 'QuantumEnergyOS – Mexicali no se apaga'"
    echo -e "    4. Energy daemon"
    echo -e "    5. Wayland compositor (tinywl)"
    echo -e "    6. Desktop environment"
    echo ""
    echo -e "${YELLOW}  Test in QEMU:${NC}"
    echo -e "    ${CYAN}qemu-system-x86_64 -enable-kvm -m 256M -vga virtio -cdrom ${ISO_NAME}${NC}"
    echo ""
    echo -e "${YELLOW}  Write to USB:${NC}"
    echo -e "    ${CYAN}sudo dd if=${ISO_NAME} of=/dev/sdX bs=4M status=progress${NC}"
    echo ""
}

# ── Main Build ───────────────────────────────────────────────────────
main() {
    print_header
    check_dependencies
    clean_work_dir
    build_compositor
    create_splash_screen
    create_init_script
    create_rootfs
    create_initramfs
    build_iso
    print_summary
}

main "$@"
