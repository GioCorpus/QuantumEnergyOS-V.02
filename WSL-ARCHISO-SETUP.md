# QuantumEnergyOS — WSL + Arch Linux + archiso Setup

## 📋 Descripción

Este documento explica cómo configurar **WSL (Windows Subsystem for Linux)** con **Arch Linux** para usar **archiso** y construir una ISO bootable de QuantumEnergyOS directamente desde Windows.

## 🎯 ¿Por qué WSL + archiso?

- **archiso** es la herramienta oficial de Arch Linux para crear ISOs
- Solo funciona en sistemas Linux
- WSL permite ejecutar Linux nativamente en Windows
- Combina lo mejor de ambos mundos: Windows como host, Linux para builds

## 🚀 Instalación Paso a Paso

### Paso 1: Instalar WSL

Abre **PowerShell como Administrador**:

```powershell
# Instalar WSL con Ubuntu por defecto
wsl --install

# Reiniciar tu computadora
Restart-Computer
```

Después del reinicio, WSL se iniciará automáticamente y te pedirá:
- Crear un nombre de usuario
- Establecer una contraseña

### Paso 2: Instalar Arch Linux en WSL

```powershell
# Listar distribuciones disponibles
wsl --list --online

# Instalar Arch Linux
wsl --install -d ArchLinux

# O si no está en la lista, usar método manual:
# 1. Descargar Arch Linux WSL desde: https://github.com/yuk7/ArchWSL
# 2. Extraer y ejecutar Arch.exe
```

### Paso 3: Configurar Arch Linux

Abre una terminal de Arch Linux en WSL:

```bash
# Actualizar el sistema
sudo pacman -Syu

# Instalar dependencias esenciales
sudo pacman -S base-devel git wget curl

# Instalar archiso
sudo pacman -S archiso

# Instalar herramientas adicionales
sudo pacman -S squashfs-tools libisoburn dosfstools
```

### Paso 4: Clonar QuantumEnergyOS

```bash
# Crear directorio de trabajo
mkdir -p ~/quantumenergyos
cd ~/quantumenergyos

# Clonar el repositorio (o copiar desde Windows)
git clone https://github.com/GioCorpus/QuantumEnergyOS.git .

# O copiar desde Windows (si ya lo tienes)
# cp -r /mnt/c/Users/HP/Documents/"Documentos Personales GACB"/Demo/QuantumEnergyOS/QuantumEnergyOS-main/* .
```

### Paso 5: Configurar el Perfil de archiso

```bash
# Crear directorio para el build
mkdir -p ~/quantum-build
cd ~/quantum-build

# Copiar el perfil base de archiso
cp -r /usr/share/archiso/configs/releng/* .

# O crear un perfil personalizado para QuantumEnergyOS
mkdir -p quantumenergyos
cd quantumenergyos
```

### Paso 6: Personalizar el Perfil

#### 6.1: Crear packages.x86_64

```bash
cat > packages.x86_64 << 'EOF'
# Base del sistema
base
linux
linux-firmware
mkinitcpio
mkinitcpio-archiso
grub
efibootmgr
syslinux

# Sistema de archivos
dosfstools
e2fsprogs
btrfs-progs

# Red
networkmanager
wireless-tools
wpa_supplicant
netctl

# Python y dependencias
python
python-pip
python-virtualenv
python-setuptools
python-wheel

# Wayland y compositor
wayland
wlroots
tinywl
xorg-xwayland

# Escritorio
xfce4
xfce4-goodies
lightdm
lightdm-gtk-greeter

# Herramientas de desarrollo
base-devel
git
vim
nano
htop
tree

# QuantumEnergyOS específico
qiskit
fastapi
uvicorn
numpy
scipy
pandas
EOF
```

#### 6.2: Crear airootfs

```bash
# Crear estructura de directorios
mkdir -p airootfs/root
mkdir -p airootfs/etc/systemd/system
mkdir -p airootfs/usr/local/bin

# Copiar archivos de QuantumEnergyOS
cp -r /mnt/c/Users/HP/Documents/"Documentos Personales GACB"/Demo/QuantumEnergyOS/QuantumEnergyOS-main/* airootfs/root/

# O si ya clonaste el repo:
# cp -r ~/quantumenergyos/* airootfs/root/
```

#### 6.3: Crear script de instalación automática

