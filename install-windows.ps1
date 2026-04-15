# ══════════════════════════════════════════════════════════════════════
#  QuantumEnergyOS — Instalador Windows (PowerShell)
#  Compatible: Windows 10, 11, 12 (futuro) | x64 + ARM64
#  Ejecutar desde PowerShell como Administrador:
#    Set-ExecutionPolicy Bypass -Scope Process -Force
#    .\install-windows.ps1
#  O instalación remota:
#    iex (iwr https://raw.githubusercontent.com/GioCorpus/QuantumEnergyOS/main/install-windows.ps1)
# ══════════════════════════════════════════════════════════════════════

#Requires -Version 5.1
[CmdletBinding()]
param(
    [string]$InstallDir = "$env:LOCALAPPDATA\QuantumEnergyOS",
    [switch]$NoVenv,
    [switch]$Silent
)

$ErrorActionPreference = "Stop"
$QEOS_VERSION = "1.0.0"
$QEOS_REPO = "https://github.com/GioCorpus/QuantumEnergyOS"
$MIN_PYTHON = [Version]"3.11.0"

# ── Banner ────────────────────────────────────────────────────
function Show-Banner {
    Write-Host ""
    Write-Host "  ╔══════════════════════════════════════════════════════════╗" -ForegroundColor Cyan
    Write-Host "  ║          ⚡  Q U A N T U M  E N E R G Y  O S  ⚡        ║" -ForegroundColor Cyan
    Write-Host "  ║        Desde Mexicali, BC — para el mundo entero         ║" -ForegroundColor Cyan
    Write-Host "  ║      Windows 10 / 11 / 12  |  MIT License                ║" -ForegroundColor Cyan
    Write-Host "  ╚══════════════════════════════════════════════════════════╝" -ForegroundColor Cyan
    Write-Host ""
}

function Write-OK    { Write-Host "[✓] $args" -ForegroundColor Green }
function Write-Warn  { Write-Host "[⚠] $args" -ForegroundColor Yellow }
function Write-Fail  { Write-Host "[✗] $args" -ForegroundColor Red; exit 1 }
function Write-Step  { Write-Host "`n── $args ──" -ForegroundColor Cyan }

# ── Detectar arquitectura Windows ────────────────────────────
function Get-WindowsArch {
    $arch = $env:PROCESSOR_ARCHITECTURE
    $armCheck = [System.Environment]::GetEnvironmentVariable("PROCESSOR_ARCHITEW6432")

    if ($arch -eq "ARM64" -or $armCheck -eq "ARM64") {
        Write-OK "Arquitectura: ARM64 (compatible con Snapdragon X Elite / Surface Pro)"
    } elseif ($arch -eq "AMD64") {
        Write-OK "Arquitectura: x64 (Intel/AMD)"
    } else {
        Write-Warn "Arquitectura: $arch — puede haber limitaciones"
    }

    # Versión de Windows
    $winVer = [System.Environment]::OSVersion.Version
    $winName = (Get-WmiObject Win32_OperatingSystem).Caption
    Write-OK "SO: $winName (Build $($winVer.Build))"
}

# ── Verificar / Instalar Python ───────────────────────────────
function Get-PythonBin {
    Write-Step "Verificando Python $MIN_PYTHON+"

    # Buscar Python en el sistema
    $pythonCmds = @("python3.12", "python3.11", "python3", "python", "py")
    foreach ($cmd in $pythonCmds) {
        try {
            $ver = & $cmd -c "import sys; print(f'{sys.version_info.major}.{sys.version_info.minor}.{sys.version_info.micro}')" 2>$null
            if ($ver) {
                $v = [Version]$ver
                if ($v -ge $MIN_PYTHON) {
                    Write-OK "Python encontrado: $cmd ($ver)"
                    return $cmd
                }
            }
        } catch {}
    }

    # No encontrado — instalar con winget
    Write-Warn "Python $MIN_PYTHON+ no encontrado. Instalando via winget..."
    Install-Python
    return "python"
}

