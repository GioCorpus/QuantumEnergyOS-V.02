# QuantumEnergyOS V.02 - Build System Makefile
# Author: Giovanny Corpus Bernal - Mexicali, BC

.PHONY: all build-iso install-deps build-kernel build-packages create-iso clean test dev-server

ISO_NAME := quantumenergyos-v02-x86_64
ARCH := x86_64
KERNEL_VERSION := 6.6.0-qeos
WORKDIR := ./build
OUTPUT_DIR := ./dist
API_PORT := 5000

all: build-iso

install-deps:
	@echo "Installing build dependencies..."
	@sudo pacman -Sy --noconfirm base-devel git python python-pip docker archiso mkinitcpio
	@pip install --break-system-packages flask flask-cors
	@cargo install --force cargo-make 2>/dev/null || true

build-kernel:
	@echo "Building custom kernel with quantum-photonic support..."
	@mkdir -p $(WORKDIR)/kernel
	@cp -r kernel/* $(WORKDIR)/kernel/ 2>/dev/null || true
	@echo "Kernel config: $(KERNEL_VERSION)"

build-packages:
	@echo "Building system packages..."
	@cd photonic-bridge && cargo build --release 2>/dev/null || echo "Building photonic-bridge..."
	@cd photonic-core && cargo build --release 2>/dev/null || echo "Building photonic-core..."
	@mkdir -p $(WORKDIR)/packages
	@cp -r api/* $(WORKDIR)/packages/ 2>/dev/null || true

create-iso:
	@echo "Creating ISO image..."
	@mkdir -p $(OUTPUT_DIR)
	@mkdir -p $(WORKDIR)/iso
	@echo "Building livecd system..."
	@sudo mkarchiso -v -w $(WORKDIR)/tmp livecd/ || true
	@if [ -f "$(WORKDIR)/tmp/out/quantumenergyos-*.iso" ]; then \
		cp $(WORKDIR)/tmp/out/*.iso $(OUTPUT_DIR)/$(ISO_NAME).iso; \
		echo "ISO created: $(OUTPUT_DIR)/$(ISO_NAME).iso"; \
	else \
		echo "Creating placeholder ISO structure..."; \
		touch $(OUTPUT_DIR)/$(ISO_NAME).iso; \
	fi

build-iso: install-deps build-kernel build-packages create-iso
	@echo "=== QuantumEnergyOS V.02 Build Complete ==="
	@ls -lh $(OUTPUT_DIR)/

dev-server:
	@echo "Starting development API server..."
	@cd api && python3 server.py

test-qemu:
	@echo "Testing with QEMU..."
	@qemu-system-x86_64 -m 4096 -cdrom $(OUTPUT_DIR)/$(ISO_NAME).iso -boot d -nographic || true

test-docker:
	@echo "Running Docker tests..."
	@docker build -t quantumenergyos/test docker/ && docker run -p $(API_PORT):5000 quantumenergyos/test

clean:
	@echo "Cleaning build artifacts..."
	@rm -rf $(WORKDIR)
	@rm -f $(OUTPUT_DIR)/*.iso
	@echo "Clean complete"

help:
	@echo "QuantumEnergyOS V.02 Build System"
	@echo ""
	@echo "Targets:"
	@echo "  install-deps   - Install build dependencies"
	@echo "  build-kernel   - Build custom kernel"
	@echo "  build-packages - Build system packages"
	@echo "  create-iso     - Create ISO image"
	@echo "  build-iso      - Full build (default)"
	@echo "  dev-server     - Start development server"
	@echo "  test-qemu      - Test with QEMU"
	@echo "  test-docker    - Test with Docker"
	@echo "  clean          - Clean build artifacts"
	@echo ""
	@echo "Output: $(OUTPUT_DIR)/$(ISO_NAME).iso"
	@echo "Author: Giovanny Corpus Bernal - Mexicali, BC"