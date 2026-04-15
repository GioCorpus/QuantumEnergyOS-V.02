#!/bin/bash
# ═══════════════════════════════════════════════════════════════════════
#  QuantumEnergyOS — Compositor Installation Script
#  Installs Wayland compositor with quantum-futuristic theme
#
#  Desde Mexicali, BC — para el mundo. Kardashev 0→1
# ═══════════════════════════════════════════════════════════════════════

set -e  # Exit on error

# ── Colors ───────────────────────────────────────────────────────────
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
CYAN='\033[0;36m'
NC='\033[0m' # No Color

# ── Configuration ────────────────────────────────────────────────────
INSTALL_DIR="/usr/local"
CONFIG_DIR="/etc/quantum-compositor"
ASSETS_DIR="/usr/share/quantum-compositor"
BIN_DIR="${INSTALL_DIR}/bin"
LIB_DIR="${INSTALL_DIR}/lib"
SHARE_DIR="${INSTALL_DIR}/share"

# ── Functions ────────────────────────────────────────────────────────
print_header() {
    echo -e "${CYAN}═══════════════════════════════════════════════════════════════${NC}"
    echo -e "${CYAN}  QuantumEnergyOS — Compositor Installation${NC}"
    echo -e "${CYAN}  Ultra-lightweight Wayland compositor${NC}"
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

check_root() {
    if [ "$EUID" -ne 0 ]; then
        print_error "This script must be run as root"
        echo "Usage: sudo ./install.sh"
        exit 1
    fi
}

check_dependencies() {
    print_step "Checking dependencies..."
    
    local missing=()
    
    # Check for required commands
    for cmd in gcc make pkg-config meson ninja; do
        if ! command -v $cmd &> /dev/null; then
            missing+=($cmd)
        fi
    done
    
    # Check for required libraries
    for lib in wayland-server wlroots xkbcommon pixman libinput cairo; do
        if ! pkg-config --exists $lib 2>/dev/null; then
            missing+=($lib)
        fi
    done
    
    if [ ${#missing[@]} -ne 0 ]; then
        print_warning "Missing dependencies: ${missing[*]}"
        echo ""
        echo "Install them with:"
        echo "  sudo apt-get install ${missing[*]}"
        echo ""
        echo "Or run 'make deps' to build wlroots from source"
        exit 1
    fi
    
    print_success "All dependencies found"
}

install_binary() {
    print_step "Installing compositor binary..."
    
    if [ ! -f "build/quantum-compositor" ]; then
        print_error "Binary not found. Run 'make' first."
        exit 1
    fi
    
    install -m 755 build/quantum-compositor ${BIN_DIR}/quantum-compositor
    print_success "Binary installed to ${BIN_DIR}/quantum-compositor"
}

install_config() {
    print_step "Installing configuration..."
    
    mkdir -p ${CONFIG_DIR}
    install -m 644 themes/quantum-neon.ini ${CONFIG_DIR}/theme.ini
    print_success "Theme installed to ${CONFIG_DIR}/theme.ini"
}

install_assets() {
    print_step "Installing assets..."
    
    mkdir -p ${ASSETS_DIR}
    
    # Install wallpaper if it exists
    if [ -f "assets/wallpaper.png" ]; then
        install -m 644 assets/wallpaper.png ${ASSETS_DIR}/wallpaper.png
        print_success "Wallpaper installed to ${ASSETS_DIR}/wallpaper.png"
    else
        print_warning "Wallpaper not found. Creating placeholder..."
        # Create a simple gradient wallpaper using ImageMagick if available
        if command -v convert &> /dev/null; then
            convert -size 1920x1080 \
                gradient:"#0a0a1a-#001a33" \
                -fill "rgba(0,212,255,0.1)" -draw "circle 960,540 960,800" \
                ${ASSETS_DIR}/wallpaper.png
            print_success "Generated placeholder wallpaper"
        else
            print_warning "ImageMagick not found. Install manually or add wallpaper.png"
        fi
    fi
    
    # Install particle sprites
    mkdir -p ${ASSETS_DIR}/particles
    for i in {1..5}; do
        if command -v convert &> /dev/null; then
            convert -size 10x10 xc:transparent \
                -fill "#00d4ff" -draw "circle 5,5 5,8" \
                -blur 0x2 \
                ${ASSETS_DIR}/particles/particle_${i}.png
        fi
    done
    print_success "Particle sprites installed"
}

install_energy_daemon() {
    print_step "Installing energy daemon..."
    
    install -m 755 src/energy-daemon.py ${BIN_DIR}/quantum-energy-daemon
    print_success "Energy daemon installed to ${BIN_DIR}/quantum-energy-daemon"
    
    # Create systemd service
    cat > /etc/systemd/system/quantum-energy.service << EOF
[Unit]
Description=QuantumEnergyOS Energy Predictor Daemon
After=network.target

[Service]
Type=simple
ExecStart=${BIN_DIR}/quantum-energy-daemon
Restart=on-failure
RestartSec=5

[Install]
WantedBy=multi-user.target
EOF
    
    print_success "Systemd service created: quantum-energy.service"
}

create_desktop_entry() {
    print_step "Creating desktop entry..."
    
    mkdir -p ${SHARE_DIR}/applications
    cat > ${SHARE_DIR}/applications/quantum-compositor.desktop << EOF
[Desktop Entry]
Name=QuantumEnergyOS Compositor
Comment=Ultra-lightweight Wayland compositor with quantum-futuristic theme
Exec=${BIN_DIR}/quantum-compositor
Type=Application
Categories=System;
Keywords=wayland;compositor;quantum;energy;
Icon=video-display
Terminal=false
StartupNotify=true
EOF
    
    print_success "Desktop entry created"
}

setup_permissions() {
    print_step "Setting up permissions..."
    
    # Create socket directory
    mkdir -p /var/run
    chmod 755 /var/run
    
    # Create log directory
    mkdir -p /var/log
    touch /var/log/quantum-energy.log
    chmod 644 /var/log/quantum-energy.log
    
    print_success "Permissions configured"
}

print_summary() {
    echo ""
    echo -e "${CYAN}═══════════════════════════════════════════════════════════════${NC}"
    echo -e "${GREEN}  Installation Complete!${NC}"
    echo -e "${CYAN}═══════════════════════════════════════════════════════════════${NC}"
    echo ""
    echo -e "  Binary:    ${BIN_DIR}/quantum-compositor"
    echo -e "  Config:    ${CONFIG_DIR}/theme.ini"
    echo -e "  Assets:    ${ASSETS_DIR}/"
    echo -e "  Daemon:    ${BIN_DIR}/quantum-energy-daemon"
    echo ""
    echo -e "${YELLOW}  Quick Start:${NC}"
    echo -e "    1. Start energy daemon:"
    echo -e "       ${CYAN}sudo systemctl start quantum-energy${NC}"
    echo -e "    2. Enable on boot:"
    echo -e "       ${CYAN}sudo systemctl enable quantum-energy${NC}"
    echo -e "    3. Run compositor:"
    echo -e "       ${CYAN}quantum-compositor -s foot${NC}"
    echo ""
    echo -e "${YELLOW}  QEMU Testing:${NC}"
    echo -e "    ${CYAN}make test${NC}"
    echo ""
    echo -e "${YELLOW}  Documentation:${NC}"
    echo -e "    ${CYAN}cat README.md${NC}"
    echo ""
}

# ── Main Installation ────────────────────────────────────────────────
main() {
    print_header
    check_root
    check_dependencies
    install_binary
    install_config
    install_assets
    install_energy_daemon
    create_desktop_entry
    setup_permissions
    print_summary
}

# Run main function
main "$@"
