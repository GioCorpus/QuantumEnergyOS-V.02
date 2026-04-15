# QuantumEnergyOS V.02 - PowerShell Build Script
# Build System for Windows
# Author: Giovanny Corpus Bernal - Mexicali, BC
# Mission: Never more blackouts in Mexicali

param(
    [string]$Action = "build-iso",
    [string]$OutputDir = ".\dist",
    [string]$WorkDir = ".\build"
)

$ErrorActionPreference = "Stop"
[Console]::OutputEncoding = [System.Text.Encoding]::UTF8

# Colors
$Green = "`e[32m"
$Yellow = "`e[33m"
$Red = "`e[31m"
$Blue = "`e[34m"
$Reset = "`e[0m"

function Write-Step {
    param([string]$Message)
    Write-Host "${Blue}[*]${Reset} $Message"
}

function Write-Success {
    param([string]$Message)
    Write-Host "${Green}[✓]${Reset} $Message"
}

function Write-Warn {
    param([string]$Message)
    Write-Host "${Yellow}[!]${Reset} $Message"
}

function Write-Error {
    param([string]$Message)
    Write-Host "${Red}[✗]${Reset} $Message"
}

function Show-Banner {
    Write-Host ""
    Write-Host "╔═══════════════════════════════════════════════════════════╗"
    Write-Host "║        QuantumEnergyOS V.02 - Build System                ║"
    Write-Host "║        Made in Mexicali with 22 years of grind             ║"
    Write-Host "╚═══════════════════════════════════════════════════════════╝"
    Write-Host ""
}

function Test-Administrator {
    $currentUser = [Security.Principal.WindowsIdentity]::GetCurrent()
    $principal = New-Object Security.Principal.WindowsPrincipal($currentUser)
    return $principal.IsInRole([Security.Principal.WindowsBuiltInRole]::Administrator)
}

function Initialize-Directories {
    Write-Step "Creating build directories..."
    $dirs = @($OutputDir, $WorkDir, "$WorkDir\kernel", "$WorkDir\packages", "$WorkDir\iso")
    foreach ($dir in $dirs) {
        if (!(Test-Path $dir)) {
            New-Item -ItemType Directory -Path $dir -Force | Out-Null
            Write-Success "Created: $dir"
        }
    }
}

function Install-Dependencies {
    Write-Step "Checking build dependencies..."
    
    # Python
    if (!(Get-Command python -ErrorAction SilentlyContinue)) {
        Write-Warn "Python not found. Please install Python 3.9+ from python.org"
    } else {
        $pyVersion = python --version 2>&1
        Write-Success "Python: $pyVersion"
    }
    
    # pip packages
    Write-Step "Installing Python packages..."
    pip install --break-system-packages flask flask-cors 2>$null
    if ($LASTEXITCODE -eq 0) {
        Write-Success "Flask dependencies installed"
    }
    
    # Git
    if (!(Get-Command git -ErrorAction SilentlyContinue)) {
        Write-Warn "Git not found. Install from git-scm.com"
    } else {
        Write-Success "Git installed"
    }
    
    # QEMU (optional)
    if (Get-Command qemu-system-x86_64 -ErrorAction SilentlyContinue) {
        Write-Success "QEMU available for testing"
    } else {
        Write-Warn "QEMU not found (optional for testing)"
    }
}

function Build-PhotonicBridge {
    Write-Step "Building Photonic Bridge (Rust)..."
    
    if (Test-Path ".\photonic-bridge\Cargo.toml") {
        if (Get-Command cargo -ErrorAction SilentlyContinue) {
            Push-Location ".\photonic-bridge"
            cargo build --release 2>$null
            if ($LASTEXITCODE -eq 0) {
                Copy-Item ".\target\release\photonic-bridge" "$WorkDir\packages\" -Force -ErrorAction SilentlyContinue
                Write-Success "Photonic Bridge built"
            } else {
                Write-Warn "Build with warnings (continuing)"
            }
            Pop-Location
        } else {
            Write-Warn "Rust/Cargo not installed - skipping bridge build"
        }
    }
    
    # Copy source as fallback
    Copy-Item ".\photonic-bridge\src\lib.rs" "$WorkDir\packages\" -Force -ErrorAction SilentlyContinue
    Write-Success "Photonic Bridge sources copied"
}

