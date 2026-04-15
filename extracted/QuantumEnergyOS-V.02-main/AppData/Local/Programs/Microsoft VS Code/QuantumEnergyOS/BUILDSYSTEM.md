# QuantumEnergyOS V.02 - Build System

## Overview
This file contains complete build instructions for Windows (PowerShell), Linux, and macOS.

## Prerequisites

### Linux (Arch Linux - Recommended)
```bash
sudo pacman -Syu
sudo pacman -S base-devel cmake make git python python-pip docker
sudo pacman -S archiso mkinitcpio
```

### Windows
- Windows 10/11 with WSL2 or PowerShell 5.1+
- 20GB free disk space
- 8GB RAM minimum

### macOS
- macOS 11+ with Xcode Command Line Tools
- Homebrew: `brew install coreutils qemu docker`

---

## Build Commands

### Linux (Native)
```bash
# Full build
make build-iso

# Step by step
make install-deps
make build-kernel
make build-packages
make create-iso

# Development
make dev-server
make test
```

### Windows PowerShell
```powershell
# Load build functions
. .\buildsystem\build.ps1

# Full build
Build-ISO

# Or step by step
Install-Dependencies
Build-Kernel
Build-Packages
Create-ISO
```

---

## Build Variables

| Variable | Default | Description |
|----------|---------|-------------|
| ISO_NAME | quantumenergyos-v02 | ISO filename |
| ARCH | x86_64 | Architecture |
| KERNEL_VERSION | 6.6.0-qeos | Custom kernel version |
| WORKDIR | ./build | Build working directory |
| OUTPUT_DIR | ./dist | Output directory |

---

## Output

The build produces:
- `dist/quantumenergyos-v02-x86_64.iso` (~2.5 GB)
- SHA256 checksum file
- Build logs

---

## Testing

### QEMU Test
```bash
make test-qemu
```

### Docker Test
```bash
make test-docker
```

---

## Troubleshooting

### Common Issues

1. **Out of space**: Ensure 20GB free
2. **Permission denied**: Run as Administrator/root
3. **archiso not found**: Install with `pacman -S archiso`
4. **Rust not found**: Install from rustup.rs

---

## Contact

- Author: Giovanny Corpus Bernal
- Location: Mexicali, Baja California
- Mission: Never more blackouts in Mexicali