```bash
cat > airootfs/root/customize-airootfs.sh << 'EOF'
#!/bin/bash
set -e

echo "[QEOS] Personalizando sistema..."

# Instalar dependencias de Python
cd /root/QuantumEnergyOS
python -m venv venv
source venv/bin/activate
pip install --upgrade pip
pip install -r requirements-pinned.txt 2>/dev/null || pip install -r requirements.txt 2>/dev/null || true

# Instalar QDK
pip install qsharp 2>/dev/null || true

# Crear comando global
cat > /usr/local/bin/qeos << 'CMD'
#!/bin/bash
source /root/QuantumEnergyOS/venv/bin/activate
cd /root/QuantumEnergyOS
exec "$@"
CMD
chmod +x /usr/local/bin/qeos

# Configurar autostart
mkdir -p /home/liveuser/.config/autostart
cat > /home/liveuser/.config/autostart/qeos-api.desktop << 'DESKTOP'
[Desktop Entry]
Type=Application
Name=QuantumEnergyOS API
Exec=bash -c "sleep 5 && qeos uvicorn api.server:app --host 0.0.0.0 --port 8000 & sleep 8 && xdg-open http://localhost:8000"
Hidden=false
NoDisplay=false
X-GNOME-Autostart-enabled=true
DESKTOP

echo "[QEOS] Personalización completada ✓"
EOF

chmod +x airootfs/root/customize-airootfs.sh
```

#### 6.4: Configurar GRUB

```bash
mkdir -p grub

cat > grub/grub.cfg << 'EOF'
set default=0
set timeout=5

insmod all_video
insmod gfxterm
set gfxmode=1920x1080,auto
terminal_output gfxterm

menuentry "⚡ QuantumEnergyOS Live — Iniciar" {
    linux /boot/vmlinuz-linux boot=live quiet splash noswap nopersistence hostname=QuantumEnergyOS username=quantum
    initrd /boot/initramfs-linux.img
}

menuentry "⚡ QuantumEnergyOS Live — Modo seguro" {
    linux /boot/vmlinuz-linux boot=live noswap nopersistence hostname=QuantumEnergyOS username=quantum
    initrd /boot/initramfs-linux.img
}

menuentry "🔧 Instalar en disco duro" {
    linux /boot/vmlinuz-linux boot=live quiet splash only-ubiquity
    initrd /boot/initramfs-linux.img
}

menuentry "Apagar" {
    halt
}
EOF
```

#### 6.5: Configurar profiledef.sh

```bash
cat > profiledef.sh << 'EOF'
#!/usr/bin/env bash
# Configuración del perfil de QuantumEnergyOS

iso_name="QuantumEnergyOS"
iso_label="QEnergOS_$(date +%Y%m)"
iso_publisher="GioCorpus <github.com/GioCorpus>"
iso_application="QuantumEnergyOS Live"
install_dir="arch"
buildmodes=('iso')
bootmodes=('bios.syslinux.mbr' 'bios.syslinux.eltorito' 'uefi-ia32.grub.esp' 'uefi-x64.grub.esp' 'uefi-ia32.grub.eltorito' 'uefi-x64.grub.eltorito')
arch="x86_64"
pacman_conf="pacman.conf"
airootfs_image_type="squashfs"
airootfs_image_tool_options=('-comp' 'xz' '-Xbcj' 'x86' '-b' '1M' '-Xdict-size' '1M')
file_permissions=(
  ["/etc/shadow"]="0:0:0400"
  ["/root"]="0:0:0750"
  ["/root/customize-airootfs.sh"]="0:0:0755"
  ["/usr/local/bin/qeos"]="0:0:0755"
)
EOF

chmod +x profiledef.sh
```

### Paso 7: Construir la ISO

```bash
# Volver al directorio del perfil
cd ~/quantum-build/quantumenergyos

# Construir la ISO (esto tomará 30-60 minutos)
sudo mkarchiso -v -w /tmp/quantum-build -o ~/ISOs .

# O con más opciones:
sudo mkarchiso -v \
  -w /tmp/quantum-build \
  -o ~/ISOs \
  -L "QEnergOS" \
  -P "GioCorpus" \
  -A "QuantumEnergyOS Live" \
  .
```

### Paso 8: Verificar la ISO

```bash
# Listar ISOs generadas
ls -lh ~/ISOs/

# Verificar checksums
cd ~/ISOs
sha256sum *.iso > *.iso.sha256
sha512sum *.iso > *.iso.sha512

# Mostrar información
echo "ISO creada: $(ls *.iso)"
echo "Tamaño: $(du -h *.iso | cut -f1)"
```

### Paso 9: Copiar la ISO a Windows

```powershell
# Desde PowerShell en Windows
wsl -d ArchLinux cp ~/ISOs/*.iso /mnt/c/Users/HP/Desktop/

# O desde WSL
cp ~/ISOs/*.iso /mnt/c/Users/HP/Desktop/
```

## 🔧 Solución de Problemas

### Error: "wsl no se reconoce como comando"

**Solución:** Instalar WSL manualmente:

```powershell
# Descargar WSL2
wsl --install --no-distribution

# Reiniciar
Restart-Computer
```

