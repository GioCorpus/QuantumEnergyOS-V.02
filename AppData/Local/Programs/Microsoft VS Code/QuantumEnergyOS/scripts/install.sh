# QuantumEnergyOS V.02 - Installation Scripts

# ===========================================
# Quick Install Script
# Run as: sudo bash scripts/install.sh
# ===========================================

#!/bin/bash

set -e

echo "╔═══════════════════════════════════════════════════════════╗"
echo "║        QuantumEnergyOS V.02 - Installer                  ║"
echo "║        Made in Mexicali with 22 years of grind            ║"
echo "╚═══════════════════════════════════════════════════════════╝"
echo ""

# Check if running as root
if [ "$EUID" -ne 0 ]; then
    echo "Please run as root or with sudo"
    exit 1
fi

echo "[1/5] Updating package database..."
pacman -Sy --noconfirm

echo "[2/5] Installing base dependencies..."
pacman -S --noconfirm \
    base-devel \
    python \
    python-pip \
    git \
    rust \
    docker \
    qemu \
    vim \
    nano \
    wget \
    curl \
    htop \
    make \
    cmake

echo "[3/5] Installing Python packages..."
pip install --break-system-packages flask flask-cors

echo "[4/5] Creating user groups..."
groupadd docker 2>/dev/null || true
usermod -aG docker $SUDO_USER 2>/dev/null || true

echo "[5/5] Copying application files..."
cp -r photonic-bridge /opt/quantumenergyos/
cp -r photonic-core /opt/quantumenergyos/
cp -r api /opt/quantumenergyos/
cp -r web-dashboard /opt/quantumenergyos/

mkdir -p /etc/quantumenergyos
cp -r config/* /etc/quantumenergyos/ 2>/dev/null || true

echo ""
echo "╔═══════════════════════════════════════════════════════════╗"
echo "║  Installation Complete!                                    ║"
echo "╚═══════════════════════════════════════════════════════════╝"
echo ""
echo "Next steps:"
echo "  1. Start API Server: cd /opt/quantumenergyos && python api/server.py"
echo "  2. Build ISO: cd /opt/quantumenergyos && make build-iso"
echo ""
echo "Author: Giovanny Corpus Bernal - Mexicali, BC"