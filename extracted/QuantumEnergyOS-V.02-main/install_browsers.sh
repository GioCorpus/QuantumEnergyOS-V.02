#!/usr/bin/env bash
# ═══════════════════════════════════════════════════════════════════════
#  QuantumEnergyOS V.02 — Instalación y Optimización de Navegadores
#  install_browsers.sh
#
#  Instala, configura y optimiza Firefox, Chromium, Brave y Falkon
#  para el entorno cuántico de QuantumEnergyOS en Arch Linux + Wayland.
#
#  Características:
#    - Hardware acceleration completa (VA-API, WebGPU, Vulkan)
#    - Perfiles separados: EnergyGrid, QuantumDev, Secure
#    - Integración con PhotonicQ Bridge y API Flask
#    - Soporte WebSocket/SSE para dashboards en tiempo real
#    - cgroups v2 para gestión de memoria y CPU
#    - Log detallado en /var/log/quantum-browsers-setup.log
#
#  Autor: Giovanny Anthony Corpus Bernal — Mexicali, BC
#  Misión: Nunca más apagones en Mexicali.
# ═══════════════════════════════════════════════════════════════════════

set -euo pipefail

LOG_FILE="/var/log/quantum-browsers-setup.log"
QEOS_DIR="/opt/QuantumEnergyOS"
API_PORT="${QEOS_API_PORT:-8000}"
QEOS_USER="${SUDO_USER:-$(whoami)}"
USER_HOME=$(eval echo "~$QEOS_USER")
PROFILE_DIR="${USER_HOME}/.config/qeos-browsers"

RED='\033[0;31m'; GREEN='\033[0;32m'; YELLOW='\033[1;33m'
CYAN='\033[0;36m'; BOLD='\033[1m'; RESET='\033[0m'

# ── Logger con timestamps ─────────────────────────────────────────────
exec > >(tee -a "$LOG_FILE") 2>&1

log()    { echo -e "$(date '+%Y-%m-%dT%H:%M:%S') ${GREEN}[✓]${RESET} $*"; }
warn()   { echo -e "$(date '+%Y-%m-%dT%H:%M:%S') ${YELLOW}[⚠]${RESET} $*"; }
error()  { echo -e "$(date '+%Y-%m-%dT%H:%M:%S') ${RED}[✗]${RESET} $*"; exit 1; }
section(){ echo -e "\n$(date '+%Y-%m-%dT%H:%M:%S') ${CYAN}${BOLD}── $* ──${RESET}"; }

# ── Verificar root ────────────────────────────────────────────────────
[ "$(id -u)" = "0" ] || error "Ejecutar como root: sudo ./install_browsers.sh"

banner() {
cat << 'BANNER'
  ╔══════════════════════════════════════════════════════════════╗
  ║   ⚡  QuantumEnergyOS V.02 — Browser Setup                  ║
  ║   Firefox · Chromium · Brave · Falkon                        ║
  ║   Hardware Acceleration + cgroups v2 + Perfiles              ║
  ╚══════════════════════════════════════════════════════════════╝
BANNER
}

banner
echo "  Log: $LOG_FILE"
echo "  Usuario: $QEOS_USER"
echo "  API: http://localhost:$API_PORT"
echo ""

# ═══════════════════════════════════════════════════════════════════════
#  1. ACTUALIZAR SISTEMA E INSTALAR NAVEGADORES
# ═══════════════════════════════════════════════════════════════════════

section "Actualizando sistema e instalando navegadores"

pacman -Syu --noconfirm --needed 2>/dev/null

# Navegadores principales
BROWSER_PKGS=(
    "firefox"           # Máxima estabilidad — Perfil Secure
    "chromium"          # WebGPU/WebGL — Perfil EnergyGrid
    "falkon"            # Ligero Qt6 — Modo embedded dashboard
)

pacman -S --noconfirm --needed "${BROWSER_PKGS[@]}" 2>/dev/null

