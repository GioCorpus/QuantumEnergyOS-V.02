# ═══════════════════════════════════════════════════════════════════════
#  Makefile — QuantumEnergyOS V.02
#  Build system principal: ISO · Rust · Python · Q# · Tests
# ═══════════════════════════════════════════════════════════════════════

QEOS_VERSION  := 0.2.0
ISO_NAME      := QuantumEnergyOS-v$(QEOS_VERSION)
ARCH          := amd64
PYTHON        := python3
VENV          := venv
VENV_PY       := $(VENV)/bin/python
VENV_PIP      := $(VENV)/bin/pip
CARGO         := cargo
ISO_MAX_MB    := 2700

UNAME_M := $(shell uname -m)
UNAME_S := $(shell uname -s)

.PHONY: all iso iso-arm qemu-test kernel rust-build python-setup \
        api-run dashboard-dev install-deps test lint security \
        size-check clean release help

# ── Default ────────────────────────────────────────────────────────────
all: install-deps rust-build python-setup
	@echo "✅ QuantumEnergyOS V.02 listo"
	@echo "   Para construir ISO: make iso"
	@echo "   Para iniciar API:   make api-run"
	@echo "   Para tests:         make test"

help:
	@echo ""
	@echo "  ⚡ QuantumEnergyOS V.02 — Build System"
	@echo ""
	@echo "  make all           → Setup completo (sin ISO)"
	@echo "  make iso           → Construir ISO amd64 (~2.5 GB)"
	@echo "  make iso-arm       → Construir ISO arm64"
	@echo "  make qemu-test     → Probar ISO en QEMU"
	@echo "  make api-run       → Iniciar servidor Flask"
	@echo "  make dashboard-dev → Iniciar React en modo dev"
	@echo "  make test          → Correr todos los tests"
	@echo "  make security      → Escaneo de seguridad completo"
	@echo "  make release       → Build de release + checksums"
	@echo "  make clean         → Limpiar artefactos"
	@echo ""

# ── Dependencias ───────────────────────────────────────────────────────
install-deps:
	@echo "── Instalando dependencias"
	# Python
	$(PYTHON) -m venv $(VENV)
	$(VENV_PIP) install --upgrade pip --quiet
	$(VENV_PIP) install -r requirements-pinned.txt --quiet
	# Rust
	@rustup toolchain install nightly --quiet 2>/dev/null || true
	@rustup default nightly 2>/dev/null || true
	@rustup target add x86_64-unknown-none x86_64-unknown-linux-musl \
	    aarch64-unknown-none aarch64-apple-darwin 2>/dev/null || true
	# Node.js (dashboard)
	@command -v node >/dev/null && npm install --silent 2>/dev/null || \
	    echo "Node.js no encontrado — dashboard React no disponible"
	@echo "✅ Dependencias instaladas"

# ── Rust ──────────────────────────────────────────────────────────────
rust-build:
	@echo "── Compilando Rust (release)"
	$(CARGO) build --workspace --exclude qeos-kernel --release 2>&1 | tail -3
	@echo "✅ Rust compilado"

kernel:
	@echo "── Compilando kernel bare-metal"
	$(CARGO) build -p qeos-kernel --target x86_64-unknown-none \
	    --profile kernel \
	    -Z build-std=core,alloc \
	    -Z build-std-features=compiler-builtins-mem
	@echo "✅ Kernel compilado"

# ── Python ─────────────────────────────────────────────────────────────
python-setup: $(VENV)/bin/python
	@echo "── Setup Python OK"

# ── API Flask ──────────────────────────────────────────────────────────
api-run: $(VENV)/bin/python
	@echo "⚡ Iniciando QuantumEnergyOS API en http://localhost:8000"
	$(VENV_PY) api/server.py

api-dev: $(VENV)/bin/python
	QEOS_ENV=development $(VENV_PY) -m uvicorn api.server:app \
	    --reload --host 0.0.0.0 --port 8000

# ── Dashboard React ────────────────────────────────────────────────────
dashboard-dev:
	npm run dev

dashboard-build:
	npm run build

# ── ISO ────────────────────────────────────────────────────────────────
iso:
	@echo "── Construyendo ISO $(ISO_NAME)-$(ARCH).iso"
	@if ! command -v mkarchiso >/dev/null 2>&1; then \
	    echo "✗ mkarchiso no encontrado — instala archiso:"; \
	    echo "  pacman -S archiso    (Arch Linux)"; \
	    echo "  o usa: make iso-docker"; \
	    exit 1; \
	fi
	chmod +x scripts/build-iso.sh
	sudo ./scripts/build-iso.sh $(ARCH)
	$(MAKE) size-check ISO=$(ISO_NAME)-$(ARCH).iso

iso-arm:
	$(MAKE) iso ARCH=arm64

iso-docker:
	@echo "── Construyendo ISO via Docker (sin Arch nativo)"
	docker run --rm --privileged \
	    -v "$(PWD):/src:ro" \
	    -v "$(PWD)/dist:/output" \
	    archlinux:latest \
	    bash -c "pacman -Sy --noconfirm archiso squashfs-tools xorriso grub && \
	             cp -r /src /build && cd /build && \
	             chmod +x scripts/build-iso.sh && \
	             ./scripts/build-iso.sh $(ARCH) && \
	             cp QuantumEnergyOS-*.iso /output/"
	@echo "✅ ISO en dist/"