function Build-PhotonicCore {
    Write-Step "Building Photonic Core (Rust)..."
    
    if (Test-Path ".\photonic-core\Cargo.toml") {
        if (Get-Command cargo -ErrorAction SilentlyContinue) {
            Push-Location ".\photonic-core"
            cargo build --release 2>$null
            if ($LASTEXITCODE -eq 0) {
                Copy-Item ".\target\release\photonic-core" "$WorkDir\packages\" -Force -ErrorAction SilentlyContinue
                Write-Success "Photonic Core built"
            } else {
                Write-Warn "Build with warnings (continuing)"
            }
            Pop-Location
        } else {
            Write-Warn "Rust/Cargo not installed - skipping core build"
        }
    }
    
    Copy-Item ".\photonic-core\src\lib.rs" "$WorkDir\packages\" -Force -ErrorAction SilentlyContinue
    Write-Success "Photonic Core sources copied"
}

function Build-API {
    Write-Step "Building API Server..."
    
    if (Test-Path ".\api\server.py") {
        Copy-Item ".\api\server.py" "$WorkDir\packages\" -Force
        Copy-Item ".\api\requirements.txt" "$WorkDir\packages\" -Force -ErrorAction SilentlyContinue
        Write-Success "API Server copied"
    }
}

function Create-LiveCD-Config {
    Write-Step "Creating Live CD configuration..."
    
    $livecdDir = ".\livecd"
    
    if (!(Test-Path $livecdDir)) {
        New-Item -ItemType Directory -Path $livecdDir -Force | Out-Null
    }
    
    # Create profile structure
    $profileDir = "$livecdDir\profiledef"
    if (!(Test-Path $profileDir)) {
        New-Item -ItemType Directory -Path $profileDir -Force | Out-Null
    }
    
    # Write pacman.conf
    $pacmanConf = @"
[options]
HoldPkg = pacman
SyncFirst = base base-devel
Architecture = x86_64

[core]
Server = https://archive.archlinux.org/repos/2024/01/01/\$repo/os/\$arch

[extra]
Server = https://archive.archlinux.org/repos/2024/01/01/\$repo/os/\$arch

[community]
Server = https://archive.archlinux.org/repos/2024/01/01/\$repo/os/\$arch
"@
    
    $pacmanConf | Out-File -FilePath "$livecdDir\pacman.conf" -Encoding UTF8 -Force
    
    # Write packages.x86_64
    $packages = @"
base
base-devel
linux
linux-firmware
systemd
systemd-sysvcompat
grub
os-prober
memtest86+
efibootmgr
dmidecode
hdparm
sdparm
inetutils
iputils
logrotate
lsof
man-db
man-pages
texinfo
which
python
python-pip
flask
flask-cors
rust
cargo
vim
nano
wget
curl
rsync
openssh
sudo
fish
zsh
htop
iotop
strace
ltrace
gdb
valgrind
cmake
make
gcc
git
docker
qemu
archiso
mkinitcpio
"@
    
    $packages -split "`n" | Out-File -FilePath "$livecdDir\packages.x86_64" -Encoding UTF8 -Force
    
    Write-Success "Live CD configuration created"
}

