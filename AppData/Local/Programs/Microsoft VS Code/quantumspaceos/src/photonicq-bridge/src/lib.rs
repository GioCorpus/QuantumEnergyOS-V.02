use serde::{Deserialize, Serialize};
use std::sync::{Arc, RwLock};
use std::time::{Duration, Instant};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PhotonicState {
    pub wavelength: f64,
    pub amplitude: f64,
    pub phase: f64,
    pub polarization: Polarization,
    pub coherence_time: Duration,
    pub entangled_pair_id: Option<u64>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum Polarization {
    Horizontal,
    Vertical,
    CircularLeft,
    CircularRight,
    Diagonal,
    AntiDiagonal,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QuantumPhoton {
    pub id: u64,
    pub state: PhotonicState,
    pub spatial_mode: SpatialMode,
    pub creation_time: Instant,
    pub ttl: Duration,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum SpatialMode {
    SingleMode,
    MultiMode,
    OrbitalAngularMomentum(i32),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PhotonicBridgeConfig {
    pub frequency_thz: f64,
    pub bandwidth_ghz: f64,
    pub input_power_dbm: f64,
    pub output_power_dbm: f64,
    pub noise_figure_db: f64,
    pub temperature_k: f64,
    pub vacuum_mode: bool,
    pub radiation_hardened: bool,
}

impl Default for PhotonicBridgeConfig {
    fn default() -> Self {
        Self {
            frequency_thz: 193.414,
            bandwidth_ghz: 100.0,
            input_power_dbm: 0.0,
            output_power_dbm: -3.0,
            noise_figure_db: 5.0,
            temperature_k: 4.0,
            vacuum_mode: true,
            radiation_hardened: true,
        }
    }
}

pub struct PhotonicQBridge {
    config: PhotonicBridgeConfig,
    state: Arc<RwLock<BridgeState>>,
    photon_buffer: Arc<RwLock<Vec<QuantumPhoton>>>,
    entangled_pairs: Arc<RwLock<Vec<EntangledPair>>>,
    statistics: Arc<RwLock<BridgeStatistics>>,
}

#[derive(Debug)]
struct BridgeState {
    operational: bool,
    photon_count: u64,
    last_transmission: Option<Instant>,
    error_count: u32,
    temperature: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EntangledPair {
    pub id: u64,
    pub photon_a: u64,
    pub photon_b: u64,
    pub bell_state: BellState,
    pub created_at: Instant,
    pub fidelity: f64,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum BellState {
    PhiPlus,
    PhiMinus,
    PsiPlus,
    PsiMinus,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BridgeStatistics {
    pub photons_transmitted: u64,
    pub photons_received: u64,
    pub entanglement_events: u64,
    pub errors_corrected: u64,
    pub average_latency_ns: u64,
    pub uptime_percentage: f64,
}

impl Default for BridgeStatistics {
    fn default() -> Self {
        Self {
            photons_transmitted: 0,
            photons_received: 0,
            entanglement_events: 0,
            errors_corrected: 0,
            average_latency_ns: 0,
            uptime_percentage: 100.0,
        }
    }
}

impl PhotonicQBridge {
    pub fn new(config: PhotonicBridgeConfig) -> Result<Self, BridgeError> {
        if config.frequency_thz < 100.0 || config.frequency_thz > 1000.0 {
            return Err(BridgeError::InvalidFrequency(config.frequency_thz));
        }
        if config.temperature_k < 0.0 || config.temperature_k > 300.0 {
            return Err(BridgeError::InvalidTemperature(config.temperature_k));
        }

        Ok(Self {
            config,
            state: Arc::new(RwLock::new(BridgeState {
                operational: true,
                photon_count: 0,
                last_transmission: None,
                error_count: 0,
                temperature: 4.0,
            })),
            photon_buffer: Arc::new(RwLock::new(Vec::new())),
            entangled_pairs: Arc::new(RwLock::new(Vec::new())),
            statistics: Arc::new(RwLock::new(BridgeStatistics::default())),
        })
    }

    pub fn new_default() -> Result<Self, BridgeError> {
        Self::new(PhotonicBridgeConfig::default())
    }

    pub fn emit_photon(&self, state: PhotonicState) -> Result<QuantumPhoton, BridgeError> {
        let state_guard = self.state.read().map_err(|_| BridgeError::LockError)?;
        if !state_guard.operational {
            return Err(BridgeError::BridgeNotOperational);
        }
        drop(state_guard);

        let photon = QuantumPhoton {
            id: self.generate_photon_id(),
            state,
            spatial_mode: SpatialMode::SingleMode,
            creation_time: Instant::now(),
            ttl: Duration::from_secs(300),
        };

        {
            let mut buffer = self
                .photon_buffer
                .write()
                .map_err(|_| BridgeError::LockError)?;
            buffer.push(photon.clone());
        }

        {
            let mut stats = self
                .statistics
                .write()
                .map_err(|_| BridgeError::LockError)?;
            stats.photons_transmitted += 1;
        }

        {
            let mut state = self.state.write().map_err(|_| BridgeError::LockError)?;
            state.photon_count += 1;
            state.last_transmission = Some(Instant::now());
        }

        Ok(photon)
    }

    pub fn receive_photon(
        &self,
        photon: &QuantumPhoton,
    ) -> Result<PhotonReceptionResult, BridgeError> {
        let now = Instant::now();
        if now.duration_since(photon.creation_time) > photon.ttl {
            return Err(BridgeError::PhotonExpired);
        }

        let latency_ns = now.duration_since(photon.creation_time).as_nanos() as u64;

        {
            let mut stats = self
                .statistics
                .write()
                .map_err(|_| BridgeError::LockError)?;
            stats.photons_received += 1;
            stats.average_latency_ns = (stats.average_latency_ns + latency_ns) / 2;
        }

        Ok(PhotonReceptionResult {
            photon_id: photon.id,
            received: true,
            latency_ns,
            fidelity: self.calculate_fidelity(&photon.state),
        })
    }

    pub fn create_entanglement(
        &self,
        photon_a: QuantumPhoton,
        photon_b: QuantumPhoton,
        bell_state: BellState,
    ) -> Result<EntangledPair, BridgeError> {
        let pair = EntangledPair {
            id: self.generate_entanglement_id(),
            photon_a: photon_a.id,
            photon_b: photon_b.id,
            bell_state,
            created_at: Instant::now(),
            fidelity: 0.98,
        };

        {
            let mut pairs = self
                .entangled_pairs
                .write()
                .map_err(|_| BridgeError::LockError)?;
            pairs.push(pair.clone());
        }

        {
            let mut stats = self
                .statistics
                .write()
                .map_err(|_| BridgeError::LockError)?;
            stats.entanglement_events += 1;
        }

        Ok(pair)
    }

    pub fn measure_entangled(&self, pair_id: u64) -> Result<EntanglementMeasurement, BridgeError> {
        let pairs = self
            .entangled_pairs
            .read()
            .map_err(|_| BridgeError::LockError)?;
        let pair = pairs
            .iter()
            .find(|p| p.id == pair_id)
            .ok_or(BridgeError::EntanglementNotFound)?;

        let measurement_a = self.generate_quantum_measurement();
        let measurement_b = self.generate_quantum_measurement();

        Ok(EntanglementMeasurement {
            pair_id,
            result_a: measurement_a,
            result_b: measurement_b,
            correlation: self.calculate_correlation(&measurement_a, &measurement_b),
            timestamp: Instant::now(),
        })
    }

    pub fn transmit_data(&self, data: &[u8]) -> Result<TransmissionResult, BridgeError> {
        let photon_count = (data.len() as f64 / 1000.0).ceil() as u32;
        let base_latency_us = 100;
        let total_latency_us = base_latency_us * photon_count as u64;

        {
            let mut stats = self
                .statistics
                .write()
                .map_err(|_| BridgeError::LockError)?;
            stats.photons_transmitted += photon_count as u64;
        }

        Ok(TransmissionResult {
            bytes_transmitted: data.len() as u64,
            photons_used: photon_count,
            latency_us: total_latency_us,
            success: true,
        })
    }

    pub fn get_statistics(&self) -> Result<BridgeStatistics, BridgeError> {
        let stats = self.statistics.read().map_err(|_| BridgeError::LockError)?;
        Ok(stats.clone())
    }

    pub fn get_status(&self) -> Result<BridgeStatus, BridgeError> {
        let state = self.state.read().map_err(|_| BridgeError::LockError)?;
        let stats = self.statistics.read().map_err(|_| BridgeError::LockError)?;

        Ok(BridgeStatus {
            operational: state.operational,
            photon_count: state.photon_count,
            temperature: state.temperature,
            uptime_percentage: stats.uptime_percentage,
            error_count: state.error_count,
        })
    }

    pub fn set_operational(&self, operational: bool) -> Result<(), BridgeError> {
        let mut state = self.state.write().map_err(|_| BridgeError::LockError)?;
        state.operational = operational;
        Ok(())
    }

    fn generate_photon_id(&self) -> u64 {
        let state = self.state.read().unwrap();
        state.photon_count + 1
    }

    fn generate_entanglement_id(&self) -> u64 {
        let pairs = self.entangled_pairs.read().unwrap();
        (pairs.len() as u64) + 1
    }

    fn calculate_fidelity(&self, state: &PhotonicState) -> f64 {
        let coherence_factor = (state.coherence_time.as_nanos() as f64 / 1e9).min(1.0);
        let phase_stability = (state.phase.sin() + 1.0) / 2.0;
        coherence_factor * 0.7 + phase_stability * 0.3
    }

    fn generate_quantum_measurement(&self) -> u8 {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};
        let mut hasher = DefaultHasher::new();
        Instant::now().hash(&mut hasher);
        (hasher.finish() % 256) as u8
    }

    fn calculate_correlation(&self, a: &u8, b: &u8) -> f64 {
        if a == b {
            1.0
        } else {
            0.0
        }
    }

    pub fn configure_for_deep_space(&mut self) {
        self.config.frequency_thz = 230.0;
        self.config.bandwidth_ghz = 500.0;
        self.config.noise_figure_db = 3.0;
        self.config.radiation_hardened = true;
    }

    pub fn configure_for_leo(&mut self) {
        self.config.frequency_thz = 193.414;
        self.config.bandwidth_ghz = 100.0;
        self.config.noise_figure_db = 5.0;
        self.config.radiation_hardened = false;
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PhotonReceptionResult {
    pub photon_id: u64,
    pub received: bool,
    pub latency_ns: u64,
    pub fidelity: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransmissionResult {
    pub bytes_transmitted: u64,
    pub photons_used: u32,
    pub latency_us: u64,
    pub success: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BridgeStatus {
    pub operational: bool,
    pub photon_count: u64,
    pub temperature: f64,
    pub uptime_percentage: f64,
    pub error_count: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EntanglementMeasurement {
    pub pair_id: u64,
    pub result_a: u8,
    pub result_b: u8,
    pub correlation: f64,
    pub timestamp: Instant,
}

#[derive(Debug)]
pub enum BridgeError {
    InvalidFrequency(f64),
    InvalidTemperature(f64),
    BridgeNotOperational,
    PhotonExpired,
    LockError,
    EntanglementNotFound,
    TransmissionFailed,
}

impl std::fmt::Display for BridgeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            BridgeError::InvalidFrequency(freq) => write!(f, "Invalid frequency: {} THz", freq),
            BridgeError::InvalidTemperature(temp) => write!(f, "Invalid temperature: {} K", temp),
            BridgeError::BridgeNotOperational => write!(f, "Bridge is not operational"),
            BridgeError::PhotonExpired => write!(f, "Photon TTL expired"),
            BridgeError::LockError => write!(f, "Failed to acquire lock"),
            BridgeError::EntanglementNotFound => write!(f, "Entangled pair not found"),
            BridgeError::TransmissionFailed => write!(f, "Data transmission failed"),
        }
    }
}

impl std::error::Error for BridgeError {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bridge_creation() {
        let bridge = PhotonicQBridge::new_default();
        assert!(bridge.is_ok());
    }

    #[test]
    fn test_photon_emission() {
        let bridge = PhotonicQBridge::new_default().unwrap();
        let state = PhotonicState {
            wavelength: 1550.0,
            amplitude: 1.0,
            phase: 0.0,
            polarization: Polarization::Horizontal,
            coherence_time: Duration::from_nanos(1000),
            entangled_pair_id: None,
        };
        let photon = bridge.emit_photon(state);
        assert!(photon.is_ok());
    }

    #[test]
    fn test_entanglement_creation() {
        let bridge = PhotonicQBridge::new_default().unwrap();
        let state = PhotonicState {
            wavelength: 1550.0,
            amplitude: 1.0,
            phase: 0.0,
            polarization: Polarization::CircularLeft,
            coherence_time: Duration::from_nanos(1000),
            entangled_pair_id: None,
        };
        let photon_a = bridge.emit_photon(state.clone()).unwrap();
        let photon_b = bridge.emit_photon(state).unwrap();
        let entanglement = bridge.create_entanglement(photon_a, photon_b, BellState::PhiPlus);
        assert!(entanglement.is_ok());
    }

    #[test]
    fn test_deep_space_configuration() {
        let bridge = PhotonicQBridge::new_default().unwrap();
        bridge.configure_for_deep_space();
        let status = bridge.get_status().unwrap();
        assert!(status.operational);
    }
}
