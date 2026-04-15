# ⚡ QuantumEnergyOS — Guía de Instalación Global

**Desde Mexicali, BC — para el mundo entero. Kardashev 0→1**

---

## 🗺️ Plataformas soportadas

| Plataforma | Arquitectura | Método | Estado |
|---|---|---|---|
| 🐧 Linux (todas las distros) | x86_64, arm64 | `install.sh` | ✅ Estable |
| 🍎 macOS — Apple M1 | arm64 | `install-macos.sh` | ✅ Estable |
| 🍎 macOS — Apple M2 | arm64 | `install-macos.sh` | ✅ Estable |
| 🍎 macOS — Apple M3 | arm64 | `install-macos.sh` | ✅ Estable |
| 🍎 macOS — Apple M4 | arm64 | `install-macos.sh` | ✅ Estable |
| 🍎 macOS — Intel | x86_64 | `install-macos.sh` | ✅ Estable |
| 🪟 Windows 10 | x64 | `install-windows.ps1` | ✅ Estable |
| 🪟 Windows 11 | x64, ARM64 | `install-windows.ps1` | ✅ Estable |
| 🪟 Windows 12 (futuro) | x64, ARM64 | `install-windows.ps1` | 🔜 Listo |
| ☁️ Azure (Container Apps) | linux/amd64 | `azure.bicep` | ✅ Estable |
| ☁️ Azure (ARM64/Graviton) | linux/arm64 | Docker multi-arch | ✅ Estable |
| 🐳 Docker (cualquier OS) | amd64, arm64 | `docker run` | ✅ Estable |
| 💿 Live ISO (booteable) | amd64, arm64 | `build-iso.sh` | 🔬 Beta |

---

## ⚡ Instalación rápida (1 línea)

### Linux / macOS
```bash
curl -fsSL https://raw.githubusercontent.com/GioCorpus/QuantumEnergyOS/main/install.sh | bash
```

### macOS (Apple Silicon M1/M2/M3/M4 + Intel)
```bash
curl -fsSL https://raw.githubusercontent.com/GioCorpus/QuantumEnergyOS/main/install-macos.sh | bash
```

### Windows (PowerShell como Administrador)
```powershell
Set-ExecutionPolicy Bypass -Scope Process -Force
iex (iwr https://raw.githubusercontent.com/GioCorpus/QuantumEnergyOS/main/install-windows.ps1)
```

### Docker (cualquier plataforma)
```bash
docker run -p 8000:8000 \
  -e QEOS_JWT_SECRET=$(openssl rand -hex 32) \
  ghcr.io/giocorpus/quantumenergyos:latest
```

---

## 🐧 Linux — Todas las distribuciones

### Instalación automática (detecta tu distro)
```bash
git clone https://github.com/GioCorpus/QuantumEnergyOS.git
cd QuantumEnergyOS
chmod +x install.sh
./install.sh
```

### Soporte por familia de distros

| Familia | Distros | Gestor |
|---|---|---|
| Debian/Ubuntu | Ubuntu 20/22/24, Mint, Pop!_OS, Zorin, Elementary, KDE Neon | `apt` |
| Red Hat | Fedora, RHEL 9+, CentOS Stream, AlmaLinux, Rocky | `dnf` |
| Arch | Arch Linux, Manjaro, EndeavourOS, ArcoLinux, Garuda | `pacman` |
| SUSE | openSUSE Tumbleweed, SUSE Linux Enterprise | `zypper` |
| Alpine | Alpine Linux (contenedores, servidores mínimos) | `apk` |
| NixOS | NixOS (reproducible) | `nix-shell` |

### Manual (cualquier distro con Python 3.11+)
```bash
python3 -m venv venv
source venv/bin/activate
pip install -r requirements-pinned.txt
uvicorn api.server:app --reload --port 8000
```

---

## 🍎 macOS — Apple Silicon e Intel

### Requisitos
- macOS 12 Monterey o superior
- Xcode Command Line Tools
- Homebrew (se instala automáticamente)

### Apple Silicon (M1, M2, M3, M4)
```bash
chmod +x install-macos.sh
./install-macos.sh
```
El instalador detecta automáticamente la generación del chip (M1/M2/M3/M4) y configura OpenBLAS nativo para máximo rendimiento en arm64.

### Intel (x86_64)
El mismo script funciona en Intel — detecta la arquitectura automáticamente.

### Verificar instalación
```bash
# Verificar que NumPy usa arm64 nativo (Apple Silicon)
python -c "import platform; print(platform.machine())"
# Debe imprimir: arm64

# Verificar QDK
python -c "import qsharp; print(qsharp.__version__)"
```

---

## 🪟 Windows 10 / 11 / 12

### Requisitos
- Windows 10 (1909+), Windows 11, o Windows 12
- PowerShell 5.1+ (incluido en Windows)
- Conexión a internet

