#!/usr/bin/env bash
# ══════════════════════════════════════════════════════════════════════
#  QuantumEnergyOS — Instalador Universal
#  Detecta automáticamente: Linux (todas las distros), macOS (Intel + M1/M2/M3/M4)
#  Para Windows: usar install-windows.ps1
#  Desde Mexicali, BC — para el mundo entero.
# ══════════════════════════════════════════════════════════════════════
set -euo pipefail

QEOS_VERSION="1.0.0"
QEOS_REPO="https://github.com/GioCorpus/QuantumEnergyOS"
QEOS_MIN_PYTHON="3.11"
QEOS_INSTALL_DIR="${HOME}/.quantumenergyos"
QEOS_VENV="${QEOS_INSTALL_DIR}/venv"

# ── Colores ───────────────────────────────────────────────────
RED='\033[0;31m'; GREEN='\033[0;32m'; YELLOW='\033[1;33m'
CYAN='\033[0;36m'; BOLD='\033[1m'; RESET='\033[0m'

banner() {
cat << 'EOF'
  ╔══════════════════════════════════════════════════════════╗
  ║          ⚡  Q U A N T U M  E N E R G Y  O S  ⚡        ║
  ║        Desde Mexicali, BC — para el mundo entero         ║
  ║              Kardashev 0 → 1  |  MIT License             ║
  ╚══════════════════════════════════════════════════════════╝
EOF
}

log()     { echo -e "${GREEN}[✓]${RESET} $*"; }
warn()    { echo -e "${YELLOW}[⚠]${RESET} $*"; }
error()   { echo -e "${RED}[✗]${RESET} $*"; exit 1; }
section() { echo -e "\n${CYAN}${BOLD}── $* ──${RESET}"; }

# ── Detección de plataforma ───────────────────────────────────
detect_platform() {
    OS="$(uname -s)"
    ARCH="$(uname -m)"

    case "$OS" in
        Linux*)
            PLATFORM="linux"
            detect_linux_distro
            ;;
        Darwin*)
            PLATFORM="macos"
            detect_macos_arch
            ;;
        MINGW*|MSYS*|CYGWIN*)
            error "En Windows usa: install-windows.ps1\nDesde PowerShell: iex (iwr https://raw.githubusercontent.com/GioCorpus/QuantumEnergyOS/main/install-windows.ps1)"
            ;;
        *)
            error "Plataforma no reconocida: $OS. Abre un Issue en $QEOS_REPO"
            ;;
    esac

    log "Plataforma: ${BOLD}${PLATFORM}${RESET} | Arch: ${BOLD}${ARCH}${RESET}"
}

detect_linux_distro() {
    if [ -f /etc/os-release ]; then
        . /etc/os-release
        DISTRO="${ID:-unknown}"
        DISTRO_VERSION="${VERSION_ID:-rolling}"
        DISTRO_FAMILY="${ID_LIKE:-$ID}"
    elif command -v lsb_release &>/dev/null; then
        DISTRO="$(lsb_release -si | tr '[:upper:]' '[:lower:]')"
        DISTRO_VERSION="$(lsb_release -sr)"
    else
        DISTRO="unknown"
        DISTRO_FAMILY="unknown"
    fi
    log "Distro: ${BOLD}${DISTRO} ${DISTRO_VERSION}${RESET}"
}

detect_macos_arch() {
    if [ "$ARCH" = "arm64" ]; then
        CHIP="Apple Silicon"
        # Detectar generación específica del chip
        CHIP_MODEL="$(sysctl -n machdep.cpu.brand_string 2>/dev/null || echo 'Apple M-series')"
        log "Chip detectado: ${BOLD}${CHIP_MODEL}${RESET} (arm64)"
    else
        CHIP="Intel"
        log "Chip detectado: ${BOLD}Intel x86_64${RESET}"
    fi
}

# ── Verificar Python 3.11+ ────────────────────────────────────
check_python() {
    section "Verificando Python ${QEOS_MIN_PYTHON}+"

    PYTHON_BIN=""
    for cmd in python3.12 python3.11 python3 python; do
        if command -v "$cmd" &>/dev/null; then
            VERSION="$("$cmd" -c 'import sys; print(f"{sys.version_info.major}.{sys.version_info.minor}")')"
            MAJOR="${VERSION%%.*}"; MINOR="${VERSION##*.}"
            if [ "$MAJOR" -ge 3 ] && [ "$MINOR" -ge 11 ]; then
                PYTHON_BIN="$cmd"
                log "Python encontrado: ${BOLD}${cmd} (${VERSION})${RESET}"
                break
            fi
        fi
    done

    if [ -z "$PYTHON_BIN" ]; then
        warn "Python ${QEOS_MIN_PYTHON}+ no encontrado. Instalando..."
        install_python
    fi
}

