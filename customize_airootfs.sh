#!/usr/bin/env bash
# ═══════════════════════════════════════════════════════════════════════
#  customize_airootfs.sh — QuantumEnergyOS V.02 Immutable (OSTree)
#  Se ejecuta dentro del chroot durante la construcción de la ISO
# ═══════════════════════════════════════════════════════════════════════
set -e

echo "⚡ [QEOS] Iniciando personalización del sistema inmutable..."

# ── 1. Configurar locale y timezone ──────────────────────────────────
echo "LANG=es_MX.UTF-8" > /etc/locale.conf
echo "es_MX.UTF-8 UTF-8" >> /etc/locale.gen
echo "en_US.UTF-8 UTF-8" >> /etc/locale.gen
locale-gen
ln -sf /usr/share/zoneinfo/America/Tijuana /etc/localtime
echo "QuantumEnergyOS" > /etc/hostname

# ── 2. Configurar usuario quantum y qeos ──────────────────────────────
useradd -m -G wheel,audio,video,network -s /bin/bash quantum 2>/dev/null || true
useradd -r -s /sbin/nologin -d /var/lib/qeos qeos 2>/dev/null || true
echo "quantum:quantum" | chpasswd
echo "root:quantum" | chpasswd
echo "%wheel ALL=(ALL) NOPASSWD: ALL" >> /etc/sudoers

# ── 3. Inicializar OSTree repo (para commits atómicos) ────────────────
echo "[QEOS] Configurando OSTree..."
mkdir -p /ostree/repo
ostree init --repo=/ostree/repo --mode=bare-user-only

# ── 4. Compilar Rust photonic-core y photonic-bridge ─────────────────
if [ -d "/opt/QuantumEnergyOS/photonic-core" ]; then
    echo "[QEOS] Compilando photonic-core..."
    cd /opt/QuantumEnergyOS/photonic-core
    cargo build --release 2>/dev/null || echo "[QEOS] Warning: Falló compilación de photonic-core"
fi

if [ -d "/opt/QuantumEnergyOS/photonic-bridge" ]; then
    echo "[QEOS] Compilando photonic-bridge..."
    cd /opt/QuantumEnergyOS/photonic-bridge
    cargo build --release 2>/dev/null || echo "[QEOS] Warning: Falló compilación de photonic-bridge"
fi

# ── 5. Directorios para volúmenes escribibles (post-instalación) ───────
mkdir -p /var/lib/qeos
mkdir -p /var/log/qeos
mkdir -p /etc/qeos
mkdir -p /var/overlayrw
mkdir -p /srv/qeos

# ── 6. Configurar OSTree para boot ────────────────────────────────────
mkdir -p /boot/ostree
cat > /etc/ostree.boot.conf << 'OSTREE_BOOT'
[Boot]
Default=deploy/qeos/stable
[Deployments]
stable.path=/ostree/deploy/qeos/stable
OSTREE_BOOT

# ── 7. Comando global qeos (para Immutable OS) ────────────────────────
cat > /usr/local/bin/qeos << 'QEOS_BIN'
#!/usr/bin/env bash
# QuantumEnergyOS launcher (Immutable Edition)
case "${1:-}" in
    start)
        echo "⚡ Iniciando QuantumEnergyOS API Container..."
        systemctl start qeos-api.service
        ;;
    stop)
        systemctl stop qeos-api.service
        ;;
    status)
        systemctl status qeos-api.service --no-pager
        ;;
    rollback)
        echo "⚠️  Ejecutando rollback..."
        ostree admin rollback
        systemctl reboot
        ;;
    update)
        echo "⚡ Aplicando actualización atómica..."
        ostree admin upgrade
        ;;
    grid)
        podman exec -it qeos-api python3 -c "
from api.core import simular_grid
import json
r = simular_grid(6, 1024, 0.5, 0.3)
print(json.dumps(r, indent=2))
" 2>/dev/null || echo "Contenedor no disponible. Ejecutar: qeos start"
        ;;
    fusion)
        podman exec -it qeos-api python3 -c "
from api.core import simular_fusion
import json
r = simular_fusion(65.0, 1.0, 1.0, 4)
print(json.dumps(r, indent=2))
" 2>/dev/null || echo "Contenedor no disponible. Ejecutar: qeos start"
        ;;
    version)
        ostree admin status
        ;;
    *)
        echo "Uso: qeos [start|stop|status|rollback|update|grid|fusion|version]"
        echo ""
        echo "Comandos específicos de Immutable OS:"
        echo "  qeos update   — Aplicar actualización atómica"
        echo "  qeos rollback — Revertir al deployment anterior"
        ;;
esac
QEOS_BIN
chmod +x /usr/local/bin/qeos