function Install-Python {
    # Intentar winget primero
    if (Get-Command winget -ErrorAction SilentlyContinue) {
        Write-OK "Instalando Python 3.11 via winget..."
        winget install --id Python.Python.3.11 --silent --accept-package-agreements --accept-source-agreements
        $env:Path = [System.Environment]::GetEnvironmentVariable("Path", "Machine") + ";" + [System.Environment]::GetEnvironmentVariable("Path", "User")
        return
    }

    # Fallback: Chocolatey
    if (Get-Command choco -ErrorAction SilentlyContinue) {
        Write-OK "Instalando Python 3.11 via Chocolatey..."
        choco install python311 -y
        return
    }

    # Fallback: Scoop
    if (Get-Command scoop -ErrorAction SilentlyContinue) {
        Write-OK "Instalando Python via Scoop..."
        scoop install python
        return
    }

    # Descarga directa
    Write-OK "Descargando Python 3.11 desde python.org..."
    $arch = if ($env:PROCESSOR_ARCHITECTURE -eq "ARM64") { "arm64" } else { "amd64" }
    $pyUrl = "https://www.python.org/ftp/python/3.11.9/python-3.11.9-$arch.exe"
    $pyInstaller = "$env:TEMP\python-installer.exe"

    Invoke-WebRequest -Uri $pyUrl -OutFile $pyInstaller -UseBasicParsing
    Start-Process -FilePath $pyInstaller -ArgumentList "/quiet", "InstallAllUsers=0", "PrependPath=1", "Include_test=0" -Wait
    Remove-Item $pyInstaller -Force
    $env:Path = [System.Environment]::GetEnvironmentVariable("Path", "User") + ";" + $env:Path
}

# ── Instalar Git si no existe ─────────────────────────────────
function Ensure-Git {
    if (-not (Get-Command git -ErrorAction SilentlyContinue)) {
        Write-Warn "Git no encontrado. Instalando..."
        if (Get-Command winget -ErrorAction SilentlyContinue) {
            winget install --id Git.Git --silent --accept-package-agreements
        } elseif (Get-Command choco -ErrorAction SilentlyContinue) {
            choco install git -y
        } else {
            Write-Warn "Instala Git desde https://git-scm.com/download/win"
        }
    } else {
        Write-OK "Git: $(git --version)"
    }
}

# ── Crear entorno virtual e instalar QEOS ────────────────────
function Install-QEOS {
    param([string]$PythonBin)

    Write-Step "Instalando QuantumEnergyOS v$QEOS_VERSION"

    # Crear directorio de instalación
    New-Item -ItemType Directory -Force -Path $InstallDir | Out-Null
    Set-Location $InstallDir

    # Clonar o actualizar repo
    if (-not (Test-Path "$InstallDir\.git")) {
        Write-OK "Clonando repositorio..."
        git clone $QEOS_REPO . --depth 1
    } else {
        Write-OK "Actualizando repositorio..."
        git pull origin main
    }

    # Crear entorno virtual
    Write-OK "Creando entorno virtual..."
    & $PythonBin -m venv venv
    $venvPip = "$InstallDir\venv\Scripts\pip.exe"
    $venvPython = "$InstallDir\venv\Scripts\python.exe"

    & $venvPip install --upgrade pip --quiet

    # Instalar dependencias
    if (Test-Path "requirements-pinned.txt") {
        Write-OK "Instalando dependencias pinadas (0 CVEs)..."
        & $venvPip install -r requirements-pinned.txt --quiet
    } elseif (Test-Path "requirements.txt") {
        & $venvPip install -r requirements.txt --quiet
    }

    # Intentar instalar QDK
    & $venvPip install qsharp --quiet 2>$null
    if ($LASTEXITCODE -eq 0) { Write-OK "QDK (Q#): instalado" } else { Write-Warn "QDK: no disponible (opcional)" }

    Write-OK "Instalado en: $InstallDir"
    return $venvPython
}

