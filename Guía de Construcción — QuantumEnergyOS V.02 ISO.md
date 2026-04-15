# Guía de Construcción — QuantumEnergyOS V.02 ISO

## Resumen

Este documento describe el proceso completo para construir la imagen ISO booteable de **QuantumEnergyOS V.02** usando el perfil `archiso` incluido en el archivo `QuantumEnergyOS-archiso-profile.zip`.

---

## Paso 1: Preparar el entorno en Arch Linux

Abre una terminal en tu máquina virtual de Arch Linux e instala `archiso`:

```bash
sudo pacman -S archiso
```

Verifica que la instalación fue exitosa:

```bash
mkarchiso --version
```

---

## Paso 2: Copiar el perfil al sistema

Extrae el archivo ZIP en tu directorio de inicio:

```bash
cd ~
unzip QuantumEnergyOS-archiso-profile.zip
cd qeos-profile
ls
```

Deberías ver los siguientes archivos:
- `profiledef.sh` — Configuración principal
- `packages.x86_64` — Lista de paquetes
- `pacman.conf` — Repositorios de pacman
- `BUILD.sh` — Script de construcción automático
- `grub/grub.cfg` — Bootloader UEFI
- `syslinux/syslinux.cfg` — Bootloader BIOS
- `airootfs/` — Sistema de archivos raíz de la ISO

---

## Paso 3: Construir la ISO

### Opción A — Script automático (recomendado)

```bash
sudo bash BUILD.sh
```

### Opción B — Comando manual

```bash
sudo mkarchiso -v -w /tmp/qeos-work -o ~/ISOs ~/qeos-profile
```

> **Nota**: El proceso puede tardar entre **30 y 60 minutos** dependiendo de la velocidad de internet y del hardware. Se descargarán aproximadamente 1-2 GB de paquetes desde los repositorios de Arch Linux.

---

## Paso 4: Verificar la ISO

Una vez completado el proceso, la ISO estará en `~/ISOs/`:

```bash
ls -lh ~/ISOs/
```

Verifica la integridad:

```bash
sha256sum ~/ISOs/QuantumEnergyOS-*.iso
```

---

## Paso 5: Probar la ISO

### En VirtualBox / VMware

1. Crear una nueva máquina virtual (Linux, 64-bit)
2. Asignar al menos 2 GB de RAM y 20 GB de disco
3. Montar la ISO como unidad óptica
4. Iniciar la máquina virtual

### En QEMU (línea de comandos)

```bash
qemu-system-x86_64 -m 2G -cdrom ~/ISOs/QuantumEnergyOS-*.iso -boot d
```

### En USB físico (con dd)

```bash
sudo dd if=~/ISOs/QuantumEnergyOS-*.iso of=/dev/sdX bs=4M status=progress && sync
```

> **Advertencia**: Reemplaza `/dev/sdX` con el dispositivo USB correcto. Verifica con `lsblk`.

---

## Credenciales del sistema en vivo

| Campo | Valor |
|-------|-------|
| Usuario | `quantum` |
| Contraseña | `quantum` |
| Root | `quantum` |
| Hostname | `QuantumEnergyOS` |

---

## Comandos especiales disponibles

| Comando | Descripción |
|---------|-------------|
| `qeos` | Lanzador principal de QuantumEnergyOS |
| `qsh` | Shell cuántico interactivo |

---

## Características incluidas en la ISO

| Componente | Versión/Detalle |
|------------|-----------------|
| Base | Arch Linux (rolling release) |
| Kernel | Linux (última versión estable) |
| Compositor | Sway (Wayland) |
| Idioma | Español México (es_MX.UTF-8) |
| Zona horaria | America/Tijuana |
| Python | 3.11+ con Qiskit, Flask, FastAPI |
| Rust | Toolchain completo |
| Red | NetworkManager |
| SSH | OpenSSH |

---

## Solución de problemas

### Error: "pacman is not available"
Asegúrate de ejecutar el comando en **Arch Linux**, no en Ubuntu o Debian.

### Error: "not enough free disk space"
Libera espacio en disco. Se necesitan al menos 20 GB libres en `/tmp` y en el directorio de salida.

### Error: "keyring"
Ejecuta antes de construir:
```bash
sudo pacman-key --init
sudo pacman-key --populate archlinux
sudo pacman -Sy archlinux-keyring
```

### La ISO no arranca en VirtualBox
Asegúrate de habilitar **EFI** en la configuración de la máquina virtual si usas modo UEFI, o desactívalo para modo BIOS.

---

## Soporte

Proyecto: [github.com/GioCorpus/QuantumEnergyOS](https://github.com/GioCorpus)

---

*Generado automáticamente — QuantumEnergyOS V.02 Build System*
