#!/usr/bin/env bash
# ═══════════════════════════════════════════════════════════════════════
#  archiso/profiledef.sh — QuantumEnergyOS V.02 Immutable (OSTree)
#  Convertido de mutable Arch Linux → sistema inmutable atómico
# ═══════════════════════════════════════════════════════════════════════

iso_name="QuantumEnergyOS"
iso_label="QEOS_V02_$(date --date="@${SOURCE_DATE_EPOCH:-$(date +%s)}" +%Y%m)"
iso_publisher="Giovanny Corpus Bernal <github.com/GioCorpus>"
iso_application="QuantumEnergyOS V.02 — Immutable Edition — Desde Mexicali"
iso_version="0.2.0-immutable"
install_dir="ostree"
buildmodes=('iso')
bootmodes=('bios.syslinux.mbr' 'bios.syslinux.eltorito'
           'uefi-ia32.grub.esp' 'uefi-x64.grub.esp'
           'uefi-x64.grub.eltorito')
arch="x86_64"
pacman_conf="${profile}/pacman.conf"

# OSTree image configuration
airootfs_image_type="squashfs"
airootfs_image_tool_options=('-comp' 'xz' '-Xbcj' 'x86' '-b' '1M' '-Xdict-size' '1M')
bootstrap_tarball_compression=('zstd' '-c' '-T0' '--auto-threads=logical' '--long' '-19')

file_permissions=(
  ["/etc/shadow"]="0:0:400"
  ["/root"]="0:0:750"
  ["/root/customize_airootfs.sh"]="0:0:755"
  ["/usr/local/bin/qeos"]="0:0:755"
  ["/usr/local/bin/qsh"]="0:0:755"
  ["/usr/local/bin/qeos-immutable-setup"]="0:0:755"
  ["/etc/ostree"]="0:0:755"
)

# ═══════════════════════════════════════════════════════════════════════
# INMUTABLE BASE (core OSTree)
# ═══════════════════════════════════════════════════════════════════════
# Paquetes del sistema base inmutable (no se pueden modificar en runtime)
# Kernel, systemd, ostree, container runtime, herramientas esenciales
#
# Paquetes OSTree base:
#   - ostree: sistema de archivos transactional
#   - linux: kernel estándar
#   - systemd: init system
#   - podman: runtime de contenedores (sin daemon)
#   - flatpak: gestión de aplicaciones
#   - kernel-core, kernel-modules: separated kernel packages
#
# Lo que NO va aquí (va en writable volumes o containers):
#   - Flask API → container/Flatpak
#   - Datos de usuario → /var, /home
#   - Configuración → /etc (overlayrw)
#   - Logs → /var/log

# ═══════════════════════════════════════════════════════════════════════
# VOLUMENES ESCRITURIBLES (Post-instalación)
# ═══════════════════════════════════════════════════════════════════════
# /var → datos variables del sistema (logs, cache, datos de apps)
# /etc → configuración del sistema (overlayrw)
# /home → datos del usuario
# /opt → aplicaciones adicionales
# /root → home del root

# ═══════════════════════════════════════════════════════════════════════
# Puntos de montaje en overlayrw:
#   - /etc → overlayrw en /var/overlayrw/etc
#   - /var → subvolumen independiente
#   - /home → subvolumen independiente

echo "═══════════════════════════════════════════════════════════════════"
echo "  QuantumEnergyOS V.02 — Buildmode Immutable (OSTree)"
echo "═══════════════════════════════════════════════════════════════════"

# Copiar script de setup inmutable
echo "Copiando scripts de configuración inmutable..."
mkdir -p ${airootfs_dir}/usr/local/bin
cp boot/quantum-boot.sh ${airootfs_dir}/usr/local/bin/quantum-boot
chmod +x ${airootfs_dir}/usr/local/bin/quantum-boot

# Copiar script de setup de volúmenes
cp boot/qeos-immutable-setup.sh ${airootfs_dir}/usr/local/bin/qeos-immutable-setup
chmod +x ${airootfs_dir}/usr/local/bin/qeos-immutable-setup

# Copiar servicio systemd para API
mkdir -p ${airootfs_dir}/etc/systemd/system
cp boot/qeos-api.service ${airootfs_dir}/etc/systemd/system/
cp boot/qeos-immutable-setup.service ${airootfs_dir}/etc/systemd/system/

# Habilitar servicios en primer boot
mkdir -p ${airootfs_dir}/etc/systemd/system/multi-user.target.wants
ln -sf /etc/systemd/system/qeos-immutable-setup.service \
       ${airootfs_dir}/etc/systemd/system/multi-user.target.wants/qeos-immutable-setup.service

echo "═══════════════════════════════════════════════════════════════════"
echo "  Immutable setup: qeos-immutable-setup.service"
echo "  API container:   qeos-api.service"
echo "═══════════════════════════════════════════════════════════════════"