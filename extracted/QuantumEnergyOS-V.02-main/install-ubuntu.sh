#!/bin/bash
# ──────────────────────────────────────────────────────────────────────────────
# install-ubuntu.sh — QuantumEnergyOS Installation Script for Ubuntu
# and Ubuntu-based distributions (Linux Mint, Pop!_OS, Zorin OS, etc.)
#
# Usage: chmod +x install-ubuntu.sh && ./install-ubuntu.sh
# ──────────────────────────────────────────────────────────────────────────────

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

echo -e "${GREEN}╔══════════════════════════════════════════════════════════════╗${NC}"
echo -e "${GREEN}║     QuantumEnergyOS - Ubuntu Installation Script             ║${NC}"
echo -e "${GREEN}╚══════════════════════════════════════════════════════════════╝${NC}"
echo ""

# Check if running as root or with sudo
if [[ $EUID -ne 0 ]]; then
    echo -e "${YELLOW}Note: This script uses sudo for package installation.${NC}"
    echo -e "${YELLOW}You may be prompted for your password.${NC}"
    echo ""
fi

# Detect Ubuntu version
echo -e "${GREEN}[1/6] Detecting Ubuntu version...${NC}"
if [[ -f /etc/lsb-release ]]; then
    source /etc/lsb-release
    echo -e "Found: ${DISTRIB_DESCRIPTION} (${DISTRIB_RELEASE})"
else
    echo -e "${YELLOW}Warning: Could not detect Ubuntu version. Continuing anyway...${NC}"
fi

# Update package lists
echo ""
echo -e "${GREEN}[2/6] Updating package lists...${NC}"
sudo apt update -qq

# Install system dependencies
echo ""
echo -e "${GREEN}[3/6] Installing system dependencies...${NC}"
sudo apt install -y \
    python3 \
    python3-pip \
    python3-venv \
    python3-dev \
    build-essential \
    libffi-dev \
    libssl-dev \
    zlib1g-dev \
    libbz2-dev \
    libreadline-dev \
    libsqlite3-dev \
    wget \
    curl \
    git \
    ca-certificates \
    libpq-dev \
    libxml2-dev \
    libxslt1-dev \
    zlib1g-dev \
    libopenblas-dev \
    liblapack-dev \
    gfortran \
    pkg-config

# Verify Python installation
echo ""
echo -e "${GREEN}[4/6] Verifying Python installation...${NC}"
PYTHON_VERSION=$(python3 --version 2>&1 | awk '{print $2}')
echo "Python version: $PYTHON_VERSION"

if [[ $(echo "$PYTHON_VERSION < 3.11" | bc) -eq 1 ]]; then
    echo -e "${YELLOW}Warning: Python 3.11+ recommended. Installing python3.11...${NC}"
    sudo apt install -y python3.11 python3.11-venv python3.11-dev
fi

# Create virtual environment
echo ""
echo -e "${GREEN}[5/6] Creating Python virtual environment...${NC}"
cd "$(dirname "$0")"

# Use Python 3.11 if available, otherwise use system python3
if command -v python3.11 &> /dev/null; then
    PYTHON_CMD="python3.11"
elif command -v python3.12 &> /dev/null; then
    PYTHON_CMD="python3.12"
else
    PYTHON_CMD="python3"
fi

$PYTHON_CMD -m venv venv
source venv/bin/activate

# Upgrade pip
echo ""
echo -e "${GREEN}    Upgrading pip...${NC}"
pip install --upgrade pip --quiet

# Install Python dependencies
echo ""
echo -e "${GREEN}[6/6] Installing QuantumEnergyOS dependencies...${NC}"

# Check if requirements files exist
if [[ -f "requirements-pinned.txt" ]]; then
    echo "    Installing from requirements-pinned.txt..."
    pip install -r requirements-pinned.txt --quiet || pip install -r requirements-pinned.txt
elif [[ -f "requirements.txt" ]]; then
    echo "    Installing from requirements.txt..."
    pip install -r requirements.txt --quiet || pip install -r requirements.txt
else
    echo -e "${YELLOW}    Warning: No requirements file found. Installing basic dependencies...${NC}"
    pip install fastapi uvicorn pydantic python-multipart cryptographynumpy scipy
fi

# Install development dependencies if available
if [[ -f "requirements dev.txt" ]]; then
    echo "    Installing development dependencies..."
    pip install -r "requirements dev.txt" --quiet || true
fi

# Create necessary directories
echo ""
echo -e "${GREEN}    Creating necessary directories...${NC}"
mkdir -p logs tmp

# Copy environment example if needed
if [[ -f ".env.example" ]] && [[ ! -f ".env" ]]; then
    cp .env.example .env
    echo -e "${YELLOW}    Note: Created .env file from template. Please configure it.${NC}"
fi

# Verify installation
echo ""
echo -e "${GREEN}╔══════════════════════════════════════════════════════════════╗${NC}"
echo -e "${GREEN}║              Installation Complete!                           ║${NC}"
echo -e "${GREEN}╚══════════════════════════════════════════════════════════════╝${NC}"
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
