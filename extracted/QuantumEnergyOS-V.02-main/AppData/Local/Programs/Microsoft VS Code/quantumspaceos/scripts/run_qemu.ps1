# QuantumSpaceOS QEMU Launch Script
# PowerShell script to run QuantumSpaceOS in QEMU
# Author: Giovanny Corpus Bernal - Mexicali, Baja California

param(
    [int]$MemoryGB = 4,
    [int]$CPUCores = 2,
    [string]$ISOPath = "iso\out\quantumspaceos-latest.iso",
    [switch]$EnableKVM,
    [switch]$NoGraphics
)

$ErrorActionPreference = "Stop"

Write-Host "========================================" -ForegroundColor Cyan
Write-Host "  QuantumSpaceOS QEMU Launcher" -ForegroundColor Cyan
Write-Host "========================================" -ForegroundColor Cyan
Write-Host ""

$ProjectRoot = Split-Path -Parent $MyInvocation.MyCommand.Path
$ProjectRoot = (Get-Item $ProjectRoot).FullName
$ISOFullPath = Join-Path $ProjectRoot $ISOPath

if (!(Test-Path $ISOFullPath)) {
    Write-Host "[ERROR] ISO no encontrada: $ISOFullPath" -ForegroundColor Red
    Write-Host "Primero ejecuta: .\scripts\build_iso.ps1" -ForegroundColor Yellow
    exit 1
}

Write-Host "[INFO] Configuración de QEMU:" -ForegroundColor Yellow
Write-Host "  ISO: $ISOFullPath" -ForegroundColor Gray
Write-Host "  Memoria: ${MemoryGB}GB" -ForegroundColor Gray
Write-Host "  CPUs: $CPUCores" -ForegroundColor Gray
Write-Host ""

$qemuArgs = @(
    "qemu-system-x86_64",
    "-m", "$MemoryGB",
    "-smp", "$CPUCores",
    "-cdrom", "`"$ISOFullPath`"",
    "-boot", "d",
    "-display", "gtk"
)

if ($EnableKVM) {
    Write-Host "[INFO] KVM habilitado" -ForegroundColor Green
    $qemuArgs += "-enable-kvm"
}

if ($NoGraphics) {
    $qemuArgs += "-nographic"
}

Write-Host "[INFO] Ejecutando QEMU..." -ForegroundColor Green
Write-Host ""

$qemuArgsStr = $qemuArgs -join " "
Write-Host "Comando: $qemuArgsStr" -ForegroundColor Gray
Write-Host ""

Write-Host "En QEMU, ejecuta:" -ForegroundColor Cyan
Write-Host "  1. mount -t proc /proc /proc" -ForegroundColor Gray
Write-Host "  2. /opt/quantumspaceos/scripts/flight-sim.sh" -ForegroundColor Gray
Write-Host "  3. python3 /opt/quantumspaceos/api/telemetry_api.py" -ForegroundColor Gray
Write-Host ""

Write-Host "Presiona Ctrl+C para salir" -ForegroundColor Yellow
Write-Host ""

& "qemu-system-x86_64" @qemuArgs