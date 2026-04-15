# ═══════════════════════════════════════════════════════════════════════
#  Build-ISO.ps1 — Script PowerShell para construir la ISO en Windows
#  QuantumEnergyOS V.02
#
#  REQUISITOS:
#    - Windows 10/11 con WSL2 (Ubuntu 22.04 o 24.04)
#    - Docker Desktop (alternativa sin WSL)
#    - Git for Windows
#
#  USO:
#    Set-ExecutionPolicy Bypass -Scope Process -Force
#    .\scripts\Build-ISO.ps1
#
#  O con opciones:
#    .\scripts\Build-ISO.ps1 -Method WSL -Arch amd64 -OutputDir C:\QEOS\dist
# ═══════════════════════════════════════════════════════════════════════

#Requires -Version 5.1
[CmdletBinding()]
param(
    [ValidateSet("WSL", "Docker", "QEMU")]
    [string]$Method = "Auto",
    [ValidateSet("amd64", "arm64")]
    [string]$Arch = "amd64",
    [string]$OutputDir = "$env:USERPROFILE\QuantumEnergyOS-dist",
    [switch]$SkipDownload,
    [switch]$TestInQEMU
)

$ErrorActionPreference = "Stop"
$QEOS_VERSION = "0.2.0"
$REPO_URL     = "https://github.com/GioCorpus/QuantumEnergyOS.git"
$ISO_NAME     = "QuantumEnergyOS-v$QEOS_VERSION-$Arch.iso"

function Write-Banner {
    Write-Host ""
    Write-Host "  ╔══════════════════════════════════════════════════════════╗" -ForegroundColor Cyan
    Write-Host "  ║       ⚡  QuantumEnergyOS V.02 — Build ISO (Windows)    ║" -ForegroundColor Cyan
    Write-Host "  ║       Arch: $Arch  |  Método: $Method  |  MIT            ║" -ForegroundColor Cyan
    Write-Host "  ╚══════════════════════════════════════════════════════════╝" -ForegroundColor Cyan
    Write-Host ""
}

function Write-OK    { Write-Host "  [✓] $args" -ForegroundColor Green  }
function Write-Warn  { Write-Host "  [⚠] $args" -ForegroundColor Yellow }
function Write-Fail  { Write-Host "  [✗] $args" -ForegroundColor Red; exit 1 }
function Write-Step  { Write-Host "`n  ── $args ──" -ForegroundColor Cyan }

# ── Detectar método disponible ───────────────────────────────────────
function Get-BuildMethod {
    if ($Method -ne "Auto") { return $Method }

    # Detectar WSL2
    try {
        $wslVersion = wsl --status 2>$null
        if ($LASTEXITCODE -eq 0) {
            $distros = wsl --list --quiet 2>$null
            if ($distros -match "Ubuntu") {
                Write-OK "WSL2 con Ubuntu detectado"
                return "WSL"
            }
        }
    } catch {}

    # Detectar Docker Desktop
    if (Get-Command docker -ErrorAction SilentlyContinue) {
        try {
            docker info 2>$null | Out-Null
            if ($LASTEXITCODE -eq 0) {
                Write-OK "Docker Desktop detectado"
                return "Docker"
            }
        } catch {}
    }

    Write-Fail "No se detectó WSL2 ni Docker. Instala uno de los dos."
}