# Brave via AUR (yay/paru)
if command -v yay &>/dev/null; then
    sudo -u "$QEOS_USER" yay -S --noconfirm brave-bin 2>/dev/null && \
        log "Brave instalado via AUR" || warn "Brave no disponible en AUR"
elif command -v paru &>/dev/null; then
    sudo -u "$QEOS_USER" paru -S --noconfirm brave-bin 2>/dev/null && \
        log "Brave instalado via paru" || warn "Brave no disponible"
else
    warn "yay/paru no encontrado — Brave omitido (instalar manualmente: yay -S brave-bin)"
fi

# Dependencias para hardware acceleration
pacman -S --noconfirm --needed \
    mesa \
    vulkan-intel \
    vulkan-radeon \
    libva-mesa-driver \
    libva-intel-driver \
    intel-media-driver \
    lib32-mesa \
    xf86-video-amdgpu \
    gst-plugins-bad \
    gst-plugins-ugly \
    ffmpeg \
    pipewire \
    pipewire-pulse 2>/dev/null || warn "Algunas dependencias de GPU no disponibles"

log "Navegadores y dependencias instalados"

# ═══════════════════════════════════════════════════════════════════════
#  2. CREAR ESTRUCTURA DE PERFILES
# ═══════════════════════════════════════════════════════════════════════

section "Creando perfiles de navegador"

mkdir -p "$PROFILE_DIR"/{energy-grid,quantum-dev,secure}
chown -R "$QEOS_USER:$QEOS_USER" "$PROFILE_DIR"

# ── Perfil EnergyGrid — Firefox para dashboards de red eléctrica ──────
cat > "$PROFILE_DIR/energy-grid/user.js" << USERJS
// ── Firefox EnergyGrid Profile — QuantumEnergyOS V.02 ──
// Optimizado para dashboards de monitoreo energético en tiempo real

// Hardware acceleration
user_pref("gfx.webrender.all", true);
user_pref("gfx.webrender.compositor", true);
user_pref("media.hardware-video-decoding.force-enabled", true);
user_pref("media.ffmpeg.vaapi.enabled", true);
user_pref("layers.acceleration.force-enabled", true);
user_pref("gfx.canvas.azure.accelerated", true);

// WebSocket y SSE para datos en tiempo real
user_pref("network.websocket.enabled", true);
user_pref("network.websocket.max-connections", 200);
user_pref("network.http.max-persistent-connections-per-server", 10);
user_pref("network.http.max-connections", 900);
user_pref("dom.server-sent-events.enabled", true);

// Rendimiento para dashboards
user_pref("javascript.options.ion", true);
user_pref("javascript.options.wasm", true);
user_pref("javascript.options.wasm_simd", true);
user_pref("dom.performance.enable_user_timing_logging", false);
user_pref("browser.cache.memory.enable", true);
user_pref("browser.cache.memory.capacity", 524288);

// Reducir consumo en background
user_pref("dom.timer.throttling.enable", false);
user_pref("dom.min_background_timeout_value", 1);
user_pref("privacy.resistFingerprinting.reduceTimerPrecision.microseconds", 0);

// WebGPU (experimental)
user_pref("dom.webgpu.enabled", true);
user_pref("dom.webgpu.unsafe-powerful-features", true);

// Autostart página de dashboard QEOS
user_pref("browser.startup.homepage", "http://localhost:$API_PORT");
user_pref("browser.startup.page", 1);

// Tema oscuro (desierto de Mexicali)
user_pref("ui.systemUsesDarkTheme", 1);
user_pref("devtools.theme", "dark");
USERJS

