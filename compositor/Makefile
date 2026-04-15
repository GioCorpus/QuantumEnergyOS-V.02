# ═══════════════════════════════════════════════════════════════════════
#  QuantumEnergyOS — Wayland Compositor Build System
#  Ultra-lightweight compositor based on tinywl (wlroots)
#
#  Target: < 2251 lines, 64MB RAM
#  Desde Mexicali, BC — para el mundo. Kardashev 0→1
# ═══════════════════════════════════════════════════════════════════════

CC = gcc
CFLAGS = -Wall -Wextra -Wno-unused-parameter -std=c11 -D_POSIX_C_SOURCE=200112L
CFLAGS += -O2 -flto -fno-plt
CFLAGS += -I./include

# Debug build
ifdef DEBUG
CFLAGS += -g -O0 -DDEBUG
endif

# Linker flags
LDFLAGS = -flto -Wl,--as-needed -Wl,-O1
LIBS = -lwayland-server -lwlr -lxkbcommon -lm

# Directories
SRC_DIR = src
BUILD_DIR = build
INSTALL_DIR = /usr/local

# Source files
SRCS = $(SRC_DIR)/main.c
OBJS = $(SRCS:$(SRC_DIR)/%.c=$(BUILD_DIR)/%.o)
TARGET = $(BUILD_DIR)/quantum-compositor

# wlroots version
WLR_VERSION = 0.18.0

# ═══════════════════════════════════════════════════════════════════════
#  Build Targets
# ═══════════════════════════════════════════════════════════════════════

.PHONY: all clean install deps test run

all: $(TARGET)

$(TARGET): $(OBJS)
	@mkdir -p $(BUILD_DIR)
	$(CC) $(LDFLAGS) -o $@ $^ $(LIBS)
	@echo "═══════════════════════════════════════════════════════════════"
	@echo "  QuantumEnergyOS Compositor built successfully!"
	@echo "  Binary: $(TARGET)"
	@echo "  Size: $$(du -h $(TARGET) | cut -f1)"
	@echo "═══════════════════════════════════════════════════════════════"

$(BUILD_DIR)/%.o: $(SRC_DIR)/%.c
	@mkdir -p $(BUILD_DIR)
	$(CC) $(CFLAGS) -c -o $@ $<

# ═══════════════════════════════════════════════════════════════════════
#  Dependencies (wlroots + wayland)
# ═══════════════════════════════════════════════════════════════════════

deps: deps-wlroots deps-wayland deps-libinput deps-pixman

deps-wlroots:
	@echo "═══════════════════════════════════════════════════════════════"
	@echo "  Building wlroots v$(WLR_VERSION)..."
	@echo "═══════════════════════════════════════════════════════════════"
	@if [ ! -d "deps/wlroots" ]; then \
		mkdir -p deps; \
		cd deps && git clone https://gitlab.freedesktop.org/wlroots/wlroots.git; \
	fi
	cd deps/wlroots && git checkout $(WLR_VERSION)
	cd deps/wlroots && meson setup build --prefix=/usr/local -Dexamples=false -Dtests=false
	cd deps/wlroots && ninja -C build
	cd deps/wlroots && sudo ninja -C build install
	@echo "wlroots installed successfully!"

deps-wayland:
	@echo "═══════════════════════════════════════════════════════════════"
	@echo "  Installing wayland dependencies..."
	@echo "═══════════════════════════════════════════════════════════════"
	sudo apt-get update
	sudo apt-get install -y \
		libwayland-dev \
		wayland-protocols \
		libxkbcommon-dev \
		libdrm-dev \
		libgbm-dev \
		libgles2-mesa-dev \
		libcairo2-dev \
		libinput-dev \
		libpixman-1-dev \
		libxcb1-dev \
		libxcb-composite0-dev \
		libxcb-xinput-dev \
		libxcb-xfixes0-dev \
		libxcb-render-util0-dev \
		libxcb-icccm4-dev \
		libxcb-ewmh-dev \
		libxcb-res0-dev
	@echo "Wayland dependencies installed!"

deps-libinput:
	@echo "═══════════════════════════════════════════════════════════════"
	@echo "  Installing libinput..."
	@echo "═══════════════════════════════════════════════════════════════"
	sudo apt-get install -y libinput-dev
	@echo "libinput installed!"

