# QuantumEnergyOS Wayland Compositor — Installation Guide

## Overview

Ultra-lightweight Wayland compositor based on tinywl (wlroots) with quantum-futuristic aesthetics.

**Target:** < 2251 lines of code, 64MB RAM, no bloat.

## Features

- ✅ Quantum-futuristic blue neon theme
- ✅ Floating particle effects (50 particles, no GPU shaders)
- ✅ Energy meter integration via Unix socket
- ✅ Mexicali desert wallpaper with aurora effects
- ✅ Minimal top panel with system info
- ✅ App launch wave effects
- ✅ Blue-neon theme configuration

## System Requirements

### Minimum
- Linux kernel 5.10+ with DRM/Wayland support
- 64MB RAM
- x86_64 or aarch64 architecture
- GPU with DRM driver (Intel, AMD, or virtio-gpu for QEMU)

### Recommended
- 128MB RAM
- Intel/AMD GPU with Mesa drivers
- libinput for input handling

## Dependencies

### Required
- wlroots v0.18+
- wayland-server
- libinput
- pixman
- cairo (for particle rendering)
- libdrm
- xkbcommon

### Optional
- ImageMagick (for wallpaper generation)
- foot (lightweight Wayland terminal)

## Installation Methods

### Method 1: From Source (Recommended)

```bash
# 1. Clone repository
cd QuantumEnergyOS/compositor

# 2. Build dependencies (wlroots)
make deps

# 3. Build compositor
make

# 4. Install system-wide
sudo make install

# 5. Start energy daemon
sudo systemctl start quantum-energy
sudo systemctl enable quantum-energy

# 6. Run compositor
quantum-compositor -s foot
```

### Method 2: Using Install Script

```bash
# 1. Make script executable
chmod +x install.sh

# 2. Run installation
sudo ./install.sh

# 3. Start energy daemon
sudo systemctl start quantum-energy
sudo systemctl enable quantum-energy

# 4. Run compositor
quantum-compositor -s foot
```

### Method 3: Manual Installation

```bash
# 1. Install dependencies
sudo apt-get install \
    libwayland-dev \
    wayland-protocols \
    libxkbcommon-dev \
    libdrm-dev \
    libgbm-dev \
    libgles2-mesa-dev \
    libcairo2-dev \
    libinput-dev \
    libpixman-1-dev

# 2. Build wlroots
git clone https://gitlab.freedesktop.org/wlroots/wlroots.git
cd wlroots
git checkout 0.18.0
meson setup build --prefix=/usr/local -Dexamples=false -Dtests=false
ninja -C build
sudo ninja -C build install

# 3. Build compositor
make

# 4. Install
sudo install -m 755 build/quantum-compositor /usr/local/bin/
sudo mkdir -p /etc/quantum-compositor
sudo install -m 644 themes/quantum-neon.ini /etc/quantum-compositor/theme.ini
sudo mkdir -p /usr/share/quantum-compositor
sudo install -m 644 assets/wallpaper.png /usr/share/quantum-compositor/
```

## Configuration

### Theme Configuration

Edit `/etc/quantum-compositor/theme.ini`:

```ini
[colors]
primary = 0x00d4ff      # Quantum blue
secondary = 0x0099ff    # Deep blue
accent = 0x00ffcc       # Cyan glow
background = 0x0a0a1a   # Dark space

[particles]
enabled = true
count = 50
size = 3
speed = 0.5
color = 0x00d4ff

[panel]
height = 32
position = top
show_energy = true
```

### Energy Daemon Configuration

The energy daemon connects to `/var/run/quantum-energy.sock`.

**Commands:**
- `GET_ENERGY` — Returns energy percentage (0-100)
- `GET_STATUS` — Returns detailed JSON status
- `SIMULATE_BLACKOUT <risk>` — Simulate blackout (0.0-1.0)

**Example:**
```bash
echo "GET_ENERGY" | socat - UNIX-CONNECT:/var/run/quantum-energy.sock
```

## Testing

### QEMU Testing

```bash
# Basic test
make test

# Custom QEMU options
qemu-system-x86_64 \
    -enable-kvm \
    -m 256M \
    -vga virtio \
    -display gtk,gl=on \
    -device virtio-gpu-pci
```

### Real Hardware Testing

```bash
# Run on real hardware
quantum-compositor -s foot

# Check logs
tail -f /var/log/quantum-energy.log
```