# ── Perfil QuantumDev — Chrome/Chromium para Q# y Qiskit ─────────────
cat > "$PROFILE_DIR/quantum-dev/flags.conf" << 'FLAGS'
# Flags Chromium para desarrollo cuántico
--enable-features=VaapiVideoDecoder,VaapiVideoEncoder,WebGPU,VulkanFromANGLE,DefaultANGLEVulkan,UseSkiaRenderer,CanvasOopRasterization,Vulkan
--use-gl=angle
--use-angle=vulkan
--enable-unsafe-webgpu
--ozone-platform=wayland
--enable-zero-copy
--ignore-gpu-blocklist
--enable-gpu-rasterization
--enable-oop-rasterization
--disable-features=UseChromeOSDirectVideoDecoder
--process-per-site
--enable-quic
--disk-cache-size=104857600
--media-cache-size=52428800
FLAGS

# ── Perfil Secure — Brave hardened para uso general ───────────────────
cat > "$PROFILE_DIR/secure/brave-flags.conf" << 'BRAVEFLAGS'
# Brave hardened para QuantumEnergyOS
--ozone-platform=wayland
--enable-features=VaapiVideoDecoder,WebGPU
--enable-zero-copy
--disable-reading-from-canvas
--password-store=gnome-libsecret
BRAVEFLAGS

log "Perfiles creados en $PROFILE_DIR"

# ═══════════════════════════════════════════════════════════════════════
#  3. LAUNCHERS OPTIMIZADOS
# ═══════════════════════════════════════════════════════════════════════

section "Creando launchers optimizados"

# ── /usr/bin/qeos-browser — Launcher principal ────────────────────────
cat > /usr/bin/qeos-browser << LAUNCHER
#!/usr/bin/env bash
# QuantumEnergyOS V.02 — Browser Launcher Principal
# Abre el dashboard cuántico en el perfil más adecuado
#
# Uso: qeos-browser [energy-grid|quantum-dev|secure|<url>]

PROFILE="\${1:-energy-grid}"
URL="http://localhost:$API_PORT"

case "\$PROFILE" in
    energy-grid|grid|e)
        # Firefox con perfil EnergyGrid — dashboards tiempo real
        exec firefox \
            --profile "$PROFILE_DIR/energy-grid" \
            --new-window "\$URL" \
            2>/dev/null &
        ;;
    quantum-dev|dev|q)
        # Chromium con flags de desarrollo cuántico
        CHROMIUM_FLAGS=\$(cat "$PROFILE_DIR/quantum-dev/flags.conf" 2>/dev/null | \
            grep -v '^#' | tr '\n' ' ')
        exec chromium \$CHROMIUM_FLAGS \
            --user-data-dir="$PROFILE_DIR/quantum-dev" \
            "\${2:-\$URL}" \
            2>/dev/null &
        ;;
    secure|s)
        # Brave hardened
        if command -v brave &>/dev/null; then
            BRAVE_FLAGS=\$(cat "$PROFILE_DIR/secure/brave-flags.conf" 2>/dev/null | \
                grep -v '^#' | tr '\n' ' ')
            exec brave \$BRAVE_FLAGS \
                --user-data-dir="$PROFILE_DIR/secure" \
                "\${2:-\$URL}" \
                2>/dev/null &
        else
            exec firefox --private-window "\${2:-\$URL}" 2>/dev/null &
        fi
        ;;
    http*|https*)
        # URL directa — usar EnergyGrid
        exec firefox \
            --profile "$PROFILE_DIR/energy-grid" \
            "\$PROFILE" \
            2>/dev/null &
        ;;
    *)
        echo "Uso: qeos-browser [energy-grid|quantum-dev|secure|<url>]"
        echo ""
        echo "  energy-grid  → Firefox + dashboard energético (localhost:$API_PORT)"
        echo "  quantum-dev  → Chromium + WebGPU + Q#/Qiskit"
        echo "  secure       → Brave hardened"
        echo "  <url>        → Abrir URL en perfil EnergyGrid"
        ;;
esac
LAUNCHER
chmod +x /usr/bin/qeos-browser

