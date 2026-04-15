#!/bin/bash
# ──────────────────────────────────────────────────────────────────────────────
# install-arch.sh — QuantumEnergyOS Installation Script for Arch Linux
# and Arch-based distributions (Manjaro, EndeavourOS, ArcoLinux, etc.)
#
# Usage: chmod +x install-arch.sh && ./install-arch.sh
# ──────────────────────────────────────────────────────────────────────────────

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

echo -e "${BLUE}╔══════════════════════════════════════════════════════════════╗${NC}"
echo -e "${BLUE}║     QuantumEnergyOS - Arch Linux Installation Script        ║${NC}"
echo -e "${BLUE}╚══════════════════════════════════════════════════════════════╝${NC}"
echo ""

# Check if running as root or with sudo
if [[ $EUID -ne 0 ]]; then
    echo -e "${YELLOW}Note: This script uses sudo for package installation.${NC}"
    echo -e "${YELLOW}You may be prompted for your password.${NC}"
    echo ""
fi

# Detect Arch-based distribution
echo -e "${BLUE}[1/6] Detecting distribution...${NC}"
if [[ -f /etc/arch-release ]]; then
    echo "Found: Arch Linux"
elif [[ -f /etc/manjaro-release ]]; then
    source /etc/manjaro-release
    echo "Found: Manjaro Linux ($MANJARO_VERSION)"
elif [[ -f /etc/endeavouros-release ]]; then
    echo "Found: EndeavourOS"
elif [[ -f /etc/lsb-release ]]; then
    source /etc/lsb-release
    echo "Found: $DISTRIB_DESCRIPTION"
else
    echo -e "${YELLOW}Warning: Could not detect specific distribution. Assuming Arch Linux.${NC}"
fi

# Update system
echo ""
echo -e "${BLUE}[2/6] Updating system packages...${NC}"
sudo pacman -Sy --noconfirm

# Install system dependencies
echo ""
echo -e "${BLUE}[3/6] Installing system dependencies...${NC}"

# Base development packages
DEPS_BASE=(
    python
    python-pip
    python-venv
    base-devel
)

# Additional dependencies for Python packages
DEPS_EXTRA=(
    openssl
    libffi
    zlib
    bzip2
    readline
    sqlite
    wget
    curl
    git
    ca-certificates
    postgresql-libs
    libxml2
    libxslt
    openblas
    lapack
    gcc-fortran
    pkgconf
)

# Detect if using pacman (Arch) or pamac (Manjaro)
if command -v pacman &> /dev/null; then
    echo "    Installing with pacman..."
    sudo pacman -S --noconfirm "${DEPS_BASE[@]}" "${DEPS_EXTRA[@]}"
elif command -v pamac &> /dev/null; then
    echo "    Installing with pamac..."
    sudo pamac install --no-confirm "${DEPS_BASE[@]}" "${DEPS_EXTRA[@]}"
else
    echo -e "${RED}Error: Neither pacman nor pamac found. Cannot install packages.${NC}"
    exit 1
fi

# Verify Python installation
echo ""
echo -e "${BLUE}[4/6] Verifying Python installation...${NC}"
PYTHON_VERSION=$(python --version 2>&1 | awk '{print $2}')
echo "Python version: $PYTHON_VERSION"

# Check for Python 3.11+ or install if needed
if [[ $(echo "$PYTHON_VERSION < 3.11" | bc) -eq 1 ]]; then
    echo -e "${YELLOW}Warning: Python 3.11+ recommended."
    if command -v pacman &> /dev/null; then
        echo "    You can install Python 3.11 with: sudo pacman -S python311${NC}"
    fi
fi

# Create virtual environment
echo ""
echo -e "${BLUE}[5/6] Creating Python virtual environment...${NC}"
cd "$(dirname "$0")"

# Determine Python command ( Arch uses 'python' not 'python3')
if command -v python3.11 &> /dev/null; then
    PYTHON_CMD="python3.11"
elif command -v python3.12 &> /dev/null; then
    PYTHON_CMD="python3.12"
elif command -v python3 &> /dev/null; then
    PYTHON_CMD="python3"
else
    PYTHON_CMD="python"
fi

$PYTHON_CMD -m venv venv
source venv/bin/activate

# Upgrade pip
echo ""
echo -e "${BLUE}    Upgrading pip...${NC}"
pip install --upgrade pip --quiet

# Install Python dependencies
echo ""
echo -e "${BLUE}[6/6] Installing QuantumEnergyOS dependencies...${NC}"

# Check if requirements files exist
if [[ -f "requirements-pinned.txt" ]]; then
    echo "    Installing from requirements-pinned.txt..."
    pip install -r requirements-pinned.txt --quiet || pip install -r requirements-pinned.txt
elif [[ -f "requirements.txt" ]]; then
    echo "    Installing from requirements.txt..."
    pip install -r requirements.txt --quiet || pip install -r requirements.txt
else
    echo -e "${YELLOW}    Warning: No requirements file found. Installing basic dependencies...${NC}"
    pip install fastapi uvicorn pydantic python-multipart cryptography numpy scipy
fi

# Install development dependencies if available
if [[ -f "requirements dev.txt" ]]; then
    echo "    Installing development dependencies..."
    pip install -r "requirements dev.txt" --quiet || true
fi

# Create necessary directories
echo ""
echo -e "${BLUE}    Creating necessary directories...${NC}"
mkdir -p logs tmp

# Copy environment example if needed
if [[ -f ".env.example" ]] && [[ ! -f ".env" ]]; then
    cp .env.example .env
    echo -e "${YELLOW}    Note: Created .env file from template. Please configure it.${NC}"
fi

# Verify installation
echo ""
echo -e "${BLUE}╔══════════════════════════════════════════════════════════════╗${NC}"
echo -e "${BLUE}║              Installation Complete!                           ║${NC}"
echo -e "${BLUE}╚══════════════════════════════════════════════════════════════╝${NC}"
echo ""
echo -e "To activate the virtual environment, run:"
echo -e "    ${YELLOW}source venv/bin/activate${NC}"
echo ""
echo -e "To start QuantumEnergyOS, run:"
echo -e "    ${YELLOW}uvicorn api.server:app --reload${NC}"
echo ""
echo -e "Or using the Python module:"
echo -e "    ${YELLOW}python -m api.server${NC}"
echo ""
