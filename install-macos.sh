#!/usr/bin/env bash
# ══════════════════════════════════════════════════════════════════════
#  QuantumEnergyOS — Instalador macOS
#  Soporta: Apple Silicon M1, M2, M3, M4 (arm64) + Intel (x86_64)
#  macOS 12 Monterey, 13 Ventura, 14 Sonoma, 15 Sequoia
# ══════════════════════════════════════════════════════════════════════
set -euo pipefail

QEOS_VERSION="1.0.0"
QEOS_REPO="https://github.com/GioCorpus/QuantumEnergyOS"
INSTALL_DIR="${HOME}/Library/Application Support/QuantumEnergyOS"
VENV_DIR="${INSTALL_DIR}/venv"
LOG_FILE="${INSTALL_DIR}/install.log"

RED='\033[0;31m'; GREEN='\033[0;32m'; YELLOW='\033[1;33m'
CYAN='\033[0;36m'; BOLD='\033[1m'; RESET='\033[0m'

log()     { echo -e "${GREEN}[✓]${RESET} $*" | tee -a "$LOG_FILE" 2>/dev/null || echo -e "${GREEN}[✓]${RESET} $*"; }
warn()    { echo -e "${YELLOW}[⚠]${RESET} $*"; }
error()   { echo -e "${RED}[✗]${RESET} $*"; exit 1; }
section() { echo -e "\n${CYAN}${BOLD}── $* ──${RESET}"; }

# ── Detectar chip Apple ───────────────────────────────────────
detect_apple_chip() {
    ARCH="$(uname -m)"

    if [ "$ARCH" = "arm64" ]; then
        IS_APPLE_SILICON=true
        BREW_PREFIX="/opt/homebrew"

        # Detectar generación del chip
        CHIP_INFO="$(sysctl -n machdep.cpu.brand_string 2>/dev/null || echo 'Apple M-series')"

        # Detectar M1/M2/M3/M4
        if system_profiler SPHardwareDataType 2>/dev/null | grep -q "M4"; then
            CHIP_GEN="M4"; log "Chip: ${BOLD}Apple M4${RESET} — máximo rendimiento cuántico"
        elif system_profiler SPHardwareDataType 2>/dev/null | grep -q "M3"; then
            CHIP_GEN="M3"; log "Chip: ${BOLD}Apple M3${RESET}"
        elif system_profiler SPHardwareDataType 2>/dev/null | grep -q "M2"; then
            CHIP_GEN="M2"; log "Chip: ${BOLD}Apple M2${RESET}"
        else
            CHIP_GEN="M1"; log "Chip: ${BOLD}Apple M1${RESET}"
        fi

        # Configurar variables de entorno para Apple Silicon
        export HOMEBREW_PREFIX="$BREW_PREFIX"
        export PATH="$BREW_PREFIX/bin:$BREW_PREFIX/sbin:$PATH"
        eval "$($BREW_PREFIX/bin/brew shellenv 2>/dev/null || true)"

    else
        IS_APPLE_SILICON=false
        BREW_PREFIX="/usr/local"
        CHIP_GEN="Intel"
        log "Chip: ${BOLD}Intel x86_64${RESET}"
    fi

    # Versión de macOS
    MACOS_VER="$(sw_vers -productVersion)"
    MACOS_NAME="$(sw_vers -productName)"
    log "macOS: ${BOLD}${MACOS_NAME} ${MACOS_VER}${RESET}"
}

# ── Xcode Command Line Tools ──────────────────────────────────
ensure_xcode_clt() {
    section "Xcode Command Line Tools"
    if xcode-select -p &>/dev/null 2>&1; then
        log "Xcode CLT: $(xcode-select -p)"
    else
        warn "Instalando Xcode Command Line Tools..."
        xcode-select --install 2>/dev/null || true
        warn "Una ventana de instalación debería aparecer."
        warn "Después de completar, ejecuta este script nuevamente."
        read -r -p "¿Ya terminó la instalación de Xcode CLT? (s/n): " resp
        [[ "$resp" =~ ^[Ss]$ ]] || exit 0
    fi
}

# ── Homebrew ──────────────────────────────────────────────────
ensure_homebrew() {
    section "Homebrew"
    if command -v brew &>/dev/null; then
        log "Homebrew: $(brew --version | head -1)"
        brew update --quiet 2>/dev/null || true
    else
        warn "Instalando Homebrew..."
        /bin/bash -c "$(curl -fsSL https://raw.githubusercontent.com/Homebrew/install/HEAD/install.sh)"

        if $IS_APPLE_SILICON; then
            echo 'eval "$(/opt/homebrew/bin/brew shellenv)"' >> "${HOME}/.zprofile"
            eval "$(/opt/homebrew/bin/brew shellenv)"
        fi
        log "Homebrew instalado"
    fi
}

