# 🛠️ Guía de Contribución — QuantumEnergyOS

¡Gracias por querer contribuir! Este proyecto existe para que la energía limpia llegue al noroeste de México y al mundo. Cada PR cuenta.

## Antes de empezar

1. Lee el [Código de Conducta](CODE_OF_CONDUCT.md)
2. Revisa los [Issues abiertos](https://github.com/GioCorpus/QuantumEnergyOS/issues)
3. Para cambios grandes, abre un Issue primero para discutir

## Setup local

```bash
# 1. Fork + clonar
git clone https://github.com/TU_USUARIO/QuantumEnergyOS.git
cd QuantumEnergyOS

# 2. Entorno virtual
python3 -m venv .venv
source .venv/bin/activate        # Linux/Mac
# .venv\Scripts\activate         # Windows

# 3. Instalar dependencias
pip install -r requirements-pinned.txt -r requirements-dev.txt

# 4. Configurar secretos (nunca hardcodear)
cp .env.example .env
# Editar .env con tus tokens

# 5. Activar commits firmados
git config commit.gpgsign true
```

## Flujo de trabajo

```bash
# Rama descriptiva
git checkout -b feat/optimizar-qaoa-capas

# Desarrollar + testear
pytest tests/ -v
bandit -r . --severity-level medium
pip-audit -r requirements-pinned.txt

# Commit firmado
git commit -S -m "feat(quantum): optimizar capas QAOA para redes de 8 nodos"

# Push + PR hacia main
git push origin feat/optimizar-qaoa-capas
```

## Reglas de seguridad para contribuidores

| Regla | Detalle |
|---|---|
| **Nunca tokens en código** | Usar siempre `os.getenv("IBM_QUANTUM_TOKEN")` |
| **MAX_QUBITS = 32** | No elevar este límite sin discusión en Issue |
| **Validar inputs** | Todo input externo pasa por `security/validation.py` |
| **Sin `.env` en git** | Está en `.gitignore` — usa `.env.example` |
| **Tests obligatorios** | Todo nuevo código necesita tests en `tests/` |

## Convención de commits

```
feat(scope):     nueva funcionalidad
fix(scope):      corrección de bug
fix(security):   parche de seguridad — prioridad máxima
chore(deps):     actualización de dependencias
docs:            solo documentación
test:            solo tests
refactor:        refactoring sin cambio de comportamiento
```

## Checklist antes de abrir PR

- [ ] Tests pasan: `pytest tests/ -v`
- [ ] Sin vulnerabilidades: `pip-audit -r requirements-pinned.txt`
- [ ] Sin problemas de seguridad: `bandit -r . --severity-level medium`
- [ ] Sin errores de lint: `ruff check .`
- [ ] Commit firmado con GPG
- [ ] `requirements-pinned.txt` actualizado si se agregaron deps
- [ ] Documentación actualizada si aplica

## Áreas donde puedes contribuir

- 🔬 **Algoritmos Q#** — mejorar QAOA, VQE, simulación de fusión
- 🐍 **Notebooks Python** — más visualizaciones, análisis de resultados
- 🔐 **Seguridad** — auditorías, mejoras al sandbox
- 📖 **Documentación** — ejemplos, tutoriales, traducciones
- 🧪 **Tests** — más cobertura, tests de integración con Qiskit Aer
- 🌐 **Demo web** — mejorar `index.html` y `api-simulada.js`
