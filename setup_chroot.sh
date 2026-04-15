#!/bin/bash
set -e

CHROOT=/home/ubuntu/archroot/root.x86_64

echo "=== Configurando chroot de Arch Linux ==="

# Configurar mirrorlist
bash -c "echo 'Server = https://geo.mirror.pkgbuild.com/\$repo/os/\$arch' > $CHROOT/etc/pacman.d/mirrorlist"
cp /etc/resolv.conf $CHROOT/etc/resolv.conf

# Descomentar DisableSandbox
sed -i '39s/^#//' $CHROOT/etc/pacman.conf 2>/dev/null || true
sed -i '40s/^#//' $CHROOT/etc/pacman.conf 2>/dev/null || true

# Montar sistemas de archivos
mount --bind /proc $CHROOT/proc 2>/dev/null || true
mount --bind /sys $CHROOT/sys 2>/dev/null || true
mount --bind /dev $CHROOT/dev 2>/dev/null || true
mount --bind /dev/pts $CHROOT/dev/pts 2>/dev/null || true
mkdir -p $CHROOT/output
mount --bind /home/ubuntu $CHROOT/output 2>/dev/null || true

echo "=== Mounts configurados ==="
cat /proc/mounts | grep archroot

echo "=== Verificando pacman ==="
chroot $CHROOT /usr/bin/pacman --version 2>&1 | head -3

echo "=== Inicializando keyring ==="
chroot $CHROOT /usr/bin/pacman-key --init 2>&1 | tail -3
chroot $CHROOT /usr/bin/pacman-key --populate archlinux 2>&1 | tail -3

echo "=== Sincronizando bases de datos ==="
chroot $CHROOT /usr/bin/pacman -Sy 2>&1 | tail -5

echo "=== Instalando archiso y dependencias ==="
mkdir -p $CHROOT/output/archcache
chroot $CHROOT /usr/bin/pacman -S --noconfirm --cachedir /output/archcache archiso squashfs-tools dosfstools mtools grub xorriso 2>&1 | tail -20

echo "=== Verificando instalación ==="
ls $CHROOT/usr/bin/mkarchiso && echo "mkarchiso OK" || echo "FALLO"
