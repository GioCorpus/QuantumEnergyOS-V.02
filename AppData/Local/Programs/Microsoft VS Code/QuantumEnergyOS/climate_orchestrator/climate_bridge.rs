use std::sync::{Arc, Mutex};
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Debug, Clone)]
pub struct ClimateEvent {
    pub temperature: f32,
    pub humidity: f32,
    pub heat_index: f32,
    pub load_index: f32,
    pub status: ClimateStatus,
    pub timestamp: u64,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ClimateStatus {
    Normal,
    HeatWarning,
    HeatWave,
    ExtremeHeat,
    Critical,
}

impl ClimateStatus {
    pub fn as_str(&self) -> &str {
        match self {
            ClimateStatus::Normal => "normal",
            ClimateStatus::HeatWarning => "heat_warning",
            ClimateStatus::HeatWave => "heat_wave",
            ClimateStatus::ExtremeHeat => "extreme_heat",
            ClimateStatus::Critical => "critical",
        }
    }
}

pub struct ClimateBridge {
    temperature_threshold: f32,
    extreme_threshold: f32,
    current_event: Arc<Mutex<Option<ClimateEvent>>>,
    auto_scaling_enabled: Arc<Mutex<bool>>,
}

impl ClimateBridge {
    pub fn new() -> Self {
        Self {
            temperature_threshold: 45.0,
            extreme_threshold: 48.0,
            current_event: Arc::new(Mutex::new(None)),
            auto_scaling_enabled: Arc::new(Mutex::new(true)),
        }
    }

    pub fn with_thresholds(temperature_threshold: f32, extreme_threshold: f32) -> Self {
        Self {
            temperature_threshold,
            extreme_threshold,
            current_event: Arc::new(Mutex::new(None)),
            auto_scaling_enabled: Arc::new(Mutex::new(true)),
        }
    }

    fn get_timestamp() -> u64 {
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs()
    }

    fn calculate_heat_index(temperature: f32, humidity: f32) -> f32 {
        let t = temperature;
        let r = humidity;

        let mut hi = -8.78469475556 + 1.61139411 * t + 2.33854883889 * r;
        hi += -0.146116972970967 * t * r;
        hi += -0.0123080943270969 * t * t + -0.0164248277778 * r * r;
        hi += 0.00221173237033 * t * t * r;
        hi += 0.000725460474 * t * r * r;
        hi += -0.00000358273841156 * t * t * r * r;

        hi.max(t)
    }

    fn determine_status(&self, temperature: f32) -> ClimateStatus {
        if temperature >= self.extreme_threshold {
            ClimateStatus::Critical
        } else if temperature >= 46.0 {
            ClimateStatus::ExtremeHeat
        } else if temperature >= 45.0 {
            ClimateStatus::HeatWave
        } else if temperature >= 42.0 {
            ClimateStatus::HeatWarning
        } else {
            ClimateStatus::Normal
        }
    }

    pub fn process_climate_data(
        &self,
        temperature: f32,
        humidity: f32,
        grid_load: f32,
    ) -> ClimateEvent {
        let heat_index = Self::calculate_heat_index(temperature, humidity);
        let status = self.determine_status(temperature);

        let load_factor = grid_load / 100.0;
        let temp_factor = temperature / 50.0;
        let load_index = (load_factor * temp_factor * 100.0).min(100.0);

        let event = ClimateEvent {
            temperature,
            humidity,
            heat_index,
            load_index,
            status: status.clone(),
            timestamp: Self::get_timestamp(),
        };

        let mut current = self.current_event.lock().unwrap();
        *current = Some(event.clone());

        if *self.auto_scaling_enabled.lock().unwrap() {
            self.trigger_auto_scaling(&event);
        }

        event
    }

    pub fn trigger_auto_scaling(&self, event: &ClimateEvent) {
        match event.status {
            ClimateStatus::Critical => {
                println!("[CLIMATE] CRITICAL: Mexicali Extreme Heat detected!");
                println!(
                    "[CLIMATE]   Temperature: {:.1}°C, Heat Index: {:.1}°C",
                    event.temperature, event.heat_index
                );
                println!("[CLIMATE]   Load Index: {:.1}%", event.load_index);
                println!("[CLIMATE]   Action: Activating emergency thermal protocol");
                println!("[CLIMATE]   Action: Reducing non-critical loads by 40%");
            }
            ClimateStatus::ExtremeHeat => {
                println!("[CLIMATE] WARNING: Extreme heat conditions detected");
                println!("[CLIMATE]   Temperature: {:.1}°C", event.temperature);
                println!("[CLIMATE]   Action: Enabling power-save mode on all nodes");
            }
            ClimateStatus::HeatWave => {
                println!("[CLIMATE] ALERT: Heat wave active in Mexicali");
                println!("[CLIMATE]   Action: Activating thermal monitoring");
            }
            _ => {}
        }
    }

    pub fn get_current_event(&self) -> Option<ClimateEvent> {
        let event = self.current_event.lock().unwrap();
        event.clone()
    }

    pub fn set_auto_scaling(&self, enabled: bool) {
        let mut auto = self.auto_scaling_enabled.lock().unwrap();
        *auto = enabled;
    }

    pub fn is_auto_scaling_enabled(&self) -> bool {
        *self.auto_scaling_enabled.lock().unwrap()
    }

    pub fn get_bridge_status(&self) -> String {
        let event = self.current_event.lock().unwrap();
        match &*event {
            Some(e) => format!(
                "ClimateBridge: Temp={:.1}°C Status={} LoadIndex={:.1}%",
                e.temperature,
                e.status.as_str(),
                e.load_index
            ),
            None => "ClimateBridge: No data".to_string(),
        }
    }
}

impl Default for ClimateBridge {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_climate_bridge_creation() {
        let bridge = ClimateBridge::new();
        assert!(bridge.is_auto_scaling_enabled());
    }

    #[test]
    fn test_heat_index_calculation() {
        let hi = ClimateBridge::calculate_heat_index(40.0, 20.0);
        assert!(hi >= 40.0);
    }

    #[test]
    fn test_status_determination() {
        let bridge = ClimateBridge::new();

        assert_eq!(bridge.determine_status(40.0), ClimateStatus::HeatWarning);
        assert_eq!(bridge.determine_status(45.0), ClimateStatus::HeatWave);
        assert_eq!(bridge.determine_status(47.0), ClimateStatus::ExtremeHeat);
        assert_eq!(bridge.determine_status(50.0), ClimateStatus::Critical);
    }

    #[test]
    fn test_climate_data_processing() {
        let bridge = ClimateBridge::new();
        let event = bridge.process_climate_data(46.0, 15.0, 85.0);

        assert_eq!(event.temperature, 46.0);
        assert!(event.heat_index >= event.temperature);
    }
}
