"""
Quantum Climate Processor
Implements QUBO optimization for climate-aware grid distribution
"""

import time
import random
import numpy as np
from typing import Dict, List, Optional, Tuple
from dataclasses import dataclass
from datetime import datetime

from .models import (
    ClimateData, GridNode, NodeState, RiskAssessment, 
    RiskLevel, OptimizationResult
)


@dataclass
class QUBOMatrix:
    linear: np.ndarray
    quadratic: np.ndarray

    @classmethod
    def zeros(cls, n_vars: int) -> "QUBOMatrix":
        return cls(
            linear=np.zeros(n_vars),
            quadratic=np.zeros((n_vars, n_vars))
        )


class QuantumClimateOptimizer:
    def __init__(
        self, 
        temp_threshold: float = 45.0,
        use_qiskit: bool = False,
        simulation_mode: bool = True
    ):
        self.temp_threshold = temp_threshold
        self.use_qiskit = use_qiskit
        self.simulation_mode = simulation_mode

    def calculate_risk_score(
        self, 
        climate: ClimateData, 
        grid_nodes: List[GridNode]
    ) -> Tuple[float, RiskLevel]:
        temp_risk = (climate.temperature / self.temp_threshold) ** 2
        
        total_load = sum(n.current_load for n in grid_nodes)
        total_capacity = sum(n.capacity for n in grid_nodes)
        load_factor = total_load / total_capacity if total_capacity > 0 else 0
        
        base_risk = temp_risk * load_factor * 100
        
        critical_nodes = sum(1 for n in grid_nodes if n.state == NodeState.CRITICAL)
        node_penalty = critical_nodes * 10
        
        risk_score = min(100, base_risk + node_penalty)
        
        if risk_score >= 75:
            level = RiskLevel.EXTREME
        elif risk_score >= 50:
            level = RiskLevel.HIGH
        elif risk_score >= 25:
            level = RiskLevel.MEDIUM
        else:
            level = RiskLevel.LOW
            
        return risk_score, level

    def build_qubo_matrix(
        self, 
        climate: ClimateData, 
        grid_nodes: List[GridNode]
    ) -> QUBOMatrix:
        n = len(grid_nodes)
        qubo = QUBOMatrix.zeros(n)
        
        for i, node in enumerate(grid_nodes):
            load_contribution = (node.current_load / node.capacity)
            temp_contribution = (climate.temperature / 50.0)
            
            qubo.linear[i] = load_contribution + temp_contribution
            
            for j, other in enumerate(grid_nodes):
                if i != j:
                    distance = abs(i - j) * 0.1
                    qubo.quadratic[i, j] = distance
        
        return qubo

    def solve_qubo(
        self, 
        qubo: QUBOMatrix,
        num_reads: int = 100
    ) -> Dict[str, int]:
        if self.simulation_mode:
            return self._simulated_annealing(qubo)
        
        if self.use_qiskit:
            return self._solve_with_qiskit(qubo, num_reads)
        
        return self._greedy_solution(qubo)

    def _simulated_annealing(self, qubo: QUBOMatrix) -> Dict[str, int]:
        n = len(qubo.linear)
        current_state = np.random.randint(0, 2, n)
        
        def energy(state: np.ndarray) -> float:
            linear_term = np.dot(qubo.linear, state)
            quadratic_term = np.sum(
                qubo.quadratic * np.outer(state, state)
            )
            return linear_term + 0.5 * quadratic_term
        
        current_energy = energy(current_state)
        temperature = 1.0
        
        for _ in range(1000):
            idx = random.randint(0, n - 1)
            new_state = current_state.copy()
            new_state[idx] = 1 - new_state[idx]
            
            new_energy = energy(new_state)
            delta = new_energy - current_energy
            
            if delta < 0 or random.random() < np.exp(-delta / temperature):
                current_state = new_state
                current_energy = new_energy
            
            temperature *= 0.995
        
        return {f"node_{i}": int(state) for i, state in enumerate(current_state)}

    def _greedy_solution(self, qubo: QUBOMatrix) -> Dict[str, int]:
        n = len(qubo.linear)
        scores = qubo.linear.copy()
        
        for i in range(n):
            for j in range(i + 1, n):
                scores[i] += qubo.quadratic[i, j]
        
        threshold = np.median(scores)
        return {f"node_{i}": 1 if scores[i] > threshold else 0 for i in range(n)}

    def _solve_with_qiskit(self, qubo: QUBOMatrix, num_reads: int) -> Dict[str, int]:
        try:
            from qiskit_optimization import QuadraticProgram
            from qiskit_optimization.algorithms import MinimumEigenOptimizer
            from qiskit_algorithms import QAOA
            from qiskit.primitives import Sampler
            
            qp = QuadraticProgram("ClimateGridOptimization")
            for i in range(len(qubo.linear)):
                qp.binary_var(name=f"node_{i}")
            
            linear_dict = {f"node_{i}": qubo.linear[i] for i in range(len(qubo.linear))}
            quadratic_dict = {}
            for i in range(len(qubo.linear)):
                for j in range(len(qubo.linear)):
                    if qubo.quadratic[i, j] != 0:
                        quadratic_dict[(f"node_{i}", f"node_{j}")] = qubo.quadratic[i, j]
            
            qp.minimize(linear=linear_dict, quadratic=quadratic_dict)
            
            sampler = Sampler()
            qaoa = QAOA(sampler=sampler, reps=2)
            optimizer = MinimumEigenOptimizer(qaoa)
            result = optimizer.solve(qp)
            
            return {f"node_{i}": int(result.x[i]) for i in range(len(result.x))}
            
        except ImportError:
            return self._simulated_annealing(qubo)

    def optimize_grid(
        self, 
        climate: ClimateData, 
        grid_nodes: List[GridNode]
    ) -> OptimizationResult:
        start_time = time.time()
        
        qubo = self.build_qubo_matrix(climate, grid_nodes)
        solution = self.solve_qubo(qubo)
        
        energy_savings = sum(
            node.current_load * 0.1 for i, node in enumerate(grid_nodes) 
            if solution.get(f"node_{i}", 0) == 1
        )
        
        risk_score, _ = self.calculate_risk_score(climate, grid_nodes)
        risk_reduction = risk_score * 0.3
        
        solution_time_ms = (time.time() - start_time) * 1000
        
        return OptimizationResult(
            optimal_states=solution,
            energy_savings=energy_savings,
            risk_reduction=risk_reduction,
            quantum_solution_time_ms=solution_time_ms
        )

    def assess_risk(
        self, 
        climate: ClimateData, 
        grid_nodes: List[GridNode]
    ) -> RiskAssessment:
        risk_score, risk_level = self.calculate_risk_score(climate, grid_nodes)
        
        affected = [
            node.id for node in grid_nodes 
            if node.state in [NodeState.CRITICAL, NodeState.POWER_SAVE]
        ]
        
        actions = []
        if risk_level == RiskLevel.EXTREME:
            actions = [
                "Activate emergency load shedding",
                "Notify CENACE of potential grid stress",
                "Enable thermal emergency protocol"
            ]
        elif risk_level == RiskLevel.HIGH:
            actions = [
                "Reduce non-critical load by 20%",
                "Activate power-save mode on all nodes"
            ]
        elif risk_level == RiskLevel.MEDIUM:
            actions = [
                "Monitor temperature closely",
                "Prepare contingency plans"
            ]
        
        predicted_peak = climate.temperature + 5.0
        
        return RiskAssessment(
            risk_score=risk_score,
            risk_level=risk_level,
            affected_nodes=affected,
            recommended_actions=actions,
            predicted_peak_temp=predicted_peak
        )
