use std::collections::{HashMap, VecDeque};
use std::sync::{Arc, Mutex};
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Debug, Clone)]
pub struct EnergyNode {
    pub id: u32,
    pub current_load: f64,
    pub capacity: f64,
    pub efficiency: f64,
    pub status: NodeStatus,
}

#[derive(Debug, Clone, PartialEq)]
pub enum NodeStatus {
    Optimal,
    Warning,
    Critical,
    Offline,
}

#[derive(Debug, Clone)]
pub struct OptimizationResult {
    pub recommended_actions: Vec<OptimizationAction>,
    pub predicted_savings: f64,
    pub risk_level: f64,
    pub confidence: f64,
}

#[derive(Debug, Clone)]
pub struct OptimizationAction {
    pub node_id: u32,
    pub action_type: ActionType,
    pub expected_impact: f64,
}

#[derive(Debug, Clone)]
pub enum ActionType {
    ReduceLoad,
    IncreaseCapacity,
    Rebalance,
    EmergencyCutoff,
}

pub struct EnergyOptimizer {
    nodes: Arc<Mutex<HashMap<u32, EnergyNode>>>,
    history: Arc<Mutex<VecDeque<(u64, f64)>>>,
    threshold_warning: f64,
    threshold_critical: f64,
    optimization_interval: u64,
}

impl EnergyOptimizer {
    pub fn new() -> Self {
        Self {
            nodes: Arc::new(Mutex::new(HashMap::new())),
            history: Arc::new(Mutex::new(VecDeque::with_capacity(10000))),
            threshold_warning: 0.75,
            threshold_critical: 0.90,
            optimization_interval: 300,
        }
    }

    pub fn register_node(&self, id: u32, capacity: f64) -> Result<(), String> {
        let mut nodes = self.nodes.lock().map_err(|e| e.to_string())?;

        let node = EnergyNode {
            id,
            current_load: 0.0,
            capacity,
            efficiency: 0.95,
            status: NodeStatus::Optimal,
        };

        nodes.insert(id, node);
        Ok(())
    }

    pub fn update_node_load(&self, node_id: u32, load: f64) -> Result<(), String> {
        let mut nodes = self.nodes.lock().map_err(|e| e.to_string())?;

        let node = nodes.get_mut(&node_id).ok_or("Node not found")?;

        node.current_load = load;
        node.status = self.calculate_node_status(load, node.capacity);

        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_err(|e| e.to_string())?
            .as_secs();

        let utilization = load / node.capacity;

        let mut history = self.history.lock().map_err(|e| e.to_string())?;
        if history.len() >= 10000 {
            history.pop_front();
        }
        history.push_back((timestamp, utilization));

        Ok(())
    }

    fn calculate_node_status(&self, load: f64, capacity: f64) -> NodeStatus {
        let utilization = load / capacity;

        if utilization >= self.threshold_critical {
            NodeStatus::Critical
        } else if utilization >= self.threshold_warning {
            NodeStatus::Warning
        } else {
            NodeStatus::Optimal
        }
    }

    pub fn optimize(&self) -> Result<OptimizationResult, String> {
        let nodes = self.nodes.lock().map_err(|e| e.to_string())?;

        let mut actions = Vec::new();
        let mut total_risk = 0.0;
        let mut predicted_savings = 0.0;

        for (id, node) in nodes.iter() {
            let utilization = node.current_load / node.capacity;

            total_risk += match node.status {
                NodeStatus::Critical => 1.0,
                NodeStatus::Warning => 0.5,
                _ => 0.1,
            };

            if node.status == NodeStatus::Critical {
                let target_reduction = node.current_load * 0.2;
                actions.push(OptimizationAction {
                    node_id: *id,
                    action_type: ActionType::ReduceLoad,
                    expected_impact: target_reduction,
                });
                predicted_savings += target_reduction * node.efficiency;
            } else if node.status == NodeStatus::Warning {
                actions.push(OptimizationAction {
                    node_id: *id,
                    action_type: ActionType::Rebalance,
                    expected_impact: node.current_load * 0.1,
                });
            }
        }

        let risk_level = (total_risk / nodes.len() as f64).min(1.0);
        let confidence = 0.85;

        Ok(OptimizationResult {
            recommended_actions: actions,
            predicted_savings,
            risk_level,
            confidence,
        })
    }