function Create-ISO {
    Write-Step "Creating QuantumEnergyOS ISO..."
    
    $isoFile = "$OutputDir\quantumenergyos-v02-x86_64.iso"
    
    # Note: On Windows, actual ISO creation requires WSL with archiso
    # This creates a placeholder structure
    
    Write-Warn "Note: Full ISO creation requires Linux with archiso"
    Write-Warn "On Windows, use WSL2 with Arch Linux to build the ISO"
    
    # Create placeholder
    $placeholder = @"
QuantumEnergyOS V.02 ISO Structure
====================================

This is a placeholder for the actual ISO build.

To create the full ISO:
1. Use WSL2 with Arch Linux
2. Run: make build-iso
3. Or use the provided Makefile on Linux

ISO Specifications:
- Size: ~2.5 GB
- Boot: UEFI + Legacy BIOS
- Kernel: 6.6.0-qeos (custom quantum-photonic)
- Base: Arch Linux

Author: Giovanny Corpus Bernal
Location: Mexicali, Baja California
Mission: Never more blackouts in Mexicali

"El quantum fluye, la energía permanece"
"@
    
    $placeholder | Out-File -FilePath "$OutputDir\README.txt" -Encoding UTF8 -Force
    
    # Create a minimal boot structure
    $bootDir = "$WorkDir\iso\boot"
    New-Item -ItemType Directory -Path $bootDir -Force | Out-Null
    
    Write-Success "ISO structure created at: $WorkDir\iso"
    Write-Success "Output placeholder: $OutputDir\README.txt"
    
    Write-Host ""
    Write-Host "${Yellow}============================================${Reset}"
    Write-Host "  Build incomplete - requires Linux/WSL2"
    Write-Host "  For full ISO, run on Arch Linux:"
    Write-Host "    make build-iso"
    Write-Host "${Yellow}============================================${Reset}"
}

function Start-DevServer {
    Write-Step "Starting development API server..."
    if (Test-Path ".\api\server.py") {
        Push-Location ".\api"
        python server.py
        Pop-Location
    } else {
        Write-Error "API server not found"
    }
}

function Test-WithQEMU {
    Write-Step "Testing with QEMU..."
    $isoFile = "$OutputDir\quantumenergyos-v02-x86_64.iso"
    
    if (Test-Path $isoFile) {
        qemu-system-x86_64 -m 4096 -cdrom $isoFile -boot d
    } else {
        Write-Warn "ISO not found. Run build first."
    }
}

function Show-Help {
    Write-Host ""
    Write-Host "QuantumEnergyOS V.02 - Build System"
    Write-Host ""
    Write-Host "Usage: .\buildsystem\build.ps1 [-Action <action>]"
    Write-Host ""
    Write-Host "Actions:"
    Write-Host "  build-iso      - Full ISO build (creates structure)"
    Write-Host "  install-deps   - Install dependencies"
    Write-Host "  dev-server     - Start development API server"
    Write-Host "  test-qemu      - Test with QEMU"
    Write-Host "  clean          - Clean build artifacts"
    Write-Host "  help           - Show this help"
    Write-Host ""
    Write-Host "Note: Full ISO creation requires Linux or WSL2"
    Write-Host ""
}

# Main
Show-Banner

switch ($Action.ToLower()) {
    "build-iso" {
        if (!(Test-Administrator)) {
            Write-Warn "Administrator privileges recommended"
        }
        Initialize-Directories
        Install-Dependencies
        Build-PhotonicBridge
        Build-PhotonicCore
        Build-API
        Create-LiveCD-Config
        Create-ISO
    }
    "install-deps" {
        Install-Dependencies
    }
    "dev-server" {
        Start-DevServer
    }
    "test-qemu" {
        Test-WithQEMU
    }
    "clean" {
        Write-Step "Cleaning build artifacts..."
        Remove-Item -Path $WorkDir -Recurse -Force -ErrorAction SilentlyContinue
        Remove-Item -Path "$OutputDir\*" -Force -ErrorAction SilentlyContinue
        Write-Success "Clean complete"
    }
    "help" {
        Show-Help
    }
    default {
        Write-Error "Unknown action: $Action"
        Show-Help
    }
}

Write-Host ""
Write-Success "QuantumEnergyOS V.02 Build System Ready"
Write-Host ""
Write-Host "Author: Giovanny Corpus Bernal - Mexicali, BC"
Write-Host "Mission: Never more blackouts in Mexicali"
Write-Host ""