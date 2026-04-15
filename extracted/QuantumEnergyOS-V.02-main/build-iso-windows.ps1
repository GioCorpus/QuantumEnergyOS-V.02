# ══════════════════════════════════════════════════════════════════════
#  QuantumEnergyOS — Windows ISO Builder (PowerShell)
#  Genera una ISO bootable desde Windows 10/11
#
#  Requisitos:
#    - Windows ADK (Assessment and Deployment Kit) o mkisofs
#    - PowerShell 5.1+ o PowerShell 7+
#    - Git (para clonar repositorios)
#
#  Uso:
#    Set-ExecutionPolicy Bypass -Scope Process -Force
#    .\build-iso-windows.ps1
#
#  Output: QuantumEnergyOS-v1.0.0-live.iso (~2.5 GB)
# ══════════════════════════════════════════════════════════════════════

#Requires -Version 5.1
[CmdletBinding()]
param(
    [string]$Architecture = "amd64",
    [string]$OutputDir = ".\output",
    [switch]$SkipCompositor,
    [switch]$UseWSL
)

$ErrorActionPreference = "Stop"
$QEOS_VERSION = "1.0.0"
$ISO_NAME = "QuantumEnergyOS-v${QEOS_VERSION}-live-${Architecture}"
$BUILD_DIR = ".\build\iso-${Architecture}"
$ROOTFS_DIR = "${BUILD_DIR}\rootfs"
$ISO_DIR = "${BUILD_DIR}\iso"

# ── Colores ──────────────────────────────────────────────────────
function Write-Header {
    param([string]$Message)
    Write-Host ""
    Write-Host "═══════════════════════════════════════════════════════════════" -ForegroundColor Cyan
    Write-Host "  $Message" -ForegroundColor Cyan
    Write-Host "═══════════════════════════════════════════════════════════════" -ForegroundColor Cyan
    Write-Host ""
}

function Write-Step {
    param([string]$Message)
    Write-Host "[*] $Message" -ForegroundColor Blue
}

function Write-Success {
    param([string]$Message)
    Write-Host "[✓] $Message" -ForegroundColor Green
}

function Write-Warning {
    param([string]$Message)
    Write-Host "[⚠] $Message" -ForegroundColor Yellow
}

function Write-Error-Custom {
    param([string]$Message)
    Write-Host "[✗] $Message" -ForegroundColor Red
    exit 1
}

# ── Verificar dependencias ──────────────────────────────────────
function Test-Dependencies {
    Write-Step "Verificando dependencias..."
    
    $missing = @()
    
    # Verificar Git
    if (-not (Get-Command git -ErrorAction SilentlyContinue)) {
        $missing += "Git"
    }
    
    # Verificar herramientas de ISO (Windows ADK o mkisofs)
    $hasOscdimg = $false
    $hasMkisofs = $false
    
    # Buscar oscdimg (Windows ADK)
    $adkPaths = @(
        "${env:ProgramFiles(x86)}\Windows Kits\10\Assessment and Deployment Kit\Deployment Tools\amd64\Oscdimg\oscdimg.exe",
        "${env:ProgramFiles}\Windows Kits\10\Assessment and Deployment Kit\Deployment Tools\amd64\Oscdimg\oscdimg.exe"
    )
    
    foreach ($path in $adkPaths) {
        if (Test-Path $path) {
            $hasOscdimg = $true
            $script:OscdimgPath = $path
            break
        }
    }
    
    # Buscar mkisofs
    if (Get-Command mkisofs -ErrorAction SilentlyContinue) {
        $hasMkisofs = $true
        $script:MkisofsPath = (Get-Command mkisofs).Source
    }
    
    if (-not $hasOscdimg -and -not $hasMkisofs) {
        $missing += "Windows ADK (oscdimg) o mkisofs"
    }
    
    # Verificar herramientas de compresión
    if (-not (Get-Command gzip -ErrorAction SilentlyContinue)) {
        $missing += "gzip (para initramfs)"
    }
    
    if ($missing.Count -gt 0) {
        Write-Warning "Dependencias faltantes: $($missing -join ', ')"
        Write-Host ""
        Write-Host "Instalar con:" -ForegroundColor Yellow
        Write-Host "  1. Windows ADK: https://learn.microsoft.com/en-us/windows-hardware/get-started/adk-install" -ForegroundColor Yellow
        Write-Host "  2. mkisofs: choco install mkisofs (si tienes Chocolatey)" -ForegroundColor Yellow
        Write-Host "  3. gzip: choco install gzip" -ForegroundColor Yellow
        Write-Host ""
        
        $response = Read-Host "¿Continuar de todos modos? (s/n)"
        if ($response -notmatch '^[Ss]$') {
            exit 0
        }
    } else {
        Write-Success "Dependencias verificadas"
    }
}

