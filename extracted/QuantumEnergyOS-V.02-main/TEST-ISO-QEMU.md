# QuantumEnergyOS — Probar ISO con QEMU

## 🎯 Comando QEMU para Windows

```powershell
# Instalar QEMU (si no lo tienes)
choco install qemu -y

# Ejecutar la ISO
qemu-system-x86_64 -cdrom QuantumEnergyOS.iso -boot d -m 2048 -cpu host
```

## 📋 Opciones de QEMU

### Básico (Recomendado para pruebas rápidas)
```powershell
qemu-system-x86_64 -cdrom QuantumEnergyOS.iso -boot d -m 2048
```

### Con más memoria (Para uso completo)
```powershell
qemu-system-x86_64 -cdrom QuantumEnergyOS.iso -boot d -m 4096 -cpu host
```

### Con aceleración KVM (Linux/WSL solamente)
```powershell
qemu-system-x86_64 -cdrom QuantumEnergyOS.iso -boot d -m 4096 -enable-kvm -cpu host
```

### Con red habilitada
```powershell
qemu-system-x86_64 -cdrom QuantumEnergyOS.iso -boot d -m 2048 -net nic -net user
```

### Con pantalla completa
```powershell
qemu-system-x86_64 -cdrom QuantumEnergyOS.iso -boot d -m 2048 -full-screen
```

### Con USB habilitado
```powershell
qemu-system-x86_64 -cdrom QuantumEnergyOS.iso -boot d -m 2048 -usb -device usb-tablet
```

## 🔧 Solución de Problemas

### Error: "qemu-system-x86_64 no se reconoce como comando"

**Solución:** Instalar QEMU:

```powershell
# Con Chocolatey
choco install qemu -y

# Con Scoop
scoop install qemu

# Manual: Descargar desde https://www.qemu.org/download/
```

### Error: "No se puede iniciar la VM"

**Solución:** Verificar que la ISO existe:

```powershell
Test-Path .\QuantumEnergyOS.iso
```

### Error: "Pantalla negra"

**Solución:** Agregar opciones de video:

```powershell
qemu-system-x86_64 -cdrom QuantumEnergyOS.iso -boot d -m 2048 -vga virtio
```

### Error: "Muy lento"

**Solución:** Habilitar aceleración:

```powershell
# En Windows (sin KVM)
qemu-system-x86_64 -cdrom QuantumEnergyOS.iso -boot d -m 2048 -accel whpx

# En Linux/WSL
qemu-system-x86_64 -cdrom QuantumEnergyOS.iso -boot d -m 2048 -enable-kvm -cpu host
```

## 🎨 Personalizar VM

### Crear disco duro virtual (Para instalación permanente)

```powershell
# Crear disco de 20 GB
qemu-img create -f qcow2 quantum-disk.qcow2 20G

# Ejecutar con disco
qemu-system-x86_64 -cdrom QuantumEnergyOS.iso -boot d -m 2048 -hda quantum-disk.qcow2
```

### Guardar estado de la VM

```powershell
# Guardar estado
qemu-system-x86_64 -cdrom QuantumEnergyOS.iso -boot d -m 2048 -loadvm mystate

# Cargar estado
qemu-system-x86_64 -cdrom QuantumEnergyOS.iso -boot d -m 2048 -loadvm mystate
```

## 📊 Opciones Comunes

| Opción | Descripción | Ejemplo |
|--------|-------------|---------|
| `-cdrom` | Archivo ISO | `-cdrom QuantumEnergyOS.iso` |
| `-boot` | Orden de boot | `-boot d` (CD-ROM) |
| `-m` | Memoria RAM | `-m 2048` (2 GB) |
| `-cpu` | Tipo de CPU | `-cpu host` |
| `-enable-kvm` | Aceleración KVM | Solo Linux |
| `-vga` | Tipo de video | `-vga virtio` |
| `-net` | Configuración de red | `-net nic -net user` |
| `-usb` | Habilitar USB | `-usb -device usb-tablet` |
| `-full-screen` | Pantalla completa | `-full-screen` |

## 🚀 Scripts de Inicio Rápido

### test-iso.ps1 (PowerShell)

```powershell
# test-iso.ps1
param(
    [string]$IsoPath = ".\QuantumEnergyOS.iso",
    [int]$Memory = 2048
)

if (-not (Test-Path $IsoPath)) {
    Write-Host "Error: ISO no encontrada en $IsoPath" -ForegroundColor Red
    exit 1
}

Write-Host "Iniciando QuantumEnergyOS en QEMU..." -ForegroundColor Cyan
Write-Host "ISO: $IsoPath" -ForegroundColor Yellow
Write-Host "Memoria: $Memory MB" -ForegroundColor Yellow
Write-Host ""

qemu-system-x86_64 -cdrom $IsoPath -boot d -m $Memory -cpu host
```

Uso:

```powershell
.\test-iso.ps1
.\test-iso.ps1 -IsoPath ".\output\QuantumEnergyOS-v1.0.0-live-amd64.iso" -Memory 4096
```

## 📝 Notas

- **Sin KVM:** En Windows, QEMU funciona sin aceleración KVM (más lento pero funcional)
- **Con KVM:** En WSL/Linux, habilita KVM para mejor rendimiento
- **Memoria:** Asigna al menos 2 GB, recomendado 4 GB para uso completo
- **Red:** La opción `-net nic -net user` habilita internet en la VM
- **USB:** Útil para probar con dispositivos USB reales

## 🔗 Recursos

- **QEMU Documentación:** https://www.qemu.org/docs/master/
- **QEMU Windows:** https://www.qemu.org/download/#windows
- **Chocolatey QEMU:** https://community.chocolatey.org/packages/qemu

---

**⚡ QuantumEnergyOS — Desde Mexicali, BC — para el mundo entero. Kardashev 0→1**
