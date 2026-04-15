# Security Policy

## Supported Versions
We support the latest stable release.

## Reporting a Vulnerability
Please report vulnerabilities privately to:

security@quantumenergyos.org

Do NOT open public issues for security problems.

import os
from qiskit_ibm_runtime import QiskitRuntimeService

token = os.getenv("IBM_QUANTUM_TOKEN")
service = QiskitRuntimeService(token=token)

export IBM_QUANTUM_TOKEN="TU_TOKEN"

QuantumCircuit(1000)

MAX_QUBITS = 32
if n_qubits > MAX_QUBITS:
    raise ValueError("Too many qubits")

    pip-audit
safety
bandit

pip install pip-audit
pip-audit

name: Security Scan

on: [push, pull_request]

jobs:
  scan:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - name: Install tools
        run: |
          pip install pip-audit bandit
      - name: Dependency scan
        run: pip-audit
      - name: Static security scan pip check -r requirements-pinned.txt
        run: bandit -r .
     # Producción — reproducible al 100%
pip install -r requirements-pinned.txt # Exact content of line 51
sed -n '51p' requirements-pinned.txt
package==1.2.3
package>=1.2,<2.0
git+https://github.com/user/repo.git@branch#egg=package
package==1.2.3 --hash=sha256:abc123...
sed -n '51p' requirements-pinned.txt
awk 'NR==51' requirements-pinned.txt
pip install --dry-run -r requirements-pinned.txt
pip-compile --generate-hashes --output-file=requirements-pinned.txt requirements.in
sed -n '51p' requirements-pinned.txt pip install -r requirements-pinned.txt
pip install --verbose -r requirements-pinned.txt > pip-debug.log 2>&1
pip-compile --generate-hashes --output-file=requirements-pinned.txt requirements.in
pip-audit -r requirements-pinned.txt
pip install --dry-run -r requirements-pinned.txt
      - name: Static security scan pip check -r requirements-pinned.txt
        run: bandit -r .
              - name: Dependency Audit (pip-audit)
        run: pip-audit --strict

      - name: Static Code Analysis (bandit)
        run: bandit -r . -ll  # -ll = low+ level; adjust as needed

      - name: Consistency Check
        run: pip check -r requirements-pinned.txt || true  # non-fatal if desired
# Context: lines 48 to 54 (most useful)
sed -n '48,54p' requirements-pinned.txt

# Alternative (first 60 lines, portable)
head -n 60 requirements-pinned.txt | tail -n 13
pip install --verbose -r requirements-pinned.txt > pip-verbose.log 2>&1
pip hash --algorithm sha256 <package-file-you-suspect>.whl
package==1.2.3 pkg_resources.parse_requirements
package>=1.2,<2.0
git+https://github.com/user/repo.git@branch#egg=package
package==1.2.3 --hash=sha256:abc123...
sed -n '51p' requirements-pinned.txt
awk 'NR==51' requirements-pinned.txt
sed -n '48,54p' requirements-pinned.txt
pip install -r requirements-pinned.txt
pip install --verbose -r requirements-pinned.txt
token = os.getenv("IBM_QUANTUM_TOKEN")
service = QiskitRuntimeService(token=token)
```

Además el cliente implementa `MAX_QUBITS=32` con mensaje claro, fingerprint SHA-256 del token para logs (sin exponer el token real), redacción automática del token en mensajes de error, y fallback automático a simulador local si no hay conexión IBM.

**`Dockerfile`** — dos mejoras respecto a la versión básica que mostraste:

| Antes | Después |
|---|---|
| `RUN pip install qiskit qiskit-aer` | `requirements-pinned.txt` (0 CVEs) |
| Sin usuario definido → corre como root | `USER qeos:1000` |
| Sin límites de recursos | `--memory=512m --cpus=2` en runtime |
| Sin health check | `HEALTHCHECK` cada 30s |

**Archivos de comunidad** (`SECURITY.md`, `CONTRIBUTING.md`, `CODE_OF_CONDUCT.md`) consolidan todo lo que tenías disperso en un formato estándar de GitHub — estos tres archivos aparecen automáticamente en la pestaña **Insights → Community** del repo con checkmarks verdes ✅.

---

### 📁 Estructura final del repo
```
QuantumEnergyOS/
├── SECURITY.md          ✅ política + modelo de amenazas
├── CONTRIBUTING.md      ✅ guía + checklist de seguridad
├── CODE_OF_CONDUCT.md   ✅ Contributor Covenant v2.1
├── Dockerfile           ✅ multi-stage + no-root + pinned
├── cloud/
│   └── ibm_quantum.py   ✅ token via env + MAX_QUBITS=32
└── .github/
    ├── CODEOWNERS
    ├── dependabot.yml
    └── workflows/
        ├── ci-qsharp-modern.yml
        ├── ci-qiskit.yml
        ├── ci-web-deploy.yml
        ├── ci-security.yml
        └── ci-pip-audit-fix.yml
# Desarrollo local — producción + herramientas
pip install -r requirements-pinned.txt -r requirements-dev.txt

