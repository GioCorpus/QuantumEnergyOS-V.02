#!/bin/sh
# ═══════════════════════════════════════════════════════════════════════
#  QuantumEnergyOS — Splash Screen
#  "QuantumEnergyOS – Mexicali no se apaga"
#  Guilty Gear-style rock vibes (Daisuke Ishiwatari)
# ═══════════════════════════════════════════════════════════════════════

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
CYAN='\033[0;36m'
MAGENTA='\033[0;35m'
NC='\033[0m'

# Clear screen
clear

# Play rock music in background (if available)
if [ -f "/usr/share/quantum-compositor/quantum-rock.ogg" ]; then
    if command -v aplay &> /dev/null; then
        aplay /usr/share/quantum-compositor/quantum-rock.ogg 2>/dev/null &
    elif command -v paplay &> /dev/null; then
        paplay /usr/share/quantum-compositor/quantum-rock.ogg 2>/dev/null &
    fi
fi

# Splash animation
echo ""
echo -e "${CYAN}═══════════════════════════════════════════════════════════════${NC}"
echo -e "${CYAN}                                                               ${NC}"
echo -e "${CYAN}   ██████╗ ██╗   ██╗ █████╗ ███╗   ██╗████████╗██╗   ██╗███╗ ${NC}"
echo -e "${CYAN}  ██╔═══██╗██║   ██║██╔══██╗████╗  ██║╚══██╔══╝██║   ██║████╗${NC}"
echo -e "${CYAN}  ██║   ██║██║   ██║███████║██╔██╗ ██║   ██║   ██║   ██║██╔██╗${NC}"
echo -e "${CYAN}  ██║▄▄ ██║██║   ██║██╔══██║██║╚██╗██║   ██║   ██║   ██║██║╚██╗${NC}"
echo -e "${CYAN}  ╚██████╔╝╚██████╔╝██║  ██║██║ ╚████║   ██║   ╚██████╔╝██║ ╚████╗${NC}"
echo -e "${CYAN}   ╚══▀▀═╝  ╚═════╝ ╚═╝  ╚═╝╚═╝  ╚═══╝   ╚═╝    ╚═════╝ ╚═╝  ╚═══╝${NC}"
echo -e "${CYAN}                                                               ${NC}"
echo -e "${CYAN}  ███████╗███╗   ██╗███████╗██████╗  ██████╗ ███████╗          ${NC}"
echo -e "${CYAN}  ██╔════╝████╗  ██║██╔════╝██╔══██╗██╔═══██╗██╔════╝          ${NC}"
echo -e "${CYAN}  █████╗  ██╔██╗ ██║█████╗  ██████╔╝██║   ██║███████╗          ${NC}"
echo -e "${CYAN}  ██╔══╝  ██║╚██╗██║██╔══╝  ██╔══██╗██║   ██║╚════██║          ${NC}"
echo -e "${CYAN}  ███████╗██║ ╚████║███████╗██║  ██║╚██████╔╝███████║          ${NC}"
echo -e "${CYAN}  ╚══════╝╚═╝  ╚═══╝╚══════╝╚═╝  ╚═╝ ╚═════╝ ╚══════╝          ${NC}"
echo -e "${CYAN}                                                               ${NC}"
echo -e "${CYAN}═══════════════════════════════════════════════════════════════${NC}"
echo ""

# Animated message
echo -e "${YELLOW}  ╔═══════════════════════════════════════════════════════════╗${NC}"
echo -e "${YELLOW}  ║                                                           ║${NC}"
echo -e "${YELLOW}  ║   ${MAGENTA}QuantumEnergyOS – Mexicali no se apaga${YELLOW}              ║${NC}"
echo -e "${YELLOW}  ║                                                           ║${NC}"
echo -e "${YELLOW}  ║   ${CYAN}Desde el desierto, para el mundo${YELLOW}                    ║${NC}"
echo -e "${YELLOW}  ║   ${CYAN}Kardashev 0 → 1${YELLOW}                                    ║${NC}"
echo -e "${YELLOW}  ║                                                           ║${NC}"
echo -e "${YELLOW}  ╚═══════════════════════════════════════════════════════════╝${NC}"
echo ""

# Loading animation
echo -ne "${GREEN}  Iniciando sistema cuántico${NC}"
for i in 1 2 3 4 5; do
    echo -ne "${GREEN}.${NC}"
    sleep 0.3
done
echo ""

echo -ne "${GREEN}  Cargando compositor Wayland${NC}"
for i in 1 2 3 4 5; do
    echo -ne "${GREEN}.${NC}"
    sleep 0.3
done
echo ""

echo -ne "${GREEN}  Conectando predictor de energía${NC}"
for i in 1 2 3 4 5; do
    echo -ne "${GREEN}.${NC}"
    sleep 0.3
done
echo ""

echo ""
echo -e "${CYAN}  ════════════════════════════════════════════════════════════${NC}"
echo -e "${GREEN}  ✓ Sistema listo. Iniciando escritorio...${NC}"
echo -e "${CYAN}  ════════════════════════════════════════════════════════════${NC}"
echo ""

sleep 2
