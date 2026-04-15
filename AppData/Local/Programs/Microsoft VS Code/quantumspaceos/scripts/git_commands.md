# QuantumSpaceOS Git Setup and Commit Commands
# =============================================
# Author: Giovanny Corpus Bernal - Mexicali, Baja California
# Motto: "No me contrataron en la Tierra, así que construí algo que pertenece a Marte"

# ==============================================================================
# CONFIGURACIÓN INICIAL (Si el repositorio no está inicializado)
# ==============================================================================

# 1. Inicializar repositorio Git (si no existe)
git init

# 2. Configurar usuario (reemplaza con tu información)
git config user.name "Giovanny Corpus Bernal"
git config user.email "giovanny.corpus@example.com"

# ==============================================================================
# PREPARACIÓN DEL REPOSITORIO
# ==============================================================================

# 3. Ver estado actual
git status

# 4. Crear archivo .gitignore para QuantumSpaceOS
@"
# QuantumSpaceOS .gitignore

# Build outputs
iso/out/
iso/work/
*.iso
*.img

# IDE
.vscode/
.idea/
*.swp
*.swo

# OS
.DS_Store
Thumbs.db

# Dependencies
node_modules/
venv/
.env

# Logs
*.log

# Rust
target/
Cargo.lock

# Python
__pycache__/
*.pyc
.pytest_cache/

# Temporary
*.tmp
*.bak
"@ | Out-File -FilePath ".gitignore" -Encoding UTF8

# ==============================================================================
# AGREGAR ARCHIVOS AL STAGING
# ==============================================================================

# 5. Agregar todos los archivos al staging
git add .

# 6. Ver qué archivos están en staging
git status

# 7. Agregar archivos específicos (opcional)
# git add README.md
# git add src/photonicq-bridge/
# git add src/flight-sim/
# git add src/api/

# ==============================================================================
# CREAR COMMITS
# ==============================================================================

# 8. Hacer commit inicial con todos los cambios
git commit -m @"
Initial commit: QuantumSpaceOS v1.0 - Del Desierto de Mexicali al Espacio

- README.md con narrativa épica y documentación completa
- Estructura completa del proyecto
- photonicq-bridge: Puente fotónico-cuántico para comunicación espacial
- orbital_mechanics.rs: Simulación de vuelo y mecánica orbital
- telemetry_api.py: API REST de telemetría espacial
- Scripts de construcción ISO (PowerShell)
- Scripts de ejecución QEMU
- Configuración del sistema base

Autor: Giovanny Corpus Bernal
Ubicación: Mexicali, Baja California, México
Motto: "No me contrataron en la Tierra, así que construí algo que pertenece a Marte"
"@

# ==============================================================================
# CONFIGURAR REMOTO (Reemplaza con tu URL de GitHub)
# ==============================================================================

# 9. Agregar remoto (reemplaza URL con la tuya)
git remote add origin https://github.com/TU_USUARIO/QuantumSpaceOS.git

# 10. Ver remotos configurados
git remote -v

# ==============================================================================
# CREAR BRANCH Y PUSH
# ==============================================================================

# 11. Renombrar branch principal a main (opcional)
git branch -M main

# 12. Push inicial (primer push requiere -u)
git push -u origin main

# ==============================================================================
# COMANDOS ADICIONALES
# ==============================================================================

# Ver historial de commits
git log --oneline --graph --decorate

# Ver diferencias
git diff
git diff --staged

# deshacer último commit (mantener cambios)
git reset --soft HEAD~1

# deshacer último commit (eliminar cambios)
git reset --hard HEAD~1

# ==============================================================================
# FLUJO DE TRABAJO RECOMENDADO
# ==============================================================================

# Para futuras modificaciones:
# 1. git status                          # Ver cambios
# 2. git add .                           # Agregar cambios
# 3. git commit -m "Descripción"         # Crear commit
# 4. git push                            # Subir cambios

# ==============================================================================
# EJEMPLO: COMMIT DE UNA NUEVA CARACTERÍSTICA
# ==============================================================================

# git add src/quantum-core/
# git commit -m "feat: Add quantum core module with qubit simulation"
# git push

# ==============================================================================
# NOTAS IMPORTANTES
# ==============================================================================

# - Asegúrate de tener instalado Git en tu sistema
# - Crea un repositorio en GitHub antes de hacer push
# - Reemplaza "TU_USUARIO" con tu nombre de usuario de GitHub
# - Para SSH usa: git@github.com:TU_USUARIO/QuantumSpaceOS.git
# - Nunca hagas commit de archivos sensibles (.env, credenciales, etc.)