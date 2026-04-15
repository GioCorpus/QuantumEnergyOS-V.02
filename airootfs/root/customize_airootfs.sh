#!/bin/bash
# customize_airootfs.sh

echo "Configurando GRUB para QuantumEnergyOS V.02..."

# Instalar GRUB
pacman -S --noconfirm grub efibootmgr

# Configurar GRUB para UEFI
grub-install --target=x86_64-efi --efi-directory=/boot --bootloader-id=QuantumEnergyOS --recheck
grub-mkconfig -o /boot/grub/grub.cfg

# También para BIOS (Legacy)
grub-install --target=i386-pc --recheck /dev/sda   # Cambiar /dev/sda según sea necesario en live

echo "GRUB configurado correctamente."

chmod +x airootfs/root/customize_airootfs.sh

sudo mkarchiso -v -w ./work -o ./out .

make build-iso