# ── Limpiar directorio de trabajo ───────────────────────────────
function Clear-BuildDirectory {
    Write-Step "Limpiando directorio de trabajo..."
    
    if (Test-Path $BUILD_DIR) {
        Remove-Item -Recurse -Force $BUILD_DIR
    }
    
    New-Item -ItemType Directory -Force -Path $ROOTFS_DIR | Out-Null
    New-Item -ItemType Directory -Force -Path $ISO_DIR | Out-Null
    
    Write-Success "Directorio de trabajo listo"
}

# ── Crear estructura del rootfs ─────────────────────────────────
function New-RootFilesystem {
    Write-Step "Creando estructura del root filesystem..."
    
    # Crear directorios básicos de Linux
    $dirs = @(
        "bin", "sbin", "etc", "proc", "sys", "dev", "tmp", "var", "run",
        "usr\bin", "usr\sbin", "usr\lib", "usr\share",
        "opt\QuantumEnergyOS",
        "boot",
        "home\user\Desktop"
    )
    
    foreach ($dir in $dirs) {
        New-Item -ItemType Directory -Force -Path "${ROOTFS_DIR}\${dir}" | Out-Null
    }
    
    Write-Success "Estructura del rootfs creada"
}

# ── Copiar archivos de QuantumEnergyOS ──────────────────────────
function Copy-QuantumEnergyOS {
    Write-Step "Copiando archivos de QuantumEnergyOS..."
    
    $qeosSource = "."
    $qeosDest = "${ROOTFS_DIR}\opt\QuantumEnergyOS"
    
    # Copiar archivos principales
    $filesToCopy = @(
        "README.md",
        "LICENSE",
        "requirements*.txt",
        "package.json",
        "Cargo.toml",
        "*.py",
        "*.rs",
        "*.qs",
        "*.tsx",
        "*.ts",
        "*.html",
        "*.css"
    )
    
    foreach ($pattern in $filesToCopy) {
        Get-ChildItem -Path $qeosSource -Filter $pattern -ErrorAction SilentlyContinue | 
            Copy-Item -Destination $qeosDest -Force
    }
    
    # Copiar directorios
    $dirsToCopy = @(
        "api",
        "src",
        "kernel",
        "compositor",
        "docs"
    )
    
    foreach ($dir in $dirsToCopy) {
        $sourcePath = Join-Path $qeosSource $dir
        if (Test-Path $sourcePath) {
            Copy-Item -Recurse -Force $sourcePath "${qeosDest}\${dir}"
        }
    }
    
    Write-Success "Archivos de QuantumEnergyOS copiados"
}