# ── Python 3.11+ ──────────────────────────────────────────────
ensure_python() {
    section "Python 3.11+"
    PYTHON_BIN=""

    for cmd in python3.12 python3.11 python3; do
        if command -v "$cmd" &>/dev/null; then
            VER="$($cmd -c 'import sys; print(f"{sys.version_info.major}.{sys.version_info.minor}")')"
            MAJOR="${VER%%.*}"; MINOR="${VER##*.}"
            if [ "$MAJOR" -ge 3 ] && [ "$MINOR" -ge 11 ]; then
                PYTHON_BIN="$cmd"
                log "Python: $cmd ($VER)"
                break
            fi
        fi
    done

    if [ -z "$PYTHON_BIN" ]; then
        warn "Instalando Python 3.11 via Homebrew..."
        brew install python@3.11
        PYTHON_BIN="${BREW_PREFIX}/bin/python3.11"

        # Configurar en shell
        SHELL_RC="${HOME}/.zshrc"
        echo "export PATH=\"${BREW_PREFIX}/opt/python@3.11/bin:\$PATH\"" >> "$SHELL_RC"
        export PATH="${BREW_PREFIX}/opt/python@3.11/bin:$PATH"
        log "Python 3.11 instalado"
    fi
}

# ── Dependencias específicas Apple Silicon ────────────────────
install_apple_silicon_deps() {
    if $IS_APPLE_SILICON; then
        section "Dependencias nativas Apple Silicon (${CHIP_GEN})"

        # OpenBLAS optimizado para Apple Silicon
        brew install openblas --quiet 2>/dev/null || true

        # Variables para compilar extensiones NumPy/SciPy nativas arm64
        export OPENBLAS="$(brew --prefix openblas)"
        export LAPACK="$(brew --prefix openblas)"
        export LDFLAGS="-L${BREW_PREFIX}/lib -L${OPENBLAS}/lib"
        export CPPFLAGS="-I${BREW_PREFIX}/include -I${OPENBLAS}/include"
        export PKG_CONFIG_PATH="${BREW_PREFIX}/lib/pkgconfig:${OPENBLAS}/lib/pkgconfig"

        # .NET para Q# si disponible
        if ! command -v dotnet &>/dev/null; then
            warn "Instalando .NET 8 (opcional, para QDK avanzado)..."
            brew install --cask dotnet-sdk --quiet 2>/dev/null || true
        fi

        log "Dependencias Apple Silicon: OK"
    fi
}

# ── Instalar QEOS ─────────────────────────────────────────────
install_qeos() {
    section "Instalando QuantumEnergyOS v${QEOS_VERSION}"

    mkdir -p "$INSTALL_DIR"
    touch "$LOG_FILE"

    # Clonar repo
    if [ ! -d "${INSTALL_DIR}/.git" ]; then
        git clone "$QEOS_REPO" "$INSTALL_DIR" --depth 1 2>/dev/null || {
            warn "No se pudo clonar — copiando archivos locales..."
            cp -r . "$INSTALL_DIR/"
        }
    else
        cd "$INSTALL_DIR" && git pull origin main --quiet
    fi

    cd "$INSTALL_DIR"

    # Crear venv
    $PYTHON_BIN -m venv "$VENV_DIR"

    VENV_PIP="${VENV_DIR}/bin/pip"
    VENV_PYTHON="${VENV_DIR}/bin/python"

    $VENV_PIP install --upgrade pip --quiet

    # En Apple Silicon: forzar wheels nativos arm64
    if $IS_APPLE_SILICON; then
        PIP_FLAGS="--no-binary :none: --prefer-binary"
    else
        PIP_FLAGS=""
    fi

    if [ -f "requirements-pinned.txt" ]; then
        $VENV_PIP install $PIP_FLAGS -r requirements-pinned.txt --quiet || \
        $VENV_PIP install -r requirements-pinned.txt --quiet
        log "Dependencias pinadas instaladas"
    elif [ -f "requirements.txt" ]; then
        $VENV_PIP install -r requirements.txt --quiet
    fi

    $VENV_PIP install qsharp --quiet 2>/dev/null && log "QDK instalado" || warn "QDK: opcional"
    log "QuantumEnergyOS instalado en: $INSTALL_DIR"
}