# ── Método 1: WSL2 ───────────────────────────────────────────────────
function Build-Via-WSL {
    Write-Step "Construyendo ISO via WSL2 (Ubuntu)"

    # Verificar Ubuntu en WSL
    $distro = wsl --list --quiet 2>$null | Where-Object { $_ -match "Ubuntu" } | Select-Object -First 1
    if (-not $distro) { Write-Fail "Ubuntu no encontrado en WSL2. Instálalo: wsl --install -d Ubuntu" }

    Write-OK "Usando distro: $distro"

    # Crear directorio de output
    New-Item -ItemType Directory -Force -Path $OutputDir | Out-Null

    # Convertir path Windows a path WSL
    $wslOutput = wsl wslpath -u "$OutputDir" 2>$null

    # Script de build en WSL
    $buildScript = @"
#!/bin/bash
set -euo pipefail

echo '⚡ [QEOS] Iniciando build en WSL2...'

# Instalar dependencias Arch Linux tools en Ubuntu
if ! command -v mkarchiso &>/dev/null; then
    echo '[QEOS] Instalando archiso...'
    sudo apt-get update -qq
    sudo apt-get install -y --no-install-recommends \
        arch-install-scripts \
        squashfs-tools \
        xorriso \
        mtools \
        dosfstools \
        libisoburn1 \
        grub-common \
        grub-pc-bin \
        grub-efi-amd64-bin \
        qemu-system-x86_64 2>/dev/null || true
fi

# Clonar repo si no existe
if [ ! -d ~/QuantumEnergyOS ]; then
    echo '[QEOS] Clonando repositorio...'
    git clone $REPO_URL ~/QuantumEnergyOS --depth 1
else
    cd ~/QuantumEnergyOS && git pull origin main
fi

cd ~/QuantumEnergyOS

# Construir ISO con el script de build
chmod +x scripts/build-iso.sh
sudo ./scripts/build-iso.sh $Arch

# Copiar ISO al output de Windows
ISO_FILE=\$(ls QuantumEnergyOS-*.iso 2>/dev/null | head -1)
if [ -z "\$ISO_FILE" ]; then
    echo '✗ No se encontró la ISO generada'
    exit 1
fi

cp "\$ISO_FILE" "$wslOutput/"
cp "\$ISO_FILE.sha256" "$wslOutput/" 2>/dev/null || true
echo "✅ ISO copiada a: $wslOutput/\$ISO_FILE"
echo "   Tamaño: \$(du -h "\$ISO_FILE" | cut -f1)"
"@

    # Ejecutar en WSL
    $tempScript = "$env:TEMP\qeos_build.sh"
    $buildScript | Set-Content $tempScript -Encoding UTF8
    $wslTempScript = wsl wslpath -u $tempScript 2>$null

    Write-OK "Ejecutando build en WSL..."
    wsl bash $wslTempScript

    if ($LASTEXITCODE -ne 0) {
        Write-Fail "Error durante el build en WSL"
    }

    $isoPath = Join-Path $OutputDir $ISO_NAME
    if (Test-Path $isoPath) {
        Write-OK "ISO construida: $isoPath"
        return $isoPath
    } else {
        Write-Warn "ISO no encontrada en $OutputDir — verificar output de WSL"
        return $null
    }
}

# ── Método 2: Docker ─────────────────────────────────────────────────
function Build-Via-Docker {
    Write-Step "Construyendo ISO via Docker"

    New-Item -ItemType Directory -Force -Path $OutputDir | Out-Null

    # Usar imagen Docker de archiso
    $dockerCmd = @"
docker run --rm --privileged `
  -v "${OutputDir}:/output" `
  -v "$(Get-Location):/src:ro" `
  archlinux:latest `
  bash -c "
    pacman -Sy --noconfirm archiso squashfs-tools xorriso grub mtools &&
    cp -r /src /build &&
    cd /build &&
    chmod +x scripts/build-iso.sh &&
    ./scripts/build-iso.sh $Arch &&
    cp QuantumEnergyOS-*.iso /output/ &&
    cp QuantumEnergyOS-*.iso.sha256 /output/ 2>/dev/null || true
  "
"@

    Write-OK "Comando Docker:"
    Write-Host "  $dockerCmd" -ForegroundColor Gray

    Invoke-Expression $dockerCmd

    $isoPath = Join-Path $OutputDir $ISO_NAME
    if (Test-Path $isoPath) {
        Write-OK "ISO construida via Docker: $isoPath"
        return $isoPath
    }
    Write-Fail "Error: ISO no generada por Docker"
}

