# QuantumEnergyOS — Build ISO en Windows

## 📋 Descripción

Este documento explica cómo construir una ISO bootable de QuantumEnergyOS directamente desde Windows 10/11 usando PowerShell.

## 🎯 Requisitos Previos

### Opción 1: Windows ADK (Recomendado)

Descarga e instala el **Windows Assessment and Deployment Kit (ADK)**:

1. Descarga desde: https://learn.microsoft.com/en-us/windows-hardware/get-started/adk-install
2. Durante la instalación, selecciona:
   - **Deployment Tools** (incluye oscdimg)
   - **User State Migration Tool** (opcional)

### Opción 2: mkisofs (Alternativa)

Si prefieres no instalar el ADK completo:

```powershell
# Con Chocolatey
choco install mkisofs -y

# Con Scoop
scoop install mkisofs
```

### Opción 3: WSL (Windows Subsystem for Linux)

Para usar los scripts Linux originales:

```powershell
# Instalar WSL
wsl --install

# Reiniciar y luego instalar dependencias en Ubuntu
sudo apt-get update
sudo apt-get install -y live-build squashfs-tools xorriso grub-common
```

## 🚀 Instrucciones de Uso

### Paso 1: Preparar el Entorno

Abre **PowerShell como Administrador**:

```powershell
# Navegar al directorio del proyecto
cd "C:\Users\HP\Documents\Documentos Personales GACB\Demo\QuantumEnergyOS\QuantumEnergyOS-main"

# Permitir ejecución de scripts
Set-ExecutionPolicy Bypass -Scope Process -Force
```

### Paso 2: Ejecutar el Script de Build

```powershell
# Build estándar (amd64)
.\build-iso-windows.ps1

# Build con arquitectura específica
.\build-iso-windows.ps1 -Architecture amd64

# Build con directorio de salida personalizado
.\build-iso-windows.ps1 -OutputDir "C:\ISOs"

# Build sin compilar el compositor (más rápido)
.\build-iso-windows.ps1 -SkipCompositor
```

### Paso 3: Esperar la Construcción

El proceso tomará entre **15-45 minutos** dependiendo de:
- Velocidad del disco duro (SSD vs HDD)
- Velocidad de internet (para descargar dependencias)
- Potencia del CPU

### Paso 4: Verificar la ISO

Una vez completado, la ISO estará en:

```
.\output\QuantumEnergyOS-v1.0.0-live-amd64.iso
```

Con sus checksums:
- `QuantumEnergyOS-v1.0.0-live-amd64.iso.sha256`
- `QuantumEnergyOS-v1.0.0-live-amd64.iso.sha512`

## 📦 Parámetros del Script

| Parámetro | Descripción | Valor por Defecto |
|-----------|-------------|-------------------|
| `-Architecture` | Arquitectura del sistema (amd64, arm64) | `amd64` |
| `-OutputDir` | Directorio donde se guardará la ISO | `.\output` |
| `-SkipCompositor` | Omitir la compilación del compositor | `false` |
| `-UseWSL` | Usar WSL para tareas Linux | `false` |

## 🔧 Solución de Problemas

### Error: "oscdimg.exe no encontrado"

**Solución:** Instala Windows ADK con la opción "Deployment Tools".

```powershell
# Verificar si oscdimg está instalado
Test-Path "${env:ProgramFiles(x86)}\Windows Kits\10\Assessment and Deployment Kit\Deployment Tools\amd64\Oscdimg\oscdimg.exe"
```

### Error: "gzip no encontrado"

**Solución:** Instala gzip:

```powershell
# Con Chocolatey
choco install gzip -y

# Con Scoop
scoop install gzip
```

### Error: "Permiso denegado"

**Solución:** Ejecuta PowerShell como Administrador:

1. Busca "PowerShell" en el menú inicio
2. Haz clic derecho → "Ejecutar como administrador"
3. Navega al directorio del proyecto
4. Ejecuta el script nuevamente

### Error: "No se pudo crear la ISO"

**Solución:** Verifica que:
1. Tienes al menos **10 GB de espacio libre** en disco
2. No hay otros procesos usando el directorio de build
3. Tu antivirus no está bloqueando la creación de archivos

## 🎨 Personalización

### Agregar Paquetes Personalizados

Edita el script `build-iso-windows.ps1` y busca la función `Copy-QuantumEnergyOS`. Agrega tus archivos personalizados ahí.

