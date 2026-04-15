use std::collections::VecDeque;
use std::sync::{Arc, Mutex};
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Debug, Clone)]
pub struct EnergyMetrics {
    pub timestamp: u64,
    pub power_consumption: f64,
    pub efficiency: f64,
    pub temperature: f64,
    pub load_factor: f64,
}

#[derive(Debug, Clone)]
pub struct Photon {
    pub id: u64,
    pub wavelength: f64,
    pub energy: f64,
    pub polarization: f64,
    pub phase: f64,
}

pub struct PhotonicCore {
    energy_queue: Arc<Mutex<VecDeque<EnergyMetrics>>>,
    active_photons: Arc<Mutex<Vec<Photon>>>,
    core_temperature: Arc<Mutex<f64>>,
    efficiency_rating: Arc<Mutex<f64>>,
    total_energy_saved: Arc<Mutex<f64>>,
}

impl PhotonicCore {
    pub fn new() -> Self {
        Self {
            energy_queue: Arc::new(Mutex::new(VecDeque::with_capacity(1000))),
            active_photons: Arc::new(Mutex::new(Vec::new())),
            core_temperature: Arc::new(Mutex::new(45.0)),
            efficiency_rating: Arc::new(Mutex::new(0.92)),
            total_energy_saved: Arc::new(Mutex::new(0.0)),
        }
    }

    pub fn record_energy_metrics(&self, metrics: EnergyMetrics) -> Result<(), String> {
        let mut queue = self.energy_queue.lock().map_err(|e| e.to_string())?;

        if queue.len() >= 1000 {
            queue.pop_front();
        }

        queue.push_back(metrics);
        Ok(())
    }

    pub fn get_latest_metrics(&self) -> Result<Option<EnergyMetrics>, String> {
        let queue = self.energy_queue.lock().map_err(|e| e.to_string())?;
        Ok(queue.back().cloned())
    }

    pub fn emit_photon(&self, wavelength: f64, polarization: f64) -> Result<Photon, String> {
        let energy = 1.2398 / wavelength;

        let photon = Photon {
            id: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .map_err(|e| e.to_string())?
                .as_nanos() as u64,
            wavelength,
            energy,
            polarization,
            phase: 0.0,
        };

        let mut photons = self.active_photons.lock().map_err(|e| e.to_string())?;
        photons.push(photon.clone());

        Ok(photon)
    }

    pub fn process_photons(&self) -> Result<f64, String> {
        let mut photons = self.active_photons.lock().map_err(|e| e.to_string())?;
        let mut total_energy = 0.0;

        for photon in photons.iter() {
            total_energy += photon.energy;
        }

        photons.clear();

        let mut temp = self.core_temperature.lock().map_err(|e| e.to_string())?;
        *temp = 45.0 + (total_energy * 0.1);

        let mut saved = self.total_energy_saved.lock().map_err(|e| e.to_string())?;
        *saved += total_energy * 0.15;

        Ok(total_energy)
    }

    pub fn optimize_energy_flow(&self, input_power: f64) -> Result<f64, String> {
        let efficiency = *self.efficiency_rating.lock().map_err(|e| e.to_string())?;

        let optimized = input_power * efficiency;

        let temp = self.core_temperature.lock().map_err(|e| e.to_string())?;
        if *temp > 80.0 {
            drop(temp);
            let mut eff = self.efficiency_rating.lock().map_err(|e| e.to_string())?;
            *eff *= 0.95;
        }

        Ok(optimized)
    }

    pub fn predict_load(&self, historical_data: &[f64]) -> f64 {
        if historical_data.is_empty() {
            return 0.5;
        }

        let sum: f64 = historical_data.iter().sum();
        let avg = sum / historical_data.len() as f64;

        let variance: f64 = historical_data
            .iter()
            .map(|x| (x - avg).powi(2))
            .sum::<f64>()
            / historical_data.len() as f64;

        let trend = if historical_data.len() > 1 {
            historical_data.last().unwrap() - historical_data.first().unwrap()
        } else {
            0.0
        };

        let prediction = avg + (trend * 0.3);

        prediction.clamp(0.0, 1.0)
    }

    pub fn get_core_status(&self) -> Result<HashMap<String, String>, String> {
        let mut status = HashMap::new();

        let queue = self.energy_queue.lock().map_err(|e| e.to_string())?;
        status.insert("metrics_count".to_string(), queue.len().to_string());

        let photons = self.active_photons.lock().map_err(|e| e.to_string())?;
        status.insert("active_photons".to_string(), photons.len().to_string());

        let temp = self.core_temperature.lock().map_err(|e| e.to_string())?;
        status.insert("temperature_c".to_string(), format!("{:.1}", *temp));

        let eff = self.efficiency_rating.lock().map_err(|e| e.to_string())?;
        status.insert("efficiency".to_string(), format!("{:.1}%", *eff * 100.0));

        let saved = self.total_energy_saved.lock().map_err(|e| e.to_string())?;
        status.insert("total_saved_kwh".to_string(), format!("{:.2}", *saved));

        Ok(status)
    }

    pub fn emergency_shutdown(&self) -> Result<(), String> {
        let mut photons = self.active_photons.lock().map_err(|e| e.to_string())?;
        photons.clear();

        let mut temp = self.core_temperature.lock().map_err(|e| e.to_string())?;
        *temp = 25.0;

        Ok(())
    }
}

impl Default for PhotonicCore {
    fn default() -> Self {
        Self::new()
    }
}

use std::collections::HashMap;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_photonic_core_initialization() {
        let core = PhotonicCore::new();
        let status = core.get_core_status().unwrap();
        assert!(status.contains_key("temperature_c"));
    }

    #[test]
    fn test_photon_emission() {
        let core = PhotonicCore::new();
        let photon = core.emit_photon(1550.0, 0.0).unwrap();
        assert!(photon.energy > 0.0);
    }
}