# ── /usr/bin/start-energy-browser — Autostart del dashboard ──────────
cat > /usr/bin/start-energy-browser << ENERGYLAUNCHER
#!/usr/bin/env bash
# QuantumEnergyOS V.02 — Autostart Energy Dashboard
# Se ejecuta automáticamente al iniciar Sway/Wayland
# Espera a que la API esté disponible antes de abrir el browser

MAX_WAIT=30
API_URL="http://localhost:$API_PORT/health"

echo "[QEOS] Esperando API en \$API_URL..."
for i in \$(seq 1 \$MAX_WAIT); do
    if curl -sf "\$API_URL" >/dev/null 2>&1; then
        echo "[QEOS] API disponible. Abriendo dashboard cuántico..."
        break
    fi
    sleep 1
done

# Variables de entorno Wayland + VA-API
export WAYLAND_DISPLAY="\${WAYLAND_DISPLAY:-wayland-1}"
export MOZ_ENABLE_WAYLAND=1
export MOZ_DBUS_REMOTE=1
export LIBVA_DRIVER_NAME="\${LIBVA_DRIVER_NAME:-iHD}"

# Abrir Firefox con perfil EnergyGrid
exec firefox \
    --profile "$PROFILE_DIR/energy-grid" \
    --new-window "http://localhost:$API_PORT" \
    --class "qeos-energy-browser" \
    2>/dev/null
ENERGYLAUNCHER
chmod +x /usr/bin/start-energy-browser

log "Launchers creados: qeos-browser, start-energy-browser"

# ═══════════════════════════════════════════════════════════════════════
#  4. VARIABLES DE ENTORNO GLOBALES
# ═══════════════════════════════════════════════════════════════════════

section "Configurando variables de entorno"

cat > /etc/environment.d/50-qeos-browsers.conf << 'ENVVARS'
# QuantumEnergyOS V.02 — Browser Environment Variables
# Activa hardware acceleration en Wayland para todos los navegadores

# Wayland nativo para Firefox
MOZ_ENABLE_WAYLAND=1
MOZ_DBUS_REMOTE=1
MOZ_WEBRENDER=1

# VA-API — hardware video decoding
# Cambiar a 'radeonsi' para AMD, 'nouveau' para Nvidia
LIBVA_DRIVER_NAME=iHD

# Vulkan para Chromium/Brave
VK_ICD_FILENAMES=/usr/share/vulkan/icd.d/intel_icd.x86_64.json:/usr/share/vulkan/icd.d/radeon_icd.x86_64.json

# Chromium/Electron flags via env
CHROMIUM_FLAGS="--enable-features=VaapiVideoDecoder,WebGPU,UseOzonePlatform --ozone-platform=wayland --enable-zero-copy"

# Reducir latencia del renderer
ELECTRON_OZONE_PLATFORM_HINT=wayland
ENVVARS

# Configuración específica de Firefox para VA-API
cat > /etc/firefox/syspref.js << 'SYSPREF' 2>/dev/null || true
// QuantumEnergyOS — Firefox system preferences
pref("gfx.webrender.all", true);
pref("media.ffmpeg.vaapi.enabled", true);
pref("media.hardware-video-decoding.force-enabled", true);
pref("layers.acceleration.force-enabled", true);
SYSPREF

log "Variables de entorno configuradas"

# ═══════════════════════════════════════════════════════════════════════
#  5. SERVICIOS SYSTEMD
# ═══════════════════════════════════════════════════════════════════════

section "Configurando servicios systemd"

# Servicio cgroup manager para navegadores
cat > /etc/systemd/system/qeos-browser-cgroup.service << 'SERVICE'
[Unit]
Description=QuantumEnergyOS Browser cgroup Manager
After=network.target qeos-api.service
Wants=qeos-api.service

[Service]
Type=simple
ExecStart=/usr/bin/python3 /opt/QuantumEnergyOS/browser_cgroup_manager.py --daemon --interval 10 --verbose
Restart=on-failure
RestartSec=5s
StandardOutput=journal
StandardError=journal