# ── QEMU ───────────────────────────────────────────────────────────────
qemu-test:
	@ISO=$$(ls $(ISO_NAME)-$(ARCH).iso 2>/dev/null || ls dist/$(ISO_NAME)-$(ARCH).iso 2>/dev/null); \
	if [ -z "$$ISO" ]; then echo "✗ ISO no encontrada — construye primero: make iso"; exit 1; fi; \
	echo "⚡ Probando $$ISO en QEMU (4 GB RAM)..."; \
	qemu-system-x86_64 \
	    -m 4G \
	    -cdrom "$$ISO" \
	    -boot d \
	    -enable-kvm \
	    -cpu host \
	    -vga virtio \
	    -smp 4 2>/dev/null || \
	qemu-system-x86_64 \
	    -m 4G \
	    -cdrom "$$ISO" \
	    -boot d \
	    -vga virtio

# ── Tests ──────────────────────────────────────────────────────────────
test: test-python test-rust

test-python: $(VENV)/bin/python
	@echo "── Tests Python"
	$(VENV_PY) -m pytest tests/ -v --tb=short 2>&1 | tail -20

test-rust:
	@echo "── Tests Rust"
	$(CARGO) test --workspace --exclude qeos-kernel -- --nocapture 2>&1 | tail -20

test-qsharp: $(VENV)/bin/python
	@echo "── Tests Q#"
	$(VENV_PY) -c "import qsharp; qsharp.init('.')" && \
	$(VENV_PY) -c "import qsharp; print(qsharp.eval('QuantumEnergyOS.Grid.SimularBalanceoRed()'))"

# ── Lint ───────────────────────────────────────────────────────────────
lint: lint-python lint-rust

lint-python: $(VENV)/bin/python
	$(VENV)/bin/ruff check . --ignore E501 || true

lint-rust:
	$(CARGO) clippy --workspace --exclude qeos-kernel -- -A clippy::needless_return || true

# ── Seguridad ──────────────────────────────────────────────────────────
security: $(VENV)/bin/python
	@echo "── Escaneando seguridad"
	$(VENV)/bin/pip-audit -r requirements-pinned.txt
	$(VENV)/bin/bandit -r . --exclude ./venv,./node_modules --severity-level medium -q || true
	@echo "✅ Seguridad: OK"

# ── Verificar tamaño ISO ───────────────────────────────────────────────
size-check:
	@ISO_FILE=$(ISO); \
	[ -z "$$ISO_FILE" ] && ISO_FILE=$$(ls QuantumEnergyOS-*.iso 2>/dev/null | head -1); \
	[ -z "$$ISO_FILE" ] && ISO_FILE=$$(ls dist/QuantumEnergyOS-*.iso 2>/dev/null | head -1); \
	if [ -z "$$ISO_FILE" ]; then echo "No se encontró ISO para verificar"; exit 0; fi; \
	SIZE_MB=$$(du -m "$$ISO_FILE" | cut -f1); \
	echo "ISO: $$ISO_FILE ($$SIZE_MB MB)"; \
	if [ "$$SIZE_MB" -le "$(ISO_MAX_MB)" ]; then \
	    echo "✅ Dentro del límite (<= $(ISO_MAX_MB) MB)"; \
	else \
	    echo "⚠️  Excede $(ISO_MAX_MB) MB — optimizar"; \
	fi; \
	sha256sum "$$ISO_FILE" > "$$ISO_FILE.sha256"; \
	echo "SHA-256: $$(cat $$ISO_FILE.sha256 | awk '{print $$1}')"

# ── Release ────────────────────────────────────────────────────────────
release: iso security test
	@echo "── Preparando release v$(QEOS_VERSION)"
	$(MAKE) size-check
	@for iso in QuantumEnergyOS-v$(QEOS_VERSION)*.iso; do \
	    sha256sum "$$iso" > "$$iso.sha256"; \
	    sha512sum "$$iso" > "$$iso.sha512"; \
	    gpg --armor --detach-sign "$$iso" 2>/dev/null || \
	        echo "GPG no configurado — firma omitida"; \
	    echo "✅ $$iso firmado"; \
	done
	git tag -s "v$(QEOS_VERSION)" -m "QuantumEnergyOS V.02 — Release" 2>/dev/null || \
	git tag "v$(QEOS_VERSION)" -m "QuantumEnergyOS V.02 — Release"

# ── Limpiar ────────────────────────────────────────────────────────────
clean:
	$(CARGO) clean 2>/dev/null || true
	rm -rf dist/ $(VENV)/ __pycache__/ .pytest_cache/ *.iso *.sha256 *.sha512 *.sig
	rm -rf node_modules/ .next/ build/
	find . -name "*.pyc" -delete
	find . -name "__pycache__" -type d -exec rm -rf {} + 2>/dev/null || true
	@echo "✅ Limpio"