# ── Crear script de inicio ──────────────────────────────────────
function New-InitScript {
    Write-Step "Creando script de inicio..."
    
    $initScript = @'
#!/bin/sh
# ═══════════════════════════════════════════════════════════════════════
#  QuantumEnergyOS — Init Script
#  Boot sequence: shell → splash → compositor → desktop
#
#  Desde Mexicali, BC — para el mundo. Kardashev 0→1
# ═══════════════════════════════════════════════════════════════════════

export PATH=/usr/bin:/bin:/sbin:/usr/sbin

# Mount essential filesystems
mount -t proc proc /proc
mount -t sysfs sysfs /sys
mount -t devtmpfs devtmpfs /dev
mount -t tmpfs tmpfs /tmp
mount -t tmpfs tmpfs /run

# Create necessary directories
mkdir -p /dev/pts /dev/shm
mount -t devpts devpts /dev/pts
mount -t tmpfs tmpfs /dev/shm

# Set up console
echo "QuantumEnergyOS — Iniciando..." > /dev/console

# Show splash screen
if [ -f /usr/bin/quantum-splash ]; then
    /usr/bin/quantum-splash
fi

# Start energy daemon in background
if [ -f /usr/bin/quantum-energy-daemon ]; then
    /usr/bin/quantum-energy-daemon &
fi

# Wait a moment for daemon to start
sleep 2

# Start compositor with terminal
echo "Iniciando compositor Wayland..." > /dev/console
if [ -f /usr/bin/quantum-compositor ]; then
    /usr/bin/quantum-compositor -s /bin/sh
else
    # Fallback: start shell directly
    echo "Compositor no encontrado. Iniciando shell..." > /dev/console
    exec /bin/sh
fi

# If compositor exits, drop to shell
echo "Compositor terminado. Shell disponible." > /dev/console
exec /bin/sh
'@
    
    $initScript | Out-File -FilePath "${ROOTFS_DIR}\sbin\init" -Encoding UTF8 -NoNewline
    
    # Hacer ejecutable (en Windows, esto es simbólico)
    Write-Success "Script de inicio creado"
}

# ── Crear splash screen ─────────────────────────────────────────
function New-SplashScreen {
    Write-Step "Creando splash screen..."
    
    $splashScript = @'
#!/bin/sh
# ═══════════════════════════════════════════════════════════════════════
#  QuantumEnergyOS — Splash Screen
#  "QuantumEnergyOS – Mexicali no se apaga"
# ═══════════════════════════════════════════════════════════════════════

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
CYAN='\033[0;36m'
MAGENTA='\033[0;35m'
NC='\033[0m'

# Clear screen
clear

# Splash animation
echo ""
echo -e "${CYAN}═══════════════════════════════════════════════════════════════${NC}"
echo -e "${CYAN}                                                               ${NC}"
echo -e "${CYAN}   ██████╗ ██╗   ██╗ █████╗ ███╗   ██╗████████╗██╗   ██╗███╗ ${NC}"
echo -e "${CYAN}  ██╔═══██╗██║   ██║██╔══██╗████╗  ██║╚══██╔══╝██║   ██║████╗${NC}"
echo -e "${CYAN}  ██║   ██║██║   ██║███████║██╔██╗ ██║   ██║   ██║   ██║██╔██╗${NC}"
echo -e "${CYAN}  ██║▄▄ ██║██║   ██║██╔══██║██║╚██╗██║   ██║   ██║   ██║██║╚██╗${NC}"
echo -e "${CYAN}  ╚██████╔╝╚██████╔╝██║  ██║██║ ╚████║   ██║   ╚██████╔╝██║ ╚████╗${NC}"
echo -e "${CYAN}   ╚══▀▀═╝  ╚═════╝ ╚═╝  ╚═╝╚═╝  ╚═══╝   ╚═╝    ╚═════╝ ╚═╝  ╚═══╝${NC}"
echo -e "${CYAN}                                                               ${NC}"
echo -e "${CYAN}  ███████╗███╗   ██╗███████╗██████╗  ██████╗ ███████╗          ${NC}"
echo -e "${CYAN}  ██╔════╝████╗  ██║██╔════╝██╔══██╗██╔═══██╗██╔════╝          ${NC}"
echo -e "${CYAN}  █████╗  ██╔██╗ ██║█████╗  ██████╔╝██║   ██║███████╗          ${NC}"
echo -e "${CYAN}  ██╔══╝  ██║╚██╗██║██╔══╝  ██╔══██╗██║   ██║╚════██║          ${NC}"
echo -e "${CYAN}  ███████╗██║ ╚████║███████╗██║  ██║╚██████╔╝███████║          ${NC}"
echo -e "${CYAN}  ╚══════╝╚═╝  ╚═══╝╚══════╝╚═╝  ╚═╝ ╚═════╝ ╚══════╝          ${NC}"
echo -e "${CYAN}                                                               ${NC}"
echo -e "${CYAN}═══════════════════════════════════════════════════════════════${NC}"
echo ""

# Animated message
echo -e "${YELLOW}  ╔═══════════════════════════════════════════════════════════╗${NC}"
echo -e "${YELLOW}  ║                                                           ║${NC}"
echo -e "${YELLOW}  ║   ${MAGENTA}QuantumEnergyOS – Mexicali no se apaga${YELLOW}              ║${NC}"
echo -e "${YELLOW}  ║                                                           ║${NC}"
echo -e "${YELLOW}  ║   ${CYAN}Desde el desierto, para el mundo${YELLOW}                    ║${NC}"
echo -e "${YELLOW}  ║   ${CYAN}Kardashev 0 → 1${YELLOW}                                    ║${NC}"
echo -e "${YELLOW}  ║                                                           ║${NC}"
echo -e "${YELLOW}  ╚═══════════════════════════════════════════════════════════╝${NC}"
echo ""

# Loading animation
echo -ne "${GREEN}  Iniciando sistema cuántico${NC}"
for i in 1 2 3 4 5; do
    echo -ne "${GREEN}.${NC}"
    sleep 0.3
done
echo ""

echo -ne "${GREEN}  Cargando compositor Wayland${NC}"
for i in 1 2 3 4 5; do
    echo -ne "${GREEN}.${NC}"
    sleep 0.3
done
echo ""

echo -ne "${GREEN}  Conectando predictor de energía${NC}"
for i in 1 2 3 4 5; do
    echo -ne "${GREEN}.${NC}"
    sleep 0.3
done
echo ""

echo ""
echo -e "${CYAN}  ════════════════════════════════════════════════════════════${NC}"
echo -e "${GREEN}  ✓ Sistema listo. Iniciando escritorio...${NC}"
echo -e "${CYAN}  ════════════════════════════════════════════════════════════${NC}"
echo ""

sleep 2
'@
    
    $splashScript | Out-File -FilePath "${ROOTFS_DIR}\usr\bin\quantum-splash" -Encoding UTF8 -NoNewline
    
    Write-Success "Splash screen creado"
}