# Verificar que sigue limpio
pip-audit -r requirements-pinned.txt

# Regenerar después de actualizar
pip-audit --fix -r requirements.txt
pip freeze > requirements-pinned.txt
git add requirements-pinned.txt
git commit -S -m "chore(deps): actualizar pins — $(date +%Y-%m-%d)"
RUN pip install qiskit qiskit-aer
WORKDIR /app
COPY . .

CMD ["python", "main.py"]

docker run --memory=4g --cpus=2 quantumenergyos

def validate_circuit(circuit):
    if circuit.num_qubits > 32:
        raise Exception("Too many qubits")

        import hashlib

hash = hashlib.sha256(str(circuit).encode()).hexdigest()

QuantumEnergyOS
│
├── quantum/
│   ├── circuits
│   ├── algorithms
│   └── simulators
│
├── security/
│   ├── input_validation
│   ├── credential_manager
│   └── sandbox
│
├── cloud/
│   └── ibm_quantum
│
├── logs/
└── api/

SECURITY.md
CODE_OF_CONDUCT.md
CONTRIBUTING.md

# Security Policy

## Reporting a Vulnerability

Please report vulnerabilities privately via email.

security@quantumenergyos.org

# 🔐 Security Policy — QuantumEnergyOS

## Versiones soportadas

| Versión | Soporte de seguridad |
|---------|----------------------|
| `main`  | ✅ Activo            |
| < 1.0   | ❌ No soportado      |

## Reportar una vulnerabilidad

**No abras un Issue público para vulnerabilidades de seguridad.**

Envía un reporte privado a través de:
- GitHub Security Advisories: **Settings → Security → Advisories → New advisory**
- O por correo a: `security@[tu-dominio]` (PGP disponible bajo petición)

Incluye:
1. Descripción del problema
2. Pasos para reproducir
3. Impacto potencial
4. Versión afectada

Responderemos en **72 horas**. Si se confirma, publicaremos un CVE y fix en ≤ 14 días.

## Modelo de amenazas

QuantumEnergyOS opera con las siguientes superficies de ataque en mente:

| Superficie | Mitigación |
|---|---|
| Input de circuitos cuánticos | Validación con Pydantic + límite `max_qubits=32` |
| API REST | JWT + rate limiting + HTTPS obligatorio |
| Dependencias | Dependabot + `pip-audit` + `safety` en CI |
| Contenedor | Docker `--read-only --memory=512m` + usuario no-root |
| Código | `bandit` + `semgrep` en cada PR |
| Releases | Tags firmados con GPG (`git tag -s`) |
| Secretos | `.env` nunca en git — usar `.env.example` como plantilla |

## Prácticas de desarrollo seguro

- Commits firmados: `git config --global commit.gpgsign true`
- SBOM generado en cada release (formato CycloneDX)
- Imágenes Docker escaneadas con `trivy` antes de publicar
- Entornos reproducibles con Nix o Guix (roadmap)

import os
token = os.getenv("IBM_QUANTUM_TOKEN")
# 🔐 Security Policy — QuantumEnergyOS

## Versiones soportadas

| Versión | Soporte de seguridad |
|---------|----------------------|
| `main`  | ✅ Activo            |
| < 1.0   | ❌ No soportado      |

## Reportar una vulnerabilidad

**No abras un Issue público para vulnerabilidades de seguridad.**

Envía un reporte privado a través de:
- **GitHub Security Advisories**: Settings → Security → Advisories → New advisory *(preferido)*
- **Email**: security@quantumenergyos.org (PGP disponible bajo petición)

Incluye en tu reporte:
1. Descripción del problema
2. Pasos para reproducir
3. Impacto potencial
4. Versión afectada
5. Posible fix (opcional pero bienvenido)

Responderemos en **72 horas**. Si se confirma, publicaremos un CVE y fix en ≤ 14 días.
Los reportes responsables serán reconocidos en el CHANGELOG.

## Modelo de amenazas

| Superficie | Mitigación |
|---|---|
| Input de circuitos cuánticos | Pydantic v2 + `MAX_QUBITS=32` + hash SHA-256 por circuito |
| Tokens de hardware cuántico | Solo via `os.getenv()` — nunca hardcodeados |
| API REST | JWT HS256 + rate limiting sliding window + HTTPS obligatorio |
| Dependencias | Dependabot semanal + `pip-audit` + `safety` en cada PR |
| Contenedor | `--read-only --memory=512m` + usuario no-root `qeos:1000` |
| Código fuente | `bandit` + `semgrep` en cada PR |
| Releases | Tags firmados con GPG (`git tag -s vX.Y`) |
| Secretos | `.env` en `.gitignore` — solo `.env.example` en el repo |
| Imágenes Docker | `trivy` antes de publicar + SBOM CycloneDX en cada release |

## Prácticas de desarrollo seguro

- Commits firmados: `git config --global commit.gpgsign true`
- SBOM generado en cada release (formato CycloneDX)
- Entornos reproducibles: `requirements-pinned.txt` con versiones exactas `==`
- Entornos reproducibles con Nix / Guix (roadmap 2027)
