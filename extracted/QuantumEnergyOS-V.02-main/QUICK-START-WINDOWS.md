# QuantumEnergyOS — Inicio Rápido en Windows

## ⚡ Comandos Esenciales

### 1. Preparar Entorno (Una sola vez)

```powershell
# Abrir PowerShell como Administrador
Set-ExecutionPolicy Bypass -Scope Process -Force

# Navegar al proyecto
cd "C:\Users\HP\Documents\Documentos Personales GACB\Demo\QuantumEnergyOS\QuantumEnergyOS-main"
```

### 2. Instalar Dependencias

```powershell
# Opción A: Windows ADK (Recomendado)
# Descargar desde: https://learn.microsoft.com/en-us/windows-hardware/get-started/adk-install
# Seleccionar "Deployment Tools" durante instalación

# Opción B: Chocolatey (Más fácil)
choco install mkisofs gzip -y

# Opción C: Scoop
scoop install mkisofs gzip
```

### 3. Construir la ISO

```powershell
# Build estándar
.\build-iso-windows.ps1

# Build rápido (sin compilar compositor)
.\build-iso-windows.ps1 -SkipCompositor

# Build con salida personalizada
.\build-iso-windows.ps1 -OutputDir "C:\ISOs"
```

### 4. Probar la ISO

```powershell
# Con QEMU (Recomendado)
choco install qemu -y
qemu-system-x86_64 -m 4G -cdrom .\output\QuantumEnergyOS-v1.0.0-live-amd64.iso -boot d

# O usar VirtualBox/VMware
```

### 5. Grabar en USB

```powershell
# Usar Rufus (https://rufus.ie)
# O balenaEtcher (https://www.balena.io/etcher/)
```

## 📋 Checklist Rápido

- [ ] PowerShell abierto como Administrador
- [ ] Windows ADK o mkisofs instalado
- [ ] gzip instalado
- [ ] Al menos 10 GB libres en disco
- [ ] Conexión a internet activa

## 🎯 Resultado Esperado

```
.\output\QuantumEnergyOS-v1.0.0-live-amd64.iso (~2.5 GB)
```

## 🆘 Solución Rápida de Problemas

| Problema | Solución |
|----------|----------|
| "oscdimg no encontrado" | Instalar Windows ADK con "Deployment Tools" |
| "gzip no encontrado" | `choco install gzip -y` |
| "Permiso denegado" | Ejecutar PowerShell como Administrador |
| "No hay espacio" | Liberar al menos 10 GB en disco |
| Build muy lento | Usar `-SkipCompositor` para build rápido |

## 📚 Documentación Completa

Ver: [`BUILD-ISO-WINDOWS.md`](BUILD-ISO-WINDOWS.md)

---

**⚡ QuantumEnergyOS — Desde Mexicali, BC — para el mundo entero**
