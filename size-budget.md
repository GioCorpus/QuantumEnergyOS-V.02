# ⚖️ QuantumEnergyOS — Presupuesto de Tamaño ISO

**Target:** < 900 MB  |  **Build System:** Rust + musl + UPX + xz

---

## 📊 Presupuesto por componente

| Componente | Sin comprimir | Comprimido (xz) | Técnica |
|---|---|---|---|
| **Kernel Rust** (qeos-kernel) | ~15 MB | ~4 MB | `opt-level=z` + LTO + strip |
| **Quantum libs** (qeos-quantum) | ~8 MB | ~2 MB | `opt-level=z` + LTO |
| **Userspace** (init + qsh + qeosd) | ~25 MB | ~8 MB | musl estático + UPX `--best --lzma` |
| **Bootloader** (GRUB UEFI + BIOS) | ~12 MB | ~5 MB | grub-mkrescue --compress=xz |
| **Python 3.11 slim** (para notebooks) | ~65 MB | ~22 MB | python3.11-minimal |
| **Qiskit + NumPy + SciPy** | ~180 MB | ~60 MB | wheels pre-compilados |
| **Q# / QDK** (qsharp package) | ~120 MB | ~40 MB | wheels pre-compilados |
| **Matplotlib** (visualizaciones) | ~50 MB | ~18 MB | sin Tk, solo Agg backend |
| **Base sistema** (musl libc, certs, timezone) | ~8 MB | ~3 MB | Alpine-based |
| **Fonts + assets mínimos** | ~5 MB | ~2 MB | solo monospace |
| **SquashFS overhead** | — | ~5 MB | — |
| **ISO filesystem** | — | ~8 MB | El Torito + GPT |
| **TOTAL ESTIMADO** | **~488 MB** | **~177 MB** | — |
| **Con margen de seguridad (x5 ratio)** | — | **~590 MB** | — |

**✅ Meta alcanzada: ~590 MB — 34% bajo el límite de 900 MB**

---

## 🔧 Técnicas de optimización de tamaño

### Kernel Rust
```toml
[profile.kernel]
opt-level     = "z"     # Prioridad: tamaño sobre velocidad
lto           = "fat"   # Link-Time Optimization total
codegen-units = 1       # Un solo codegen unit → mejor LTO
panic         = "abort" # Sin unwind tables (~200 KB ahorro)
strip         = "symbols"
```

### Userspace musl estático
```bash
# Sin glibc — binarios 100% portátiles entre distros
cargo build --target x86_64-unknown-linux-musl

# UPX comprime ejecutables ~50-60%
upx --best --lzma target/.../qeosd   # 8 MB → ~3 MB
upx --best --lzma target/.../qsh     # 5 MB → ~2 MB
```

### Python slim (solo lo necesario)
```dockerfile
# Solo paquetes esenciales — sin pip cache, sin .pyc duplicados
FROM python:3.11-alpine AS python-slim
RUN pip install --no-cache-dir \
    qiskit==1.4.2 \
    qiskit-aer==0.16.0 \
    qsharp==1.11.0 \
    numpy==2.2.4 \
    scipy==1.17.0 \
    matplotlib==3.10.8 \
    --target /opt/python-pkgs
```

### SquashFS máxima compresión
```bash
mksquashfs rootfs/ rootfs.squashfs \
    -comp xz \
    -Xbcj x86 \           # Branch/Call/Jump filter para mejor compresión
    -b 1048576 \           # Block size 1MB
    -no-xattrs \
    -noappend
```

---

## 📐 Árbol de directorios de la ISO

```
QuantumEnergyOS-1.0.0-amd64.iso (~590 MB)
│
├── boot/
│   ├── qeos-kernel.bin          (~4 MB comprimido)
│   └── grub/
│       ├── grub.cfg
│       ├── i386-pc/             (BIOS boot)
│       └── x86_64-efi/          (UEFI boot)
│
├── rootfs.squashfs              (~570 MB — todo el sistema de archivos)
│   ├── sbin/
│   │   ├── init                 (~800 KB — PID 1 en Rust+musl)
│   │   └── qeosd                (~2 MB — daemon cuántico)
│   ├── bin/
│   │   └── qsh                  (~1.5 MB — Quantum Shell)
│   ├── opt/
│   │   └── qeos/
│   │       ├── quantum/         (lib cuántica Rust compilada)
│   │       └── python/          (Python slim + Qiskit + QDK)
│   └── lib/
│       └── musl/                (~1 MB — musl libc)
│
├── EFI/
│   └── BOOT/
│       ├── BOOTX64.EFI          (UEFI x86_64)
│       └── BOOTAA64.EFI         (UEFI arm64)
│
└── [checksums]
    ├── QuantumEnergyOS-1.0.0-amd64.iso.sha256
    └── QuantumEnergyOS-1.0.0-amd64.iso.sha512
```

---

## 🍎 macOS — "Frontera Final"

macOS no bootea ISOs de terceros — la estrategia es:

| Componente | Implementación | Tamaño |
|---|---|---|
| **App bundle `.app`** | Binario Rust nativo arm64/x86_64 | ~8 MB |
| **Universal Binary** | `lipo` M-series + Intel en uno | ~14 MB |
| **NEON/AdvSIMD** | Álgebra cuántica acelerada en hardware | — |
| **Metal compute** | GPU M-series para simulación de circuitos | roadmap |
| **Homebrew package** | `brew tap giocorpus/qeos && brew install qeos` | — |
| **DMG installer** | `.dmg` con `.app` + drag-to-Applications | ~50 MB |

---

## 🎯 Resumen de targets y tamaños

| Target | Formato | Tamaño estimado | Estado |
|---|---|---|---|
| `x86_64` Linux | ISO booteable | ~590 MB | ✅ |
| `aarch64` Linux | ISO booteable (UTM/QEMU) | ~590 MB | ✅ |
| `aarch64-apple-darwin` | `.app` Universal | ~50 MB DMG | ✅ |
| `x86_64-apple-darwin` | `.app` Universal | incluido en DMG | ✅ |
| Windows `x64` | `.zip` portable | ~200 MB | ✅ |
| Windows `ARM64` | `.zip` portable | ~200 MB | ✅ |
| Docker multi-arch | `ghcr.io/...` | ~180 MB comprimido | ✅ |