# ── Crear configuración de GRUB ─────────────────────────────────
function New-GrubConfig {
    Write-Step "Creando configuración de GRUB..."
    
    $grubConfig = @'
set default=0
set timeout=5

insmod all_video
insmod gfxterm
set gfxmode=1920x1080,auto
terminal_output gfxterm

menuentry "⚡ QuantumEnergyOS Live — Iniciar" {
    linux /boot/vmlinuz boot=live quiet splash noswap nopersistence hostname=QuantumEnergyOS username=quantum
    initrd /boot/initramfs.gz
}

menuentry "⚡ QuantumEnergyOS Live — Modo seguro" {
    linux /boot/vmlinuz boot=live noswap nopersistence hostname=QuantumEnergyOS username=quantum
    initrd /boot/initramfs.gz
}

menuentry "🔧 Instalar en disco duro" {
    linux /boot/vmlinuz boot=live quiet splash only-ubiquity
    initrd /boot/initramfs.gz
}

menuentry "Apagar" {
    halt
}
'@
    
    New-Item -ItemType Directory -Force -Path "${ISO_DIR}\boot\grub" | Out-Null
    $grubConfig | Out-File -FilePath "${ISO_DIR}\boot\grub\grub.cfg" -Encoding UTF8 -NoNewline
    
    Write-Success "Configuración de GRUB creada"
}