### Instalación
```powershell
# Desde PowerShell como Administrador:
Set-ExecutionPolicy Bypass -Scope Process -Force
.\install-windows.ps1
```

El instalador:
- Detecta arquitectura (x64 o ARM64)
- Instala Python 3.11+ via winget/Chocolatey/descarga directa
- Instala Git si no está presente
- Crea entorno virtual en `%LOCALAPPDATA%\QuantumEnergyOS`
- Crea acceso directo en el Escritorio
- Agrega exclusión en Windows Defender

### Activar manualmente
```powershell
& "$env:LOCALAPPDATA\QuantumEnergyOS\venv\Scripts\Activate.ps1"
cd "$env:LOCALAPPDATA\QuantumEnergyOS"
uvicorn api.server:app --reload --port 8000
```

---

## ☁️ Microsoft Azure

### Opción 1: Azure Container Apps (recomendado)
```bash
# 1. Login
az login
az account set --subscription "TU_SUBSCRIPTION_ID"

# 2. Crear Resource Group en México Central
az group create --name qeos-rg --location mexicocentral

# 3. Desplegar con Bicep
az deployment group create \
  --resource-group qeos-rg \
  --template-file azure.bicep \
  --parameters jwtSecret="$(openssl rand -hex 32)"

# 4. Obtener URL
az containerapp show \
  --name quantumenergyos-api \
  --resource-group qeos-rg \
  --query properties.configuration.ingress.fqdn -o tsv
```

### Opción 2: Azure Quantum (hardware cuántico real)
```bash
# Configurar workspace de Azure Quantum
az quantum workspace create \
  --resource-group qeos-rg \
  --workspace-name qeos-quantum \
  --location mexicocentral \
  --provider-sku-list "Microsoft/free,ionq/pay-as-you-go-credits"
```

### Opción 3: Docker en Azure VM
```bash
# VM Ubuntu en Azure
az vm create \
  --resource-group qeos-rg \
  --name qeos-vm \
  --image Ubuntu2404 \
  --size Standard_D4s_v3 \
  --admin-username quantum \
  --generate-ssh-keys

# SSH + Docker
ssh quantum@<VM_IP>
curl -fsSL https://get.docker.com | bash
docker run -d -p 8000:8000 \
  -e QEOS_JWT_SECRET=$(openssl rand -hex 32) \
  --restart unless-stopped \
  ghcr.io/giocorpus/quantumenergyos:latest
```

---

## 💿 ISO Booteable (para el mundo entero)

Genera una ISO live que bootea directamente en QuantumEnergyOS:

```bash
# Requisitos: Ubuntu 22.04+ con acceso root
sudo apt-get install -y live-build squashfs-tools xorriso

# Build ISO amd64 (x86_64 — PCs estándar)
sudo ./build-iso.sh amd64

# Build ISO arm64 (Apple M1/M2/M3/M4 via UTM, Raspberry Pi, servidores ARM)
sudo ./build-iso.sh arm64
```

**Output:** `QuantumEnergyOS-v1.0.0-live-amd64.iso` (~2.5 GB)

### Grabar en USB
```bash
# Linux/macOS
sudo dd if=QuantumEnergyOS-v1.0.0-live-amd64.iso of=/dev/sdX bs=4M status=progress

# Windows: usar Rufus (https://rufus.ie) o balenaEtcher
```

### Correr en VM (sin USB)
```bash
# QEMU/KVM
qemu-system-x86_64 -m 4G -cdrom QuantumEnergyOS-v1.0.0-live-amd64.iso -boot d

# Apple Silicon (M1/M2/M3/M4) — usar UTM
# https://mac.getutm.app/ → Nueva VM → Virtualizar → Linux → ISO arm64
```

---

## ✅ Verificar instalación (todas las plataformas)

```bash
source venv/bin/activate   # Linux/macOS
# o: .\venv\Scripts\activate  (Windows)

# API
curl http://localhost:8000/health

# Qiskit
python -c "import qiskit; print('Qiskit:', qiskit.__version__)"

# Q# (QDK)
python -c "import qsharp; print('QDK:', qsharp.__version__)"

# Seguridad (0 CVEs)
pip-audit -r requirements-pinned.txt
```

---

## 🔧 Requisitos mínimos del sistema

| Recurso | Mínimo | Recomendado |
|---|---|---|
| CPU | 2 cores | 4+ cores |
| RAM | 4 GB | 8+ GB |
| Disco | 2 GB | 10+ GB |
| Python | 3.11 | 3.12 |
| SO | Ver tabla arriba | — |

---

## 🆘 Soporte

- **Issues**: https://github.com/GioCorpus/QuantumEnergyOS/issues
- **Seguridad**: security@quantumenergyos.org
- **Contribuir**: Ver [CONTRIBUTING.md](CONTRIBUTING.md)

---

*⚡ Desde Mexicali, Baja California — para el noroeste y el mundo entero.*
*Porque la energía limpia es un derecho, no un privilegio. Kardashev 0→1.*