[Install]
WantedBy=multi-user.target
SERVICE

# Servicio monitor de métricas de browser
cat > /etc/systemd/system/qeos-browser-monitor.service << 'SERVICE2'
[Unit]
Description=QuantumEnergyOS Browser Monitor (métricas + QAOA)
After=qeos-api.service qeos-browser-cgroup.service
Wants=qeos-api.service

[Service]
Type=simple
WorkingDirectory=/opt/QuantumEnergyOS
ExecStart=/opt/QuantumEnergyOS/venv/bin/python browser_monitor_module.py
Restart=on-failure
RestartSec=3s

[Install]
WantedBy=multi-user.target
SERVICE2

systemctl daemon-reload
systemctl enable qeos-browser-cgroup.service 2>/dev/null || warn "No se pudo habilitar qeos-browser-cgroup"

log "Servicios systemd configurados"

# ═══════════════════════════════════════════════════════════════════════
#  6. EXTENSIONES AUTOMÁTICAS (Firefox)
# ═══════════════════════════════════════════════════════════════════════

section "Configurando extensiones Firefox"

EXTENSIONS_DIR="/usr/lib/firefox/distribution/extensions"
mkdir -p "$EXTENSIONS_DIR"

# Configuración de distribución para instalar extensiones automáticamente
cat > /usr/lib/firefox/distribution/policies.json << 'POLICIES'
{
  "policies": {
    "ExtensionSettings": {
      "uBlock0@raymondhill.net": {
        "installation_mode": "force_installed",
        "install_url": "https://addons.mozilla.org/firefox/downloads/latest/ublock-origin/latest.xpi"
      },
      "{446900e4-71c2-419f-a6a7-df9c091e268b}": {
        "installation_mode": "force_installed",
        "install_url": "https://addons.mozilla.org/firefox/downloads/latest/bitwarden-password-manager/latest.xpi"
      },
      "addon@darkreader.org": {
        "installation_mode": "force_installed",
        "install_url": "https://addons.mozilla.org/firefox/downloads/latest/darkreader/latest.xpi"
      }
    },
    "DisableTelemetry": true,
    "DisableFirefoxStudies": true,
    "OverrideFirstRunPage": "http://localhost:8000",
    "OverridePostUpdatePage": "",
    "Homepage": {
      "URL": "http://localhost:8000",
      "Locked": false,
      "StartPage": "homepage"
    },
    "Preferences": {
      "media.hardware-video-decoding.force-enabled": {
        "Value": true,
        "Status": "default"
      },
      "gfx.webrender.all": {
        "Value": true,
        "Status": "default"
      },
      "dom.webgpu.enabled": {
        "Value": true,
        "Status": "default"
      }
    }
  }
}
POLICIES

log "Políticas Firefox configuradas (uBlock + Bitwarden + Dark Reader)"

# ═══════════════════════════════════════════════════════════════════════
#  7. INTEGRACIÓN CON SWAY (Wayland compositor)
# ═══════════════════════════════════════════════════════════════════════

section "Integrando con compositor Wayland (Sway)"

SWAY_CONFIG_DIR="${USER_HOME}/.config/sway"
SWAY_CONFIG="${SWAY_CONFIG_DIR}/config"

mkdir -p "$SWAY_CONFIG_DIR"

# Añadir configuración de browsers si ya existe config de Sway
if [ -f "$SWAY_CONFIG" ]; then
    # Solo añadir si no está ya
    if ! grep -q "qeos-browser" "$SWAY_CONFIG"; then
        cat >> "$SWAY_CONFIG" << 'SWAY_APPEND'

# ── QuantumEnergyOS — Browser shortcuts ──────────────────────────────
# Ctrl+Shift+E → Dashboard energético (Firefox EnergyGrid)
bindsym Ctrl+Shift+e exec start-energy-browser
# Ctrl+Shift+Q → Chromium QuantumDev
bindsym Ctrl+Shift+q exec qeos-browser quantum-dev
# Ctrl+Shift+B → Brave Secure
bindsym Ctrl+Shift+b exec qeos-browser secure