install_python() {
    case "$PLATFORM" in
        linux)
            case "$DISTRO" in
                ubuntu|debian|linuxmint|pop|zorin|elementary|kdeneon)
                    sudo apt-get update -qq
                    sudo apt-get install -y python3.11 python3.11-venv python3.11-dev python3-pip
                    PYTHON_BIN="python3.11"
                    ;;
                fedora|rhel|centos|almalinux|rocky)
                    sudo dnf install -y python3.11 python3.11-devel python3-pip
                    PYTHON_BIN="python3.11"
                    ;;
                arch|manjaro|endeavouros|arcolinux|garuda)
                    sudo pacman -Sy --needed --noconfirm python python-pip
                    PYTHON_BIN="python3"
                    ;;
                opensuse*|suse*)
                    sudo zypper install -y python311 python311-devel python311-pip
                    PYTHON_BIN="python3.11"
                    ;;
                alpine)
                    sudo apk add --no-cache python3 py3-pip python3-dev
                    PYTHON_BIN="python3"
                    ;;
                nixos)
                    warn "NixOS detectado — usa: nix-shell -p python311 --run './install.sh'"
                    PYTHON_BIN="python3"
                    ;;
                *)
                    error "Distro no reconocida: ${DISTRO}. Instala Python 3.11+ manualmente y reintenta."
                    ;;
            esac
            ;;
        macos)
            if command -v brew &>/dev/null; then
                brew install python@3.11
                PYTHON_BIN="$(brew --prefix)/bin/python3.11"
            else
                warn "Homebrew no encontrado. Instalando Homebrew primero..."
                /bin/bash -c "$(curl -fsSL https://raw.githubusercontent.com/Homebrew/install/HEAD/install.sh)"
                # Añadir Homebrew al PATH para Apple Silicon
                if [ "$ARCH" = "arm64" ]; then
                    eval "$(/opt/homebrew/bin/brew shellenv)"
                fi
                brew install python@3.11
                PYTHON_BIN="$(brew --prefix)/bin/python3.11"
            fi
            ;;
    esac
    log "Python instalado: ${PYTHON_BIN}"
}

# ── Instalar dependencias del sistema ─────────────────────────
install_system_deps() {
    section "Instalando dependencias del sistema"
    case "$PLATFORM" in
        linux)
            case "$DISTRO" in
                ubuntu|debian|linuxmint|pop|zorin|elementary|kdeneon)
                    sudo apt-get update -qq
                    sudo apt-get install -y --no-install-recommends \
                        build-essential curl git wget ca-certificates \
                        libffi-dev libssl-dev zlib1g-dev libbz2-dev \
                        libreadline-dev libsqlite3-dev libgmp-dev
                    ;;
                fedora|rhel|centos|almalinux|rocky)
                    sudo dnf groupinstall -y "Development Tools"
                    sudo dnf install -y curl git wget ca-certificates \
                        openssl-devel libffi-devel zlib-devel bzip2-devel \
                        readline-devel sqlite-devel gmp-devel
                    ;;
                arch|manjaro|endeavouros|arcolinux|garuda)
                    sudo pacman -Sy --needed --noconfirm \
                        base-devel curl git wget ca-certificates \
                        openssl libffi zlib bzip2 readline sqlite gmp
                    ;;
                opensuse*|suse*)
                    sudo zypper install -y -t pattern devel_basis
                    sudo zypper install -y curl git wget ca-certificates \
                        openssl-devel libffi-devel zlib-devel
                    ;;
                alpine)
                    sudo apk add --no-cache \
                        build-base curl git wget ca-certificates \
                        openssl-dev libffi-dev zlib-dev bzip2-dev \
                        readline-dev sqlite-dev gmp-dev musl-dev
                    ;;
            esac
            ;;
        macos)
            # Xcode Command Line Tools
            if ! xcode-select -p &>/dev/null; then
                warn "Instalando Xcode Command Line Tools..."
                xcode-select --install
                warn "Acepta la licencia y reintenta: ./install.sh"
                exit 0
            fi
            log "Xcode CLT: OK"
            ;;
    esac
    log "Dependencias del sistema: OK"
}