    pub fn predict_outage_risk(&self) -> Result<f64, String> {
        let history = self.history.lock().map_err(|e| e.to_string())?;

        if history.len() < 10 {
            return Ok(0.0);
        }

        let recent: Vec<f64> = history.iter().rev().take(100).map(|(_, u)| *u).collect();

        let avg_utilization: f64 = recent.iter().sum::<f64>() / recent.len() as f64;

        let variance: f64 = recent
            .iter()
            .map(|u| (u - avg_utilization).powi(2))
            .sum::<f64>()
            / recent.len() as f64;

        let trend = if recent.len() > 1 {
            recent[recent.len() - 1] - recent[0]
        } else {
            0.0
        };

        let risk = if avg_utilization > self.threshold_critical {
            0.9
        } else if avg_utilization > self.threshold_warning {
            0.5 + (variance * 0.3)
        } else {
            0.2
        };

        if trend > 0.1 {
            return Ok((risk + 0.2).min(1.0));
        }

        Ok(risk)
    }

    pub fn get_load_distribution(&self) -> Result<HashMap<u32, f64>, String> {
        let nodes = self.nodes.lock().map_err(|e| e.to_string())?;

        let mut distribution = HashMap::new();

        for (id, node) in nodes.iter() {
            distribution.insert(*id, node.current_load / node.capacity);
        }

        Ok(distribution)
    }

    pub fn rebalance_load(&self) -> Result<HashMap<u32, f64>, String> {
        let mut nodes = self.nodes.lock().map_err(|e| e.to_string())?;

        let total_capacity: f64 = nodes.values().map(|n| n.capacity).sum();
        let total_load: f64 = nodes.values().map(|n| n.current_load).sum();

        if total_capacity == 0.0 {
            return Err("No capacity available".to_string());
        }

        let target_per_node = total_load / nodes.len() as f64;

        let mut new_allocations = HashMap::new();

        for (id, node) in nodes.iter_mut() {
            let difference = target_per_node - node.current_load;
            let adjusted = node.current_load + (difference * node.efficiency);

            new_allocations.insert(*id, adjusted.clamp(0.0, node.capacity));

            node.current_load = new_allocations[id];
        }

        Ok(new_allocations)
    }

    pub fn get_system_status(&self) -> Result<HashMap<String, String>, String> {
        let mut status = HashMap::new();

        let nodes = self.nodes.lock().map_err(|e| e.to_string())?;

        let optimal = nodes
            .values()
            .filter(|n| n.status == NodeStatus::Optimal)
            .count();
        let warning = nodes
            .values()
            .filter(|n| n.status == NodeStatus::Warning)
            .count();
        let critical = nodes
            .values()
            .filter(|n| n.status == NodeStatus::Critical)
            .count();

        status.insert("nodes_total".to_string(), nodes.len().to_string());
        status.insert("nodes_optimal".to_string(), optimal.to_string());
        status.insert("nodes_warning".to_string(), warning.to_string());
        status.insert("nodes_critical".to_string(), critical.to_string());

        let risk = self.predict_outage_risk()?;
        status.insert("outage_risk".to_string(), format!("{:.1}%", risk * 100.0));

        Ok(status)
    }
}

impl Default for EnergyOptimizer {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_optimizer_initialization() {
        let optimizer = EnergyOptimizer::new();
        let status = optimizer.get_system_status().unwrap();
        assert_eq!(status.get("nodes_total"), Some(&"0".to_string()));
    }

    #[test]
    fn test_node_registration() {
        let optimizer = EnergyOptimizer::new();
        optimizer.register_node(1, 100.0).unwrap();
        let distribution = optimizer.get_load_distribution().unwrap();
        assert!(distribution.contains_key(&1));
    }
}