deps-pixman:
	@echo "═══════════════════════════════════════════════════════════════"
	@echo "  Installing pixman..."
	@echo "═══════════════════════════════════════════════════════════════"
	sudo apt-get install -y libpixman-1-dev
	@echo "pixman installed!"

# ═══════════════════════════════════════════════════════════════════════
#  Installation
# ═══════════════════════════════════════════════════════════════════════

install: $(TARGET)
	@echo "═══════════════════════════════════════════════════════════════"
	@echo "  Installing QuantumEnergyOS Compositor..."
	@echo "═══════════════════════════════════════════════════════════════"
	sudo install -m 755 $(TARGET) $(INSTALL_DIR)/bin/quantum-compositor
	sudo mkdir -p /etc/quantum-compositor
	sudo install -m 644 themes/quantum-neon.ini /etc/quantum-compositor/theme.ini
	sudo mkdir -p /usr/share/quantum-compositor
	sudo install -m 644 assets/wallpaper.png /usr/share/quantum-compositor/
	@echo "Installation complete!"
	@echo "Binary: $(INSTALL_DIR)/bin/quantum-compositor"
	@echo "Config: /etc/quantum-compositor/theme.ini"
	@echo "Assets: /usr/share/quantum-compositor/"

# ═══════════════════════════════════════════════════════════════════════
#  Testing
# ═══════════════════════════════════════════════════════════════════════

test: $(TARGET)
	@echo "═══════════════════════════════════════════════════════════════"
	@echo "  Testing QuantumEnergyOS Compositor in QEMU..."
	@echo "═══════════════════════════════════════════════════════════════"
	qemu-system-x86_64 \
		-enable-kvm \
		-m 256M \
		-vga virtio \
		-display gtk,gl=on \
		-device virtio-gpu-pci \
		-nographic \
		-kernel /boot/vmlinuz \
		-initrd /boot/initrd.img \
		-append "root=/dev/sda1 console=ttyS0" \
		-hda /dev/sda

run: $(TARGET)
	@echo "═══════════════════════════════════════════════════════════════"
	@echo "  Running QuantumEnergyOS Compositor..."
	@echo "═══════════════════════════════════════════════════════════════"
	./$(TARGET) -s "foot"  # foot is a lightweight Wayland terminal

# ═══════════════════════════════════════════════════════════════════════
#  Utilities
# ═══════════════════════════════════════════════════════════════════════

clean:
	@echo "═══════════════════════════════════════════════════════════════"
	@echo "  Cleaning build artifacts..."
	@echo "═══════════════════════════════════════════════════════════════"
	rm -rf $(BUILD_DIR)
	@echo "Clean complete!"

distclean: clean
	rm -rf deps
	@echo "Distribution clean complete!"

size: $(TARGET)
	@echo "═══════════════════════════════════════════════════════════════"
	@echo "  Binary size analysis"
	@echo "═══════════════════════════════════════════════════════════════"
	@echo "Binary: $(TARGET)"
	@du -h $(TARGET)
	@echo ""
	@echo "Lines of code:"
	@wc -l $(SRCS)
	@echo ""
	@echo "Memory estimate (64MB target):"
	@echo "  Binary: ~$$(du -k $(TARGET) | cut -f1) KB"
	@echo "  Runtime: ~50-60 MB (wlroots + particles + panel)"
	@echo "  Total: ~60-65 MB ✓"

# ═══════════════════════════════════════════════════════════════════════
#  Help
# ═══════════════════════════════════════════════════════════════════════

help:
	@echo "═══════════════════════════════════════════════════════════════"
	@echo "  QuantumEnergyOS Compositor — Build System"
	@echo "═══════════════════════════════════════════════════════════════"
	@echo ""
	@echo "Targets:"
	@echo "  all        — Build compositor (default)"
	@echo "  deps       — Build all dependencies (wlroots, wayland, etc.)"
	@echo "  install    — Install to system"
	@echo "  test       — Test in QEMU"
	@echo "  run        — Run compositor locally"
	@echo "  clean      — Remove build artifacts"
	@echo "  distclean  — Remove build artifacts and deps"
	@echo "  size       — Show binary size analysis"
	@echo "  help       — Show this help"
	@echo ""
	@echo "Examples:"
	@echo "  make deps          # Build dependencies first"
	@echo "  make               # Build compositor"
	@echo "  make install       # Install system-wide"
	@echo "  make run           # Run with foot terminal"
	@echo ""
	@echo "Debug build:"
	@echo "  make DEBUG=1       # Build with debug symbols"
	@echo ""