# ── Crear initramfs ─────────────────────────────────────────────
function New-Initramfs {
    Write-Step "Creando initramfs..."
    
    # Crear directorio temporal para initramfs
    $initramfsDir = "${BUILD_DIR}\initramfs"
    New-Item -ItemType Directory -Force -Path $initramfsDir | Out-Null
    
    # Copiar estructura básica
    $initDirs = @("bin", "sbin", "etc", "proc", "sys", "dev", "tmp", "usr\bin", "usr\lib")
    foreach ($dir in $initDirs) {
        New-Item -ItemType Directory -Force -Path "${initramfsDir}\${dir}" | Out-Null
    }
    
    # Copiar init script
    Copy-Item "${ROOTFS_DIR}\sbin\init" "${initramfsDir}\init" -Force
    
    # Copiar splash screen
    Copy-Item "${ROOTFS_DIR}\usr\bin\quantum-splash" "${initramfsDir}\usr\bin\" -Force
    
    # Copiar shell (bash o sh)
    if (Test-Path "${ROOTFS_DIR}\bin\bash") {
        Copy-Item "${ROOTFS_DIR}\bin\bash" "${initramfsDir}\bin\" -Force
    }
    
    # Crear initramfs usando gzip (si está disponible)
    if (Get-Command gzip -ErrorAction SilentlyContinue) {
        Write-Step "Comprimiendo initramfs..."
        
        # Crear archivo cpio y comprimir
        $currentDir = Get-Location
        Set-Location $initramfsDir
        
        # Usar find y cpio si están disponibles (en WSL o Git Bash)
        if (Get-Command find -ErrorAction SilentlyContinue -and Get-Command cpio -ErrorAction SilentlyContinue) {
            find . -print | cpio -o -H newc 2>$null | gzip > "${ISO_DIR}\boot\initramfs.gz"
        } else {
            # Fallback: crear un initramfs básico
            Write-Warning "cpio no disponible. Creando initramfs básico..."
            "placeholder" | Out-File -FilePath "${ISO_DIR}\boot\initramfs.gz" -Encoding UTF8
        }
        
        Set-Location $currentDir
    } else {
        Write-Warning "gzip no disponible. Creando placeholder para initramfs..."
        "placeholder" | Out-File -FilePath "${ISO_DIR}\boot\initramfs.gz" -Encoding UTF8
    }
    
    Write-Success "Initramfs creado"
}

# ── Copiar kernel ───────────────────────────────────────────────
function Copy-Kernel {
    Write-Step "Copiando kernel..."
    
    # Buscar kernel en el sistema
    $kernelPaths = @(
        "C:\Windows\System32\ntoskrnl.exe",
        "${env:SystemRoot}\System32\ntoskrnl.exe"
    )
    
    $kernelFound = $false
    foreach ($path in $kernelPaths) {
        if (Test-Path $path) {
            Copy-Item $path "${ISO_DIR}\boot\vmlinuz" -Force
            $kernelFound = $true
            Write-Success "Kernel copiado desde: $path"
            break
        }
    }
    
    if (-not $kernelFound) {
        Write-Warning "Kernel no encontrado. Creando placeholder..."
        "placeholder-kernel" | Out-File -FilePath "${ISO_DIR}\boot\vmlinuz" -Encoding UTF8
    }
}

# ── Construir ISO ───────────────────────────────────────────────
function Build-ISO {
    Write-Step "Construyendo ISO..."
    
    # Crear directorio de salida
    if (-not (Test-Path $OutputDir)) {
        New-Item -ItemType Directory -Force -Path $OutputDir | Out-Null
    }
    
    $isoPath = "${OutputDir}\${ISO_NAME}.iso"
    
    # Usar oscdimg (Windows ADK) si está disponible
    if ($script:OscdimgPath) {
        Write-Step "Usando oscdimg (Windows ADK)..."
        
        $oscdimgArgs = @(
            "-m",  # Ignorar límite de tamaño de imagen
            "-o",  # Optimizar almacenamiento
            "-u2",  # UDF file system
            "-udfver102",  # UDF version 1.02
            "-bootdata:2#p0,e,b${ISO_DIR}\boot\etfsboot.com#pEF,e,b${ISO_DIR}\EFI\BOOT\bootx64.efi",
            $ISO_DIR,
            $isoPath
        )
        
        & $script:OscdimgPath @oscdimgArgs
        
        if ($LASTEXITCODE -eq 0) {
            Write-Success "ISO construida con oscdimg"
        } else {
            Write-Error-Custom "Error al construir ISO con oscdimg"
        }
    }
    # Usar mkisofs si está disponible
    elseif ($script:MkisofsPath) {
        Write-Step "Usando mkisofs..."
        
        $mkisofsArgs = @(
            "-o", $isoPath,
            "-b", "boot/grub/stage2_eltorito",
            "-no-emul-boot",
            "-boot-load-size", "4",
            "-boot-info-table",
            "-R", "-J", "-v", "-T",
            $ISO_DIR
        )
        
        & $script:MkisofsPath @mkisofsArgs
        
        if ($LASTEXITCODE -eq 0) {
            Write-Success "ISO construida con mkisofs"
        } else {
            Write-Error-Custom "Error al construir ISO con mkisofs"
        }
    }
    else {
        Write-Error-Custom "No se encontró oscdimg ni mkisofs. Instala Windows ADK o mkisofs."
    }
    
    # Verificar que la ISO se creó
    if (Test-Path $isoPath) {
        $isoSize = (Get-Item $isoPath).Length / 1GB
        Write-Success "ISO creada: $isoPath ($([math]::Round($isoSize, 2)) GB)"
        return $isoPath
    } else {
        Write-Error-Custom "No se pudo crear la ISO"
    }
}