# ── Probar ISO en QEMU ───────────────────────────────────────────────
function Test-ISO-QEMU {
    param([string]$ISOPath)

    Write-Step "Probando ISO en QEMU"

    # Buscar QEMU en Windows
    $qemuPaths = @(
        "C:\Program Files\qemu\qemu-system-x86_64.exe",
        "C:\qemu\qemu-system-x86_64.exe",
        "$env:ProgramFiles\QEMU\qemu-system-x86_64.exe"
    )

    $qemuExe = $null
    foreach ($p in $qemuPaths) {
        if (Test-Path $p) { $qemuExe = $p; break }
    }

    if (-not $qemuExe) {
        if (Get-Command qemu-system-x86_64 -ErrorAction SilentlyContinue) {
            $qemuExe = "qemu-system-x86_64"
        }
    }

    if (-not $qemuExe) {
        Write-Warn "QEMU no encontrado. Instala desde: https://www.qemu.org/download/#windows"
        Write-Warn "O usa: winget install qemu"
        Write-Warn "Para probar manualmente:"
        Write-Host "  qemu-system-x86_64 -m 4G -cdrom `"$ISOPath`" -boot d -vga virtio" -ForegroundColor Yellow
        return
    }

    Write-OK "QEMU encontrado: $qemuExe"
    Write-OK "Lanzando VM con 4 GB de RAM..."
    Write-Host "  ISO: $ISOPath" -ForegroundColor Gray

    $qemuArgs = @(
        "-m", "4G",
        "-cdrom", "`"$ISOPath`"",
        "-boot", "d",
        "-vga", "virtio",
        "-enable-kvm",
        "-cpu", "host",
        "-smp", "4"
    )

    # En Windows KVM no disponible — usar TCG
    $qemuArgsFallback = @(
        "-m", "4G",
        "-cdrom", "`"$ISOPath`"",
        "-boot", "d",
        "-vga", "virtio",
        "-cpu", "max",
        "-smp", "4"
    )

    try {
        & $qemuExe @qemuArgs
    } catch {
        Write-Warn "KVM no disponible — usando emulación TCG (más lento)..."
        & $qemuExe @qemuArgsFallback
    }
}

# ── Generar checksums ────────────────────────────────────────────────
function Write-Checksums {
    param([string]$ISOPath)

    Write-Step "Generando checksums"
    $sha256 = Get-FileHash -Path $ISOPath -Algorithm SHA256
    $sha512 = Get-FileHash -Path $ISOPath -Algorithm SHA512

    "$($sha256.Hash.ToLower())  $ISO_NAME" | Set-Content "$ISOPath.sha256"
    "$($sha512.Hash.ToLower())  $ISO_NAME" | Set-Content "$ISOPath.sha512"

    Write-OK "SHA-256: $($sha256.Hash.Substring(0,16).ToLower())..."
    Write-OK "SHA-512: $($sha512.Hash.Substring(0,16).ToLower())..."
}

# ── Resumen final ────────────────────────────────────────────────────
function Write-Summary {
    param([string]$ISOPath)

    $size = if (Test-Path $ISOPath) {
        "$([math]::Round((Get-Item $ISOPath).Length / 1MB, 0)) MB"
    } else { "N/A" }

    Write-Host ""
    Write-Host "  ╔══════════════════════════════════════════════════════╗" -ForegroundColor Green
    Write-Host "  ║    ✅  QuantumEnergyOS V.02 — ISO lista              ║" -ForegroundColor Green
    Write-Host "  ╚══════════════════════════════════════════════════════╝" -ForegroundColor Green
    Write-Host ""
    Write-Host "  ISO:      $ISOPath" -ForegroundColor White
    Write-Host "  Tamaño:   $size" -ForegroundColor White
    Write-Host "  SHA-256:  $ISOPath.sha256" -ForegroundColor White
    Write-Host ""
    Write-Host "  Grabar en USB:" -ForegroundColor White
    Write-Host "    balenaEtcher:  https://www.balena.io/etcher" -ForegroundColor Yellow
    Write-Host "    Rufus:         https://rufus.ie" -ForegroundColor Yellow
    Write-Host ""
    Write-Host "  O probar en VM:" -ForegroundColor White
    Write-Host "    .\scripts\Build-ISO.ps1 -TestInQEMU" -ForegroundColor Yellow
    Write-Host ""
    Write-Host "  ⚡ Desde Mexicali, BC — Kardashev 0→1" -ForegroundColor Cyan
    Write-Host ""
}

# ── MAIN ─────────────────────────────────────────────────────────────
Write-Banner

$actualMethod = Get-BuildMethod
Write-OK "Método de build: $actualMethod"
Write-OK "Arquitectura: $Arch"
Write-OK "Output: $OutputDir"

$isoPath = switch ($actualMethod) {
    "WSL"    { Build-Via-WSL    }
    "Docker" { Build-Via-Docker }
    default  { Write-Fail "Método no soportado: $actualMethod" }
}

if ($isoPath -and (Test-Path $isoPath)) {
    Write-Checksums -ISOPath $isoPath
    Write-Summary   -ISOPath $isoPath

    if ($TestInQEMU) {
        Test-ISO-QEMU -ISOPath $isoPath
    }
} else {
    Write-Warn "Build completado pero ISO no encontrada en path esperado"
    Write-Warn "Verificar: $OutputDir"
}