# ── Crear acceso directo en el escritorio ─────────────────────
function Create-Shortcut {
    param([string]$VenvPython)

    Write-Step "Creando accesos directos"

    # Script de inicio .bat
    $launcherBat = "$InstallDir\qeos.bat"
    @"
@echo off
call "$InstallDir\venv\Scripts\activate.bat"
python -m api.server %*
"@ | Set-Content $launcherBat

    # Acceso directo en escritorio
    $desktopPath = [Environment]::GetFolderPath("Desktop")
    $shortcut = (New-Object -ComObject WScript.Shell).CreateShortcut("$desktopPath\QuantumEnergyOS.lnk")
    $shortcut.TargetPath = $launcherBat
    $shortcut.WorkingDirectory = $InstallDir
    $shortcut.Description = "QuantumEnergyOS - Energía Cuántica"
    $shortcut.Save()
    Write-OK "Acceso directo creado en el escritorio"

    # Agregar al PATH del usuario
    $userPath = [Environment]::GetEnvironmentVariable("Path", "User")
    if ($userPath -notlike "*$InstallDir*") {
        [Environment]::SetEnvironmentVariable("Path", "$userPath;$InstallDir", "User")
        Write-OK "Agregado al PATH del usuario"
    }
}

# ── Windows Defender exclusión ────────────────────────────────
function Add-DefenderExclusion {
    try {
        Add-MpPreference -ExclusionPath $InstallDir -ErrorAction SilentlyContinue
        Write-OK "Exclusión de Windows Defender agregada para $InstallDir"
    } catch {
        Write-Warn "No se pudo agregar exclusión de Defender (requiere admin)"
    }
}

# ── Verificación final ────────────────────────────────────────
function Test-Installation {
    param([string]$VenvPython)

    Write-Step "Verificando instalación"

    $tests = @(
        @{ Pkg = "qiskit"; Label = "Qiskit" },
        @{ Pkg = "fastapi"; Label = "FastAPI" },
        @{ Pkg = "pydantic"; Label = "Pydantic" },
        @{ Pkg = "numpy"; Label = "NumPy" }
    )

    foreach ($t in $tests) {
        $ver = & $VenvPython -c "import $($t.Pkg); print($($t.Pkg).__version__)" 2>$null
        if ($ver) { Write-OK "$($t.Label): $ver" } else { Write-Warn "$($t.Label): no disponible" }
    }
}

# ── Resumen ───────────────────────────────────────────────────
function Show-Success {
    Write-Host ""
    Write-Host "  ╔══════════════════════════════════════════════════════╗" -ForegroundColor Green
    Write-Host "  ║    ✅  QuantumEnergyOS instalado en Windows          ║" -ForegroundColor Green
    Write-Host "  ╚══════════════════════════════════════════════════════╝" -ForegroundColor Green
    Write-Host ""
    Write-Host "  Activar entorno:" -ForegroundColor White
    Write-Host "    $InstallDir\venv\Scripts\activate" -ForegroundColor Yellow
    Write-Host ""
    Write-Host "  Iniciar servidor:" -ForegroundColor White
    Write-Host "    uvicorn api.server:app --reload --port 8000" -ForegroundColor Yellow
    Write-Host ""
    Write-Host "  Acceso directo en el Escritorio: QuantumEnergyOS.lnk" -ForegroundColor White
    Write-Host ""
    Write-Host "  ⚡ Desde Mexicali, BC — para el mundo entero. Kardashev 0→1" -ForegroundColor Cyan
    Write-Host ""
}

# ── Main ──────────────────────────────────────────────────────
Show-Banner
Get-WindowsArch
Ensure-Git
$pythonBin  = Get-PythonBin
$venvPython = Install-QEOS -PythonBin $pythonBin
Add-DefenderExclusion
Create-Shortcut -VenvPython $venvPython
Test-Installation -VenvPython $venvPython
Show-Success