# ── 8. Quantum Shell shortcut ─────────────────────────────────────────
cat > /usr/local/bin/qsh << 'QSH_BIN'
#!/usr/bin/env bash
# Quantum Shell — REPL interactivo de QuantumEnergyOS
echo "⚡ QuantumEnergyOS Quantum Shell (Container)"
echo "   Comandos: grid(), fusion(), braiding(), cooling()"
podman exec -it qeos-api python3 -i -c "
try:
    from api.core import simular_grid as grid, simular_fusion as fusion
    print('Módulos cargados. Prueba: grid(4, 1024, 0.5, 0.3)')
except ImportError:
    print('Módulos no disponibles. Iniciar con: qeos start')
" 2>/dev/null || echo "Contenedor no disponible. Ejecutar: qeos start"
QSH_BIN
chmod +x /usr/local/bin/qsh

# ── 9. Configurar Sway (Wayland compositor) ───────────────────────────
mkdir -p /etc/skel/.config/sway
cat > /etc/skel/.config/sway/config << 'SWAY_CONF'
set $mod Mod4
output * background #0a0a1a solid_color
set $quantum_blue   #00ffff
set $desert_orange  #ff6b00
set $deep_black     #0a0a1a
bindsym $mod+Return exec foot
bindsym $mod+d      exec wofi --show run
bindsym $mod+q      kill
bindsym $mod+Shift+e exec swaynag -t warning -m '¿Cerrar sesión?' -B 'Sí' 'swaymsg exit'
bindsym $mod+h focus left
bindsym $mod+j focus down
bindsym $mod+k focus up
bindsym $mod+l focus right
bindsym $mod+f fullscreen toggle
bindsym Print exec grim ~/screenshot-$(date +%Y%m%d-%H%M%S).png
exec_always qeos start
exec_always bash -c "sleep 5 && foot -e bash -c 'echo ⚡ QuantumEnergyOS V.02 Immutable listo; bash'"
default_border pixel 2
gaps inner 8
gaps outer 4
client.focused          $quantum_blue $deep_black $quantum_blue $quantum_blue
client.unfocused        #333333 $deep_black #888888 #333333
client.urgent           $desert_orange $deep_black $desert_orange $desert_orange
bar {
    position top
    status_command while date +'⚡ QEOS Immutable %H:%M:%S  |  Kardashev 0→1  |  Mexicali'; do sleep 1; done
    colors {
        background $deep_black
        statusline $quantum_blue
        focused_workspace $quantum_blue $deep_black $quantum_blue
        inactive_workspace #333333 $deep_black #888888
    }
}
SWAY_CONF
mkdir -p /home/quantum/.config/sway
cp /etc/skel/.config/sway/config /home/quantum/.config/sway/config
chown -R quantum:quantum /home/quantum/.config

# ── 10. Habilitar servicios ────────────────────────────────────────────
systemctl enable NetworkManager
systemctl enable sshd

# ── 11. Autologin en TTY1 ───────────────────────────────────────────────
mkdir -p /etc/systemd/system/getty@tty1.service.d
cat > /etc/systemd/system/getty@tty1.service.d/autologin.conf << 'AUTOLOGIN'
[Service]
ExecStart=
ExecStart=-/sbin/agetty --autologin quantum --noclear %I $TERM
AUTOLOGIN

cat >> /home/quantum/.bash_profile << 'BASHPROFILE'
if [ -z "${WAYLAND_DISPLAY}" ] && [ "${XDG_VTNR}" -eq 1 ]; then
    exec sway
fi
BASHPROFILE
chown quantum:quantum /home/quantum/.bash_profile

# ── 12. MOTD ───────────────────────────────────────────────────────────
cat > /etc/motd << 'MOTD'

  ╔═══════════════════════════════════════════════════════════╗
  ║   ⚡  QuantumEnergyOS V.02  —  Immutable OSTree Edition  ║
  ║   Desde Mexicali, BC — para el mundo entero               ║
  ║   Kardashev 0→1                                           ║
  ╠═══════════════════════════════════════════════════════════╣
  ║  Comandos:                                                ║
  ║    qeos start    — Iniciar API (http://localhost:8000)    ║
  ║    qeos update   — Actualización atómica                  ║
  ║    qeos rollback — Rollback al deployment anterior        ║
  ║    qeos status   — Ver estado del sistema                 ║
  ╚═══════════════════════════════════════════════════════════╝

MOTD

echo ""
echo "✅ [QEOS] Personalización completada - Immutable OSTree Edition"
echo "   API ejecutándose en: Podman container"
echo "   Actualizaciones: ostree admin upgrade"
echo "   Rollback: qeos rollback"