# Autostart dashboard al iniciar sesión
exec_always --no-startup-id "sleep 6 && start-energy-browser"

# Ventana flotante para el dashboard energético
for_window [class="qeos-energy-browser"] floating enable, resize set 1600 900
SWAY_APPEND
        log "Shortcuts añadidos a Sway config"
    fi
else
    log "Sway config no encontrada — shortcuts no añadidos (configurar manualmente)"
fi

chown -R "$QEOS_USER:$QEOS_USER" "$SWAY_CONFIG_DIR" 2>/dev/null || true

# ═══════════════════════════════════════════════════════════════════════
#  8. VERIFICACIÓN FINAL
# ═══════════════════════════════════════════════════════════════════════

section "Verificación final"

echo ""
echo "  Estado de instalación:"
echo ""

check() {
    if command -v "$1" &>/dev/null; then
        echo -e "  ${GREEN}✓${RESET} $1: $(command -v "$1")"
    else
        echo -e "  ${YELLOW}✗${RESET} $1: no encontrado"
    fi
}

check firefox
check chromium
check brave
check falkon
check qeos-browser
check start-energy-browser

echo ""
echo "  Archivos de configuración:"
[ -f "$PROFILE_DIR/energy-grid/user.js" ] && \
    echo -e "  ${GREEN}✓${RESET} Perfil EnergyGrid: $PROFILE_DIR/energy-grid/user.js" || \
    echo -e "  ${YELLOW}✗${RESET} Perfil EnergyGrid"

[ -f "/usr/lib/firefox/distribution/policies.json" ] && \
    echo -e "  ${GREEN}✓${RESET} Políticas Firefox: /usr/lib/firefox/distribution/policies.json" || \
    echo -e "  ${YELLOW}✗${RESET} Políticas Firefox"

# Verificar hardware acceleration disponible
echo ""
echo "  Hardware acceleration:"
if vainfo &>/dev/null 2>&1; then
    echo -e "  ${GREEN}✓${RESET} VA-API: disponible"
else
    echo -e "  ${YELLOW}⚠${RESET} VA-API: no detectado (normal en VM)"
fi

if vulkaninfo --summary &>/dev/null 2>&1; then
    echo -e "  ${GREEN}✓${RESET} Vulkan: disponible"
else
    echo -e "  ${YELLOW}⚠${RESET} Vulkan: no detectado (normal en VM)"
fi

# ═══════════════════════════════════════════════════════════════════════
#  RESUMEN FINAL
# ═══════════════════════════════════════════════════════════════════════

echo ""
echo -e "  ${GREEN}${BOLD}╔══════════════════════════════════════════════════════════╗${RESET}"
echo -e "  ${GREEN}${BOLD}║    ✅  QuantumEnergyOS Browsers — Setup Completado       ║${RESET}"
echo -e "  ${GREEN}${BOLD}╚══════════════════════════════════════════════════════════╝${RESET}"
echo ""
echo "  Perfiles disponibles:"
echo -e "  ${CYAN}  qeos-browser energy-grid${RESET}  → Firefox + Dashboard energético"
echo -e "  ${CYAN}  qeos-browser quantum-dev${RESET}   → Chromium + WebGPU + Q#/Qiskit"
echo -e "  ${CYAN}  qeos-browser secure${RESET}        → Brave hardened"
echo ""
echo "  Shortcuts Sway (si configurado):"
echo "    Ctrl+Shift+E → Dashboard energético"
echo "    Ctrl+Shift+Q → QuantumDev"
echo "    Ctrl+Shift+B → Secure"
echo ""
echo "  Log completo: $LOG_FILE"
echo ""
echo -e "  ${CYAN}⚡ Desde Mexicali, BC — Nunca más apagones. Kardashev 0→1${RESET}"
echo ""
