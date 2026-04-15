# ═══════════════════════════════════════════════════════════════════════
#  Dockerfile — QuantumEnergyOS V.02 Immutable (OSTree-ready)
#  Contenedor atomic para la API cuántica
#  Compatible con sistema inmutable OSTree
# ═══════════════════════════════════════════════════════════════════════

# ═══════════════════════════════════════════════════════════════════════
#  Stage 1: Builder
# ═══════════════════════════════════════════════════════════════════════
FROM python:3.11-slim-bookworm AS builder

ARG BUILD_DATE
ARG GIT_SHA

LABEL org.opencontainers.image.title="QuantumEnergyOS"
LABEL org.opencontainers.image.description="Optimización cuántica de redes eléctricas - Immutable Edition"
LABEL org.opencontainers.image.source="https://github.com/GioCorpus/QuantumEnergyOS"
LABEL org.opencontainers.image.licenses="MIT"
LABEL version="0.2.0-immutable"

ENV PYTHONDONTWRITEBYTECODE=1 \
    PYTHONUNBUFFERED=1 \
    PIP_NO_CACHE_DIR=1 \
    PIP_DISABLE_PIP_VERSION_CHECK=1

WORKDIR /build

# ═══════════════════════════════════════════════════════════════════════
#  Install system dependencies for quantum computing
# ═══════════════════════════════════════════════════════════════════════
RUN apt-get update && apt-get install -y --no-install-recommends \
    gcc \
    g++ \
    libopenblas-dev \
    curl \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

# ═══════════════════════════════════════════════════════════════════════
#  Install Python dependencies
# ═══════════════════════════════════════════════════════════════════════
COPY requirements.txt requirements-pinned.txt* ./
RUN pip install --upgrade pip --no-cache-dir && \
    (pip install --no-cache-dir --require-hashes -r requirements-pinned.txt 2>/dev/null || \
     pip install --no-cache-dir -r requirements.txt)

# ═══════════════════════════════════════════════════════════════════════
#  Stage 2: Runtime
# ═══════════════════════════════════════════════════════════════════════
FROM python:3.11-slim-bookworm AS runtime

# ═══════════════════════════════════════════════════════════════════════
#  Security: non-root user, minimal capabilities
# ═══════════════════════════════════════════════════════════════════════
RUN groupadd --gid 1000 qeos && \
    useradd --uid 1000 --gid 1000 --no-create-home --shell /bin/false qeos

# Copy dependencies from builder
COPY --from=builder /usr/local/lib/python3.11/site-packages/ /usr/local/lib/python3.11/site-packages/

WORKDIR /app

# ═══════════════════════════════════════════════════════════════════════
#  Copy application code
# ═══════════════════════════════════════════════════════════════════════
COPY --chown=qeos:qeos server.py .
COPY --chown=qeos:qeos core.py .
COPY --chown=qeos:qeos api/ ./api/
COPY --chown=qeos:qeos cloud/ ./cloud/ 2>/dev/null || true
COPY --chown=qeos:qeos photonic-core/ ./photonic-core/ 2>/dev/null || true
COPY --chown=qeos:qeos photonic-bridge/ ./photonic-bridge/ 2>/dev/null || true

# ═══════════════════════════════════════════════════════════════════════
#  Create necessary directories with correct permissions
# ═══════════════════════════════════════════════════════════════════════
RUN mkdir -p /tmp/qeos /app/logs /data /var/log && \
    chown -R qeos:qeos /tmp/qeos /app/logs /data /var/log

# Switch to non-root user
USER qeos:qeos

# ═══════════════════════════════════════════════════════════════════════
#  Environment variables
# ═══════════════════════════════════════════════════════════════════════
ENV PYTHONDONTWRITEBYTECODE=1 \
    PYTHONUNBUFFERED=1 \
    PORT=8000 \
    QEOS_ENV=production \
    QEOS_VERSION=0.2.0-immutable \
    QEOS_MISSION="Nunca más apagones en Mexicali" \
    QEOS_LOCATION="Mexicali, Baja California, México" \
    PYTHONPATH=/app \
    LOG_LEVEL=info

# ═══════════════════════════════════════════════════════════════════════
#  Expose port
# ═══════════════════════════════════════════════════════════════════════
EXPOSE 8000

# Health check
HEALTHCHECK --interval=30s --timeout=10s --start-period=40s --retries=3 \
    CMD curl -sf http://localhost:8000/health || exit 1

# ═══════════════════════════════════════════════════════════════════════
#  Run the API
# ═══════════════════════════════════════════════════════════════════════
CMD ["python", "server.py"]

# ═══════════════════════════════════════════════════════════════════════
#  VOLUMES (datos persistentes - mounts desde host OSTree)
# ═══════════════════════════════════════════════════════════════════════
#  /data     → datos de aplicación (bind mount desde /var/lib/qeos)
#  /var/log  → logs (bind mount desde /var/log/qeos)
# ═══════════════════════════════════════════════════════════════════════
VOLUME ["/data", "/var/log"]

# ═══════════════════════════════════════════════════════════════════════
#  HOW TO RUN (Immutable/OSTree deployment):
# ═══════════════════════════════════════════════════════════════════════
#
#  Build:
#    podman build -t localhost/qeos-api:latest .
#
#  Run with security:
#    podman run \
#      --name qeos-api \
#      --hostname qeos-api \
#      --read-only=true \
#      --tmpfs /tmp:size=64m,noexec,nosuid \
#      --memory=512m \
#      --memory-swap=512m \
#      --cpus=2 \
#      --security-opt no-new-privileges \
#      --cap-drop ALL \
#      --cap-add NET_BIND_SERVICE \
#      --network host \
#      --volume /var/lib/qeos/api:/data:z \
#      --volume /var/log/qeos:/var/log:z \
#      --env-file /etc/qeos/api.env \
#      --health-cmd "curl -sf http://localhost:8000/health" \
#      --health-interval "30s" \
#      localhost/qeos-api:latest
#