## Usage

### Starting the Compositor

```bash
# Basic start
quantum-compositor

# With startup command
quantum-compositor -s foot

# With custom theme
quantum-compositor -t /path/to/theme.ini
```

### Keybindings

| Keybinding | Action |
|------------|--------|
| `Super+Return` | Open terminal |
| `Super+D` | Application launcher |
| `Super+Shift+Q` | Close window |
| `Super+Left` | Previous workspace |
| `Super+Right` | Next workspace |
| `Super+LeftMouse` | Move window |
| `Super+RightMouse` | Resize window |

### Energy Meter

The energy meter displays real-time energy status:

- 🟢 **Green (>70%)** — Normal operation
- 🟡 **Yellow (30-70%)** — Warning
- 🔴 **Red (<30%)** — Critical/blackout risk

## Troubleshooting

### Compositor won't start

```bash
# Check if wlroots is installed
pkg-config --modversion wlroots

# Check DRM support
ls -la /dev/dri/

# Check Wayland support
echo $WAYLAND_DISPLAY
```

### Energy daemon not connecting

```bash
# Check if daemon is running
systemctl status quantum-energy

# Check socket
ls -la /var/run/quantum-energy.sock

# Check logs
journalctl -u quantum-energy -f
```

### Particles not showing

```bash
# Check theme configuration
cat /etc/quantum-compositor/theme.ini | grep particles

# Enable particles
sudo sed -i 's/enabled = false/enabled = true/' /etc/quantum-compositor/theme.ini
```

### Low performance

```bash
# Reduce particles
sudo sed -i 's/count = 50/count = 20/' /etc/quantum-compositor/theme.ini

# Disable animations
sudo sed -i 's/enabled = true/enabled = false/' /etc/quantum-compositor/theme.ini

# Enable low power mode
sudo sed -i 's/low_power_mode = false/low_power_mode = true/' /etc/quantum-compositor/theme.ini
```

## Uninstallation

```bash
# Stop services
sudo systemctl stop quantum-energy
sudo systemctl disable quantum-energy

# Remove files
sudo rm -f /usr/local/bin/quantum-compositor
sudo rm -f /usr/local/bin/quantum-energy-daemon
sudo rm -rf /etc/quantum-compositor
sudo rm -rf /usr/share/quantum-compositor
sudo rm -f /etc/systemd/system/quantum-energy.service

# Reload systemd
sudo systemctl daemon-reload
```

## Development

### Building for Development

```bash
# Debug build
make DEBUG=1

# Run with debug output
./build/quantum-compositor -s foot 2>&1 | tee debug.log
```

### Code Structure

```
compositor/
├── src/
│   ├── main.c              # Main compositor (~400 lines)
│   └── energy-daemon.py    # Energy predictor daemon
├── themes/
│   └── quantum-neon.ini    # Theme configuration
├── assets/
│   ├── wallpaper.png       # Mexicali desert wallpaper
│   └── particles/          # Particle sprites
├── Makefile                # Build system
├── install.sh              # Installation script
├── README.md               # Project overview
└── INSTALL.md              # This file
```

### Adding New Features

1. **New Theme Colors**
   - Edit `themes/quantum-neon.ini`
   - Add color constants to `src/main.c`

2. **New Particle Effects**
   - Modify `particles_init()` in `src/main.c`
   - Add particle sprites to `assets/particles/`

3. **New Energy Sources**
   - Extend `EnergyPredictor` class in `src/energy-daemon.py`
   - Add new commands to socket protocol

## Performance

### Memory Usage

- Binary: ~200KB
- Runtime: ~50-60MB (wlroots + particles + panel)
- Total: ~60-65MB ✓ (under 64MB target)

### CPU Usage

- Idle: <1%
- Active: 2-5%
- Particles: <1% additional

### Binary Size

- Uncompressed: ~200KB
- Stripped: ~150KB
- With LTO: ~120KB

## License

MIT License — See LICENSE file

## Credits

- **wlroots** — Wayland compositor library
- **tinywl** — Minimal compositor reference
- **Giovanny Anthony Corpus Bernal** — QuantumEnergyOS project

## Support

- GitHub: https://github.com/GioCorpus/QuantumEnergyOS
- Issues: https://github.com/GioCorpus/QuantumEnergyOS/issues

---

**Desde Mexicali, BC — para el mundo. Kardashev 0→1**
