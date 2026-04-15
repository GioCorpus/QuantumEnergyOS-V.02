# QuantumEnergyOS V.02 - Docker Configuration
# For API Server and testing

FROM archlinux:base

LABEL maintainer="Giovanny Corpus Bernal <giovanny@quantumenergyos.mx>"
LABEL description="QuantumEnergyOS Energy API Server"

# Install dependencies
RUN pacman -Sy --noconfirm python python-pip git rust

# Install Python packages
RUN pip install --break-system-packages flask flask-cors

# Copy application files
COPY api/ /app/api/
COPY photonic-core/ /app/photonic-core/
COPY photonic-bridge/ /app/photonic-bridge/

WORKDIR /app

# Expose API port
EXPOSE 5000

# Health check
HEALTHCHECK --interval=30s --timeout=10s --start-period=5s --retries=3 \
    CMD curl -f http://localhost:5000/health || exit 1

# Start API server
CMD ["python", "api/server.py"]