### Error: "Arch Linux no está en la lista de distribuciones"

**Solución:** Instalar manualmente:

1. Descargar ArchWSL: https://github.com/yuk7/ArchWSL/releases
2. Extraer el archivo `.zip`
3. Ejecutar `Arch.exe`
4. Seguir las instrucciones

### Error: "mkarchiso no encontrado"

**Solución:** Instalar archiso:

```bash
sudo pacman -S archiso
```

### Error: "No hay espacio en disco"

**Solución:** Liberar espacio en WSL:

```bash
# Verificar espacio
df -h

# Limpiar cache de pacman
sudo pacman -Scc

# Eliminar builds anteriores
sudo rm -rf /tmp/quantum-build
```

### Error: "Permiso denegado"

**Solución:** Usar sudo:

```bash
sudo mkarchiso -v -w /tmp/quantum-build -o ~/ISOs .
```

## 📊 Estructura del Perfil

```
quantumenergyos/
├── packages.x86_64          # Lista de paquetes
├── profiledef.sh            # Configuración del perfil
├── pacman.conf              # Configuración de pacman
├── airootfs/                # Root filesystem
│   ├── root/
│   │   ├── QuantumEnergyOS/ # Archivos del proyecto
│   │   └── customize-airootfs.sh
│   └── etc/
│       └── systemd/
└── grub/
    └── grub.cfg             # Configuración de GRUB
```

## 🎨 Personalización Avanzada

### Agregar Paquetes Personalizados

Edita `packages.x86_64` y agrega:

```
# Tus paquetes personalizados
mi-paquete-1
mi-paquete-2
```

### Modificar el Splash Screen

Crea `airootfs/root/splash.sh`:

```bash
#!/bin/bash
# Tu splash screen personalizado
echo "⚡ QuantumEnergyOS — Mexicali no se apaga ⚡"
```

### Cambiar el Kernel

Edita `packages.x86_64`:

```
# En lugar de linux, usar linux-lts o linux-zen
linux-zen
linux-zen-headers
```

## 🖥️ Probar la ISO

### Con QEMU (Desde WSL)

```bash
# Instalar QEMU
sudo pacman -S qemu-full

# Ejecutar la ISO
qemu-system-x86_64 \
  -m 4G \
  -cdrom ~/ISOs/QuantumEnergyOS-*.iso \
  -boot d \
  -enable-kvm
```

### Con VirtualBox (Desde Windows)

1. Copia la ISO a Windows (Paso 9)
2. Abre VirtualBox
3. Crea nueva VM (Linux → Arch Linux 64-bit)
4. Asigna 4 GB de RAM
5. Selecciona la ISO como disco óptico
6. Inicia la VM

### Grabar en USB

```powershell
# Desde PowerShell en Windows
# Usar Rufus o balenaEtcher con la ISO copiada al escritorio
```

## 📝 Comandos Útiles de WSL

```powershell
# Listar distribuciones instaladas
wsl --list --verbose

# Iniciar Arch Linux
wsl -d ArchLinux

# Detener WSL
wsl --shutdown

# Exportar distribución
wsl --export ArchLinux arch-backup.tar

# Importar distribución
wsl --import ArchLinux C:\WSL\Arch arch-backup.tar
```

## 🚀 Automatización

### Script de Build Automatizado

Crea `build-iso.sh` en WSL:

```bash
#!/bin/bash
set -e

echo "🚀 Iniciando build de QuantumEnergyOS ISO..."

# Actualizar sistema
sudo pacman -Syu --noconfirm

# Construir ISO
cd ~/quantum-build/quantumenergyos
sudo mkarchiso -v -w /tmp/quantum-build -o ~/ISOs .

# Copiar a Windows
cp ~/ISOs/*.iso /mnt/c/Users/HP/Desktop/

echo "✅ ISO copiada al escritorio de Windows"
```

Hacer ejecutable:

```bash
chmod +x build-iso.sh
./build-iso.sh
```

## 📚 Recursos Adicionales

- **WSL Documentación:** https://learn.microsoft.com/en-us/windows/wsl/
- **ArchWSL:** https://github.com/yuk7/ArchWSL
- **archiso Wiki:** https://wiki.archlinux.org/title/archiso
- **QuantumEnergyOS:** https://github.com/GioCorpus/QuantumEnergyOS

## 🆘 Soporte

Si encuentras problemas:

1. Verifica que WSL esté instalado: `wsl --list`
2. Verifica que Arch Linux esté corriendo: `wsl -d ArchLinux uname -a`
3. Verifica que archiso esté instalado: `which mkarchiso`
4. Consulta la wiki de Arch Linux

---

**⚡ QuantumEnergyOS — Desde Mexicali, BC — para el mundo entero. Kardashev 0→1**
