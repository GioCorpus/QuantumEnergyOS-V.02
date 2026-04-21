"""
climate_orchestrator/actions.py — Registry de acciones predefinidas
"""
from __future__ import annotations

from dataclasses import dataclass, field
from typing import Dict, List

@dataclass
class Action:
    """Acción ejecutable por el orquestador."""
    id: str
    name: str
    description: str
    impact: str  # "bajo", "medio", "alto", "crítico"
    command: str  # categoría de comando
    dry_run: bool = True
    executed: bool = False
    result: str | None = None

    def to_dict(self) -> dict:
        d = {
            "id": self.id,
            "name": self.name,
            "description": self.description,
            "impact": self.impact,
            "command": self.command,
            "executed": self.executed,
            "dry_run": self.dry_run,
        }
        if self.result:
            d["result"] = self.result
        if self.dry_run:
            d["note"] = "[DRY RUN] La acción no se ejecutó realmente"
        return d

class ActionRegistry:
    """Registro central de todas las acciones disponibles."""

    def __init__(self):
        self._actions: Dict[str, Action] = {}

    def register(self, action: Action) -> None:
        self._actions[action.id] = action

    def get(self, action_id: str) -> Action | None:
        return self._actions.get(action_id)

    def list(self) -> List[Action]:
        return list(self._actions.values())

    def clear(self) -> None:
        self._actions.clear()