# ── App Bundle macOS ──────────────────────────────────────────
create_app_bundle() {
    section "Creando app bundle (.app)"

    APP_DIR="${HOME}/Applications/QuantumEnergyOS.app"
    mkdir -p "${APP_DIR}/Contents/MacOS"
    mkdir -p "${APP_DIR}/Contents/Resources"

    # Script ejecutable
    cat > "${APP_DIR}/Contents/MacOS/QuantumEnergyOS" << APPSCRIPT
#!/bin/bash
source "${VENV_DIR}/bin/activate"
cd "${INSTALL_DIR}"
open http://localhost:8000
uvicorn api.server:app --host 0.0.0.0 --port 8000
APPSCRIPT
    chmod +x "${APP_DIR}/Contents/MacOS/QuantumEnergyOS"

    # Info.plist
    cat > "${APP_DIR}/Contents/Info.plist" << PLIST
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
    <key>CFBundleName</key>
    <string>QuantumEnergyOS</string>
    <key>CFBundleDisplayName</key>
    <string>Quantum Energy OS</string>
    <key>CFBundleIdentifier</key>
    <string>com.giocorpus.quantumenergyos</string>
    <key>CFBundleVersion</key>
    <string>${QEOS_VERSION}</string>
    <key>CFBundleExecutable</key>
    <string>QuantumEnergyOS</string>
    <key>CFBundlePackageType</key>
    <string>APPL</string>
    <key>LSMinimumSystemVersion</key>
    <string>12.0</string>
    <key>NSHighResolutionCapable</key>
    <true/>
    <key>NSRequiresAquaSystemAppearance</key>
    <false/>
</dict>
</plist>
PLIST

    log "App bundle: ${APP_DIR}"

    # Comando qeos en /usr/local/bin
    cat > /usr/local/bin/qeos << QEOSCMD
#!/bin/bash
source "${VENV_DIR}/bin/activate"
cd "${INSTALL_DIR}"
exec "\$@"
QEOSCMD
    chmod +x /usr/local/bin/qeos 2>/dev/null || \
        sudo chmod +x /usr/local/bin/qeos 2>/dev/null || \
        warn "No se pudo crear /usr/local/bin/qeos — usa el app bundle directamente"
}

# ── .zshrc / .bash_profile ────────────────────────────────────
configure_shell() {
    section "Configurando shell"
    SHELL_RC="${HOME}/.zshrc"
    [ -f "${HOME}/.bashrc" ] && [ -z "${ZSH_VERSION:-}" ] && SHELL_RC="${HOME}/.bashrc"

    ALIAS_LINE="alias qeos='source ${VENV_DIR}/bin/activate && cd ${INSTALL_DIR}'"
    grep -qxF "$ALIAS_LINE" "$SHELL_RC" 2>/dev/null || echo "$ALIAS_LINE" >> "$SHELL_RC"
    log "Alias 'qeos' agregado a $SHELL_RC"
}

# ── Main ──────────────────────────────────────────────────────
main() {
    clear
    echo ""
    echo -e "${CYAN}${BOLD}  ╔══════════════════════════════════════════════╗"
    echo -e "  ║   ⚡  QuantumEnergyOS — Instalador macOS   ║"
    echo -e "  ║   M1 · M2 · M3 · M4 · Intel  |  MIT      ║"
    echo -e "  ╚══════════════════════════════════════════════╝${RESET}"
    echo ""

    detect_apple_chip
    ensure_xcode_clt
    ensure_homebrew
    ensure_python
    install_apple_silicon_deps
    install_qeos
    create_app_bundle
    configure_shell

    echo ""
    echo -e "${GREEN}${BOLD}  ✅  QuantumEnergyOS instalado para macOS (${CHIP_GEN})${RESET}"
    echo ""
    echo -e "  Iniciar:   ${YELLOW}open ~/Applications/QuantumEnergyOS.app${RESET}"
    echo -e "  o bien:    ${YELLOW}source ${VENV_DIR}/bin/activate && uvicorn api.server:app --port 8000${RESET}"
    echo -e "  API docs:  ${YELLOW}http://localhost:8000/docs${RESET}"
    echo ""
    echo -e "  ${CYAN}⚡ Desde Mexicali, BC — para el mundo entero. Kardashev 0→1${RESET}"
    echo ""
}

main "$@"