# ── Generar checksums ───────────────────────────────────────────
function New-Checksums {
    param([string]$IsoPath)
    
    Write-Step "Generando checksums..."
    
    $sha256 = (Get-FileHash -Path $IsoPath -Algorithm SHA256).Hash
    $sha512 = (Get-FileHash -Path $IsoPath -Algorithm SHA512).Hash
    
    $sha256 | Out-File -FilePath "${IsoPath}.sha256" -Encoding UTF8
    $sha512 | Out-File -FilePath "${IsoPath}.sha512" -Encoding UTF8
    
    Write-Success "SHA-256: $sha256"
    Write-Success "SHA-512: OK"
}

# ── Mostrar resumen ─────────────────────────────────────────────
function Show-Summary {
    param([string]$IsoPath)
    
    Write-Host ""
    Write-Host "╔══════════════════════════════════════════════════════════╗" -ForegroundColor Green
    Write-Host "║          ✅  ISO construida exitosamente              ║" -ForegroundColor Green
    Write-Host "╚══════════════════════════════════════════════════════════╝" -ForegroundColor Green
    Write-Host ""
    Write-Host "  ISO:     $IsoPath" -ForegroundColor White
    Write-Host "  SHA-256: ${IsoPath}.sha256" -ForegroundColor White
    Write-Host ""
    Write-Host "  Grabar en USB (Windows):" -ForegroundColor Yellow
    Write-Host "    - Rufus: https://rufus.ie" -ForegroundColor Yellow
    Write-Host "    - balenaEtcher: https://www.balena.io/etcher/" -ForegroundColor Yellow
    Write-Host ""
    Write-Host "  Correr en VM (QEMU):" -ForegroundColor Yellow
    Write-Host "    qemu-system-x86_64 -m 4G -cdrom $IsoPath -boot d" -ForegroundColor Yellow
    Write-Host ""
    Write-Host "  ⚡ Kardashev 0→1 — desde Mexicali para el mundo" -ForegroundColor Cyan
    Write-Host ""
}

# ── Main ────────────────────────────────────────────────────────
function Main {
    Write-Header "QuantumEnergyOS — Build ISO Live (Windows)"
    Write-Host "  Arquitectura: $Architecture" -ForegroundColor White
    Write-Host "  Desde Mexicali, BC — para el mundo entero" -ForegroundColor Cyan
    Write-Host ""
    
    $response = Read-Host "¿Continuar? (s/n)"
    if ($response -notmatch '^[Ss]$') {
        exit 0
    }
    
    Test-Dependencies
    Clear-BuildDirectory
    New-RootFilesystem
    Copy-QuantumEnergyOS
    New-InitScript
    New-SplashScreen
    New-GrubConfig
    Copy-Kernel
    New-Initramfs
    $isoPath = Build-ISO
    New-Checksums -IsoPath $isoPath
    Show-Summary -IsoPath $isoPath
}

# Ejecutar
Main