### Modificar el Splash Screen

Edita la función `New-SplashScreen` en el script para personalizar:
- Colores
- Mensajes
- Animaciones

### Cambiar la Configuración de GRUB

Edita la función `New-GrubConfig` para modificar:
- Tiempo de espera
- Opciones del menú
- Parámetros del kernel

## 🖥️ Probar la ISO

### Opción 1: QEMU (Recomendado para pruebas rápidas)

```powershell
# Instalar QEMU
choco install qemu -y

# Ejecutar la ISO
qemu-system-x86_64 -m 4G -cdrom .\output\QuantumEnergyOS-v1.0.0-live-amd64.iso -boot d
```

### Opción 2: VirtualBox

1. Abre VirtualBox
2. Crea una nueva máquina virtual (Linux → Ubuntu 64-bit)
3. Asigna al menos 4 GB de RAM
4. En "Almacenamiento", selecciona la ISO como disco óptico
5. Inicia la máquina virtual

### Opción 3: VMware Workstation

1. Abre VMware
2. Crea una nueva VM (Linux → Ubuntu 64-bit)
3. Asigna al menos 4 GB de RAM
4. En "CD/DVD", selecciona la ISO
5. Inicia la VM

### Opción 4: Grabar en USB (Para hardware real)

#### Con Rufus (Recomendado)

1. Descarga Rufus: https://rufus.ie
2. Inserta un USB de al menos 8 GB
3. Selecciona la ISO de QuantumEnergyOS
4. Haz clic en "Iniciar"

#### Con balenaEtcher

1. Descarga balenaEtcher: https://www.balena.io/etcher/
2. Selecciona la ISO
3. Selecciona el USB
4. Haz clic en "Flash!"

## 📊 Estructura de la ISO

```
QuantumEnergyOS-v1.0.0-live-amd64.iso/
├── boot/
│   ├── vmlinuz              # Kernel de Linux
│   ├── initramfs.gz         # Initramfs con compositor
│   └── grub/
│       └── grub.cfg         # Configuración de GRUB
├── opt/
│   └── QuantumEnergyOS/     # Archivos del sistema
│       ├── api/
│       ├── src/
│       ├── kernel/
│       └── compositor/
└── sbin/
    └── init                 # Script de inicio
```

## 🔐 Verificar Integridad

Después de descargar o copiar la ISO, verifica su integridad:

```powershell
# Verificar SHA-256
$expectedHash = Get-Content ".\output\QuantumEnergyOS-v1.0.0-live-amd64.iso.sha256"
$actualHash = (Get-FileHash ".\output\QuantumEnergyOS-v1.0.0-live-amd64.iso" -Algorithm SHA256).Hash

if ($expectedHash -eq $actualHash) {
    Write-Host "✓ Integridad verificada" -ForegroundColor Green
} else {
    Write-Host "✗ La ISO está corrupta" -ForegroundColor Red
}
```

## 🌟 Características de la ISO

- **Tamaño:** ~2.5 GB
- **Arquitectura:** amd64 (x86_64)
- **Base:** Ubuntu 24.04 LTS (Noble Numbat)
- **Escritorio:** XFCE4 (ligero)
- **Compositor:** Wayland (tinywl)
- **Bootloader:** GRUB2
- **Modo:** Live (no requiere instalación)

## 📝 Notas Importantes

1. **Espacio en disco:** Necesitas al menos **10 GB libres** para el build
2. **Tiempo de build:** El proceso puede tomar **30-60 minutos**
3. **Antivirus:** Algunos antivirus pueden interferir con el build
4. **Permisos:** Necesitas ejecutar como **Administrador**
5. **Internet:** Se requiere conexión a internet para descargar dependencias

## 🆘 Soporte

Si encuentras problemas:

1. Revisa los logs en `.\build\iso-amd64\`
2. Verifica que todos los requisitos estén instalados
3. Consulta la documentación del proyecto principal
4. Abre un issue en GitHub

## 🚀 Siguientes Pasos

Después de crear la ISO:

1. **Prueba en VM** para verificar que funciona
2. **Graba en USB** para probar en hardware real
3. **Comparte** con la comunidad
4. **Contribuye** mejoras al proyecto

---

**⚡ QuantumEnergyOS — Desde Mexicali, BC — para el mundo entero. Kardashev 0→1**