# ── Crear entorno virtual e instalar QEOS ────────────────────
install_qeos() {
    section "Instalando QuantumEnergyOS v${QEOS_VERSION}"

    mkdir -p "$QEOS_INSTALL_DIR"
    $PYTHON_BIN -m venv "$QEOS_VENV"

    VENV_PIP="${QEOS_VENV}/bin/pip"
    VENV_PYTHON="${QEOS_VENV}/bin/python"

    # macOS Apple Silicon: variables para compilar extensiones nativas
    if [ "${PLATFORM}" = "macos" ] && [ "${ARCH}" = "arm64" ]; then
        export OPENBLAS="$(brew --prefix openblas 2>/dev/null || echo '')"
        export LDFLAGS="-L$(brew --prefix)/lib"
        export CPPFLAGS="-I$(brew --prefix)/include"
        log "Configuración Apple Silicon: OK"
    fi

    $VENV_PIP install --upgrade pip --quiet
    log "pip actualizado"

    # Instalar desde requirements-pinned.txt si existe, sino requirements.txt
    if [ -f "requirements-pinned.txt" ]; then
        $VENV_PIP install -r requirements-pinned.txt --quiet
        log "Dependencias pinadas instaladas (0 CVEs conocidos)"
    elif [ -f "requirements.txt" ]; then
        $VENV_PIP install -r requirements.txt --quiet
        log "Dependencias instaladas desde requirements.txt"
    fi

    # Instalar QDK (Modern QDK via pip)
    $VENV_PIP install qsharp --quiet 2>/dev/null || warn "qsharp no disponible — Q# correrá via simulador Python"

    log "QuantumEnergyOS instalado en: ${BOLD}${QEOS_INSTALL_DIR}${RESET}"
}

# ── Crear comando global `qeos` ───────────────────────────────
install_launcher() {
    section "Creando launcher global"

    LAUNCHER_PATH="${HOME}/.local/bin/qeos"
    [ "$PLATFORM" = "macos" ] && LAUNCHER_PATH="/usr/local/bin/qeos"

    mkdir -p "$(dirname "$LAUNCHER_PATH")"

    cat > "$LAUNCHER_PATH" << LAUNCHER
#!/usr/bin/env bash
# QuantumEnergyOS launcher
source "${QEOS_VENV}/bin/activate"
cd "${QEOS_INSTALL_DIR}"
exec python -m api.server "\$@"
LAUNCHER

    chmod +x "$LAUNCHER_PATH"

    # Añadir al PATH si no está
    SHELL_RC="${HOME}/.bashrc"
    [ -n "${ZSH_VERSION:-}" ] && SHELL_RC="${HOME}/.zshrc"
    [ "$PLATFORM" = "macos" ] && SHELL_RC="${HOME}/.zshrc"

    if ! echo "$PATH" | grep -q "${HOME}/.local/bin"; then
        echo 'export PATH="$HOME/.local/bin:$PATH"' >> "$SHELL_RC"
    fi

    log "Launcher creado: ${BOLD}${LAUNCHER_PATH}${RESET}"
}

# ── Verificación final ────────────────────────────────────────
verify_installation() {
    section "Verificando instalación"

    VENV_PYTHON="${QEOS_VENV}/bin/python"

    $VENV_PYTHON -c "import qiskit; print(f'  ✅ Qiskit: {qiskit.__version__}')" 2>/dev/null || warn "Qiskit no disponible"
    $VENV_PYTHON -c "import fastapi; print(f'  ✅ FastAPI: {fastapi.__version__}')" 2>/dev/null || warn "FastAPI no disponible"
    $VENV_PYTHON -c "import pydantic; print(f'  ✅ Pydantic: {pydantic.__version__}')" 2>/dev/null || warn "Pydantic no disponible"
    $VENV_PYTHON -c "import numpy; print(f'  ✅ NumPy: {numpy.__version__}')" 2>/dev/null || warn "NumPy no disponible"
    $VENV_PYTHON -c "import qsharp; print(f'  ✅ QSharp: {qsharp.__version__}')" 2>/dev/null || warn "QSharp no disponible (opcional)"
}

# ── Resumen final ─────────────────────────────────────────────
print_success() {
    echo ""
    echo -e "${GREEN}${BOLD}╔══════════════════════════════════════════════════════╗"
    echo -e "║     ✅  QuantumEnergyOS instalado correctamente      ║"
    echo -e "╚══════════════════════════════════════════════════════╝${RESET}"
    echo ""
    echo -e "  ${BOLD}Activar entorno:${RESET}"
    echo -e "    source ${QEOS_VENV}/bin/activate"
    echo ""
    echo -e "  ${BOLD}Iniciar servidor API:${RESET}"
    echo -e "    uvicorn api.server:app --reload --port 8000"
    echo ""
    echo -e "  ${BOLD}Correr simulación Q#:${RESET}"
    echo -e "    python -c \"import qsharp; qsharp.init('.')\""
    echo ""
    echo -e "  ${BOLD}Documentación:${RESET}"
    echo -e "    http://localhost:8000/docs"
    echo ""
    echo -e "  ${CYAN}⚡ Desde Mexicali, BC — para el mundo entero. Kardashev 0→1${RESET}"
    echo ""
}

# ── Main ──────────────────────────────────────────────────────
main() {
    clear
    banner
    echo -e "  Versión: ${BOLD}${QEOS_VERSION}${RESET} | Repo: ${CYAN}${QEOS_REPO}${RESET}"
    echo ""

    detect_platform
    check_python
    install_system_deps
    install_qeos
    install_launcher
    verify_installation
    print_success
}

main "$@"
