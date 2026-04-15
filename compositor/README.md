# QuantumEnergyOS Wayland Compositor

## Overview
Ultra-lightweight Wayland compositor based on tinywl (wlroots) with quantum-futuristic aesthetics.

**Target:** < 2251 lines of code, 64MB RAM, no bloat.

## Architecture
```
compositor/
├── src/
│   ├── main.c              # Entry point (~100 lines, tinywl base)
│   ├── server.c            # Wayland server core
│   ├── server.h            # Server structures
│   ├── output.c            # Output/rendering
│   ├── output.h
│   ├── input.c             # Keyboard/mouse handling
│   ├── input.h
│   ├── particle.c          # Quantum particle system
│   ├── particle.h
│   ├── panel.c             # Top panel with energy meter
│   ├── panel.h
│   ├── energy.c            # Energy predictor socket client
│   ├── energy.h
│   ├── theme.c             # Blue-neon theme loader
│   └── theme.h
├── themes/
│   └── quantum-neon.ini    # Theme configuration
├── assets/
│   ├── wallpaper.png       # Mexicali desert with auroras
│   └── particles/          # Particle sprites
├── Makefile
└── README.md
```

## Dependencies
- wlroots v0.18+
- wayland-server
- libinput
- pixman
- cairo (for particle rendering)
- libdrm

## Build
```bash
make deps    # Build wlroots
make         # Build compositor
make install # Install to /usr/local
```

## Run
```bash
# In QEMU
qemu-system-x86_64 -enable-kvm -m 256M -vga virtio ...

# On real hardware
./quantum-compositor
```

## Theme: Quantum Neon
- **Primary:** #00d4ff (quantum blue)
- **Secondary:** #0099ff (deep blue)
- **Accent:** #00ffcc (cyan glow)
- **Background:** #0a0a1a (dark space)
- **Particles:** Floating blue dots with glow

## Energy Meter
Connects to `/var/run/quantum-energy.sock` for real-time energy status.
Displays available energy percentage with color coding:
- Green (>70%): Normal operation
- Yellow (30-70%): Warning
- Red (<30%): Critical/blackout risk

## Mexicali Desert Wallpaper
Night desert scene with:
- Sand dunes in foreground
- Starry sky background
- Blue aurora waves (quantum energy visualization)
- Solar glow on horizon
