use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};
use std::f64::consts::{E, PI};

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum OrbitType {
    LEO,
    MEO,
    GEO,
    HEO,
    GTO,
    MarsTransfer,
    LunarTransfer,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum MissionType {
    EarthOrbit,
    MarsInsertion,
    LunarOrbit,
    AtmosphericEntry,
    Descent,
    Landing,
    Flyby,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrbitalElements {
    pub semi_major_axis_km: f64,
    pub eccentricity: f64,
    pub inclination_deg: f64,
    pub right_ascension_deg: f64,
    pub argument_of_perigee_deg: f64,
    pub true_anomaly_deg: f64,
    pub epoch: DateTime<Utc>,
}

impl OrbitalElements {
    pub fn new_leo(altitude_km: f64) -> Self {
        let earth_radius_km = 6371.0;
        Self {
            semi_major_axis_km: earth_radius_km + altitude_km,
            eccentricity: 0.001,
            inclination_deg: 28.5,
            right_ascension_deg: 0.0,
            argument_of_perigee_deg: 0.0,
            true_anomaly_deg: 0.0,
            epoch: Utc::now(),
        }
    }

    pub fn new_geo() -> Self {
        let earth_radius_km = 6371.0;
        let geo_altitude_km = 35786.0;
        Self {
            semi_major_axis_km: earth_radius_km + geo_altitude_km,
            eccentricity: 0.0,
            inclination_deg: 0.0,
            right_ascension_deg: 0.0,
            argument_of_perigee_deg: 0.0,
            true_anomaly_deg: 0.0,
            epoch: Utc::now(),
        }
    }

    pub fn new_mars_transfer() -> Self {
        Self {
            semi_major_axis_km: 225_000_000.0,
            eccentricity: 0.2,
            inclination_deg: 1.85,
            right_ascension_deg: 0.0,
            argument_of_perigee_deg: 0.0,
            true_anomaly_deg: 0.0,
            epoch: Utc::now(),
        }
    }

    pub fn new_lunar() -> Self {
        let earth_radius_km = 6371.0;
        let moon_distance_km = 384400.0;
        Self {
            semi_major_axis_km: earth_radius_km + moon_distance_km,
            eccentricity: 0.0549,
            inclination_deg: 5.145,
            right_ascension_deg: 0.0,
            argument_of_perigee_deg: 0.0,
            true_anomaly_deg: 0.0,
            epoch: Utc::now(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StateVector {
    pub position_km: [f64; 3],
    pub velocity_km_s: [f64; 3],
    pub epoch: DateTime<Utc>,
}

impl StateVector {
    pub fn from_orbital_elements(elements: &OrbitalElements) -> Self {
        let a = elements.semi_major_axis_km;
        let e = elements.eccentricity;
        let i = elements.inclination_deg.to_radians();
        let omega = elements.right_ascension_deg.to_radians();
        let w = elements.argument_of_perigee_deg.to_radians();
        let nu = elements.true_anomaly_deg.to_radians();

        let r = a * (1.0 - e * e) / (1.0 + e * nu.cos());

        let x_orb = r * nu.cos();
        let y_orb = r * nu.sin();

        let cos_omega = omega.cos();
        let sin_omega = omega.sin();
        let cos_i = i.cos();
        let sin_i = i.sin();
        let cos_w = w.cos();
        let sin_w = w.sin();

        let x = x_orb * (cos_omega * cos_w - sin_omega * sin_w * cos_i)
            - y_orb * (cos_omega * sin_w + sin_omega * cos_w * cos_i);
        let y = x_orb * (sin_omega * cos_w + cos_omega * sin_w * cos_i)
            - y_orb * (sin_omega * sin_w - cos_omega * cos_w * cos_i);
        let z = x_orb * (sin_w * sin_i) + y_orb * (cos_w * sin_i);

        let mu = 398600.4418;
        let h = (mu * a * (1.0 - e * e)).sqrt();
        let vx = -mu / h
            * (-sin_omega * (e * nu.cos() + nu.cos()) + cos_omega * (1.0 + e * nu.cos()) * cos_i);
        let vy = -mu / h
            * (cos_omega * (e * nu.cos() + nu.cos()) + sin_omega * (1.0 + e * nu.cos()) * cos_i);
        let vz = -mu / h * (1.0 + e * nu.cos()) * sin_i;

        Self {
            position_km: [x, y, z],
            velocity_km_s: [vx, vy, vz],
            epoch: elements.epoch,
        }
    }

    pub fn altitude_km(&self) -> f64 {
        let r = (self.position_km[0].powi(2)
            + self.position_km[1].powi(2)
            + self.position_km[2].powi(2))
        .sqrt();
        r - 6371.0
    }

    pub fn speed_km_s(&self) -> f64 {
        (self.velocity_km_s[0].powi(2)
            + self.velocity_km_s[1].powi(2)
            + self.velocity_km_s[2].powi(2))
        .sqrt()
    }
}

pub struct OrbitalMechanics {
    mu_km3_s2: f64,
    earth_radius_km: f64,
}

impl OrbitalMechanics {
    pub fn new() -> Self {
        Self {
            mu_km3_s2: 398600.4418,
            earth_radius_km: 6371.0,
        }
    }

    pub fn new_mars() -> Self {
        Self {
            mu_km3_s2: 42828.3,
            earth_radius_km: 3389.5,
        }
    }

    pub fn period_seconds(&self, semi_major_axis_km: f64) -> f64 {
        2.0 * PI * (semi_major_axis_km.powi(3) / self.mu_km3_s2).sqrt()
    }

    pub fn period_minutes(&self, semi_major_axis_km: f64) -> f64 {
        self.period_seconds(semi_major_axis_km) / 60.0
    }

    pub fn velocity_circular(&self, altitude_km: f64) -> f64 {
        let r = self.earth_radius_km + altitude_km;
        (self.mu_km3_s2 / r).sqrt()
    }

    pub fn velocity_escape(&self, altitude_km: f64) -> f64 {
        let r = self.earth_radius_km + altitude_km;
        (2.0 * self.mu_km3_s2 / r).sqrt()
    }

    pub fn delta_v_to_transfer(&self, r1_km: f64, r2_km: f64) -> f64 {
        let v1 = (self.mu_km3_s2 / r1_km).sqrt();
        let v2 = (self.mu_km3_s2 / r2_km).sqrt();
        let a_trans = (r1_km + r2_km) / 2.0;
        let v_trans = (self.mu_km3_s2 * (2.0 / r1_km - 1.0 / a_trans)).sqrt();
        (v_trans - v1).abs() + (v2 - (self.mu_km3_s2 * (2.0 / r2_km - 1.0 / a_trans)).sqrt()).abs()
    }

    pub fn calculate_keplerian_propagation(
        &self,
        elements: &OrbitalElements,
        dt_seconds: f64,
    ) -> OrbitalElements {
        let n = (self.mu_km3_s2 / elements.semi_major_axis_km.powi(3)).sqrt();
        let m0 = elements.true_anomaly_deg.to_radians();
        let m = (m0 + n * dt_seconds) % (2.0 * PI);

        let mut new_elements = elements.clone();
        new_elements.true_anomaly_deg = m.to_degrees() % 360.0;
        new_elements.epoch = elements.epoch + Duration::seconds(dt_seconds as i64);
        new_elements
    }

    pub fn propagate_to_time(
        &self,
        state: &StateVector,
        target_time: DateTime<Utc>,
    ) -> StateVector {
        let dt_seconds = (target_time - state.epoch).num_seconds() as f64;
        let mut new_state = state.clone();

        let r = (state.position_km[0].powi(2)
            + state.position_km[1].powi(2)
            + state.position_km[2].powi(2))
        .sqrt();

        let a = -self.mu_km3_s2 / (2.0 * (state.speed_km_s().powi(2) - 2.0 * self.mu_km3_s2 / r));
        let period = self.period_seconds(a);
        let mean_motion = 2.0 * PI / period;

        let angle = mean_motion * dt_seconds;
        let cos_a = state.position_km[0] / r;
        let sin_a = state.position_km[1] / r;

        new_state.position_km[0] = r * (cos_a * angle.cos() - sin_a * angle.sin());
        new_state.position_km[1] = r * (sin_a * angle.cos() + cos_a * angle.sin());
        new_state.epoch = target_time;

        new_state
    }

    pub fn calculate_eccentric_anomaly(&self, m: f64, e: f64) -> f64 {
        let mut e_anom = m;
        for _ in 0..10 {
            e_anom = m + e * e_anom.sin();
        }
        e_anom
    }

    pub fn calculate_true_anomaly(&self, e_anom: f64, e: f64) -> f64 {
        2.0 * ((1.0 + e).sqrt() * (e_anom / 2.0).tan().atan())
            .tan()
            .asin()
    }

    pub fn hill_frame(&self, state: &StateVector) -> [[f64; 3]; 3] {
        let r = [
            state.position_km[0],
            state.position_km[1],
            state.position_km[2],
        ];
        let v = [
            state.velocity_km_s[0],
            state.velocity_km_s[1],
            state.velocity_km_s[2],
        ];

        let r_mag = (r[0].powi(2) + r[1].powi(2) + r[2].powi(2)).sqrt();
        let h = [
            r[1] * v[2] - r[2] * v[1],
            r[2] * v[0] - r[0] * v[2],
            r[0] * v[1] - r[1] * v[0],
        ];
        let h_mag = (h[0].powi(2) + h[1].powi(2) + h[2].powi(2)).sqrt();

        let i_hat = [r[0] / r_mag, r[1] / r_mag, r[2] / r_mag];
        let k_hat = [h[0] / h_mag, h[1] / h_mag, h[2] / h_mag];
        let j_hat = [
            k_hat[1] * i_hat[2] - k_hat[2] * i_hat[1],
            k_hat[2] * i_hat[0] - k_hat[0] * i_hat[2],
            k_hat[0] * i_hat[1] - k_hat[1] * i_hat[0],
        ];

        [
            [i_hat[0], j_hat[0], k_hat[0]],
            [i_hat[1], j_hat[1], k_hat[1]],
            [i_hat[2], j_hat[2], k_hat[2]],
        ]
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThrustVector {
    pub magnitude_n: f64,
    pub direction_deg: [f64; 3],
    pub throttle: f64,
}

impl ThrustVector {
    pub fn new(magnitude_n: f64, pitch_deg: f64, yaw_deg: f64, roll_deg: f64) -> Self {
        Self {
            magnitude_n,
            direction_deg: [pitch_deg, yaw_deg, roll_deg],
            throttle: 1.0,
        }
    }

    pub fn from_delta_v(delta_v_ms: f64, mass_kg: f64, burn_time_s: f64) -> Self {
        let thrust = mass_kg * delta_v_ms / burn_time_s;
        Self::new(thrust, 0.0, 0.0, 0.0)
    }
}

pub struct FlightSimulation {
    orbital_mechanics: OrbitalMechanics,
    current_state: StateVector,
    mass_kg: f64,
    fuel_kg: f64,
    total_fuel_kg: f64,
    mission_type: MissionType,
    time: DateTime<Utc>,
}

impl FlightSimulation {
    pub fn new(mission_type: MissionType, initial_mass_kg: f64, fuel_kg: f64) -> Self {
        let elements = match mission_type {
            MissionType::EarthOrbit => OrbitalElements::new_leo(400.0),
            MissionType::MarsInsertion => OrbitalElements::new_mars_transfer(),
            MissionType::LunarOrbit => OrbitalElements::new_lunar(),
            _ => OrbitalElements::new_leo(400.0),
        };

        Self {
            orbital_mechanics: OrbitalMechanics::new(),
            current_state: StateVector::from_orbital_elements(&elements),
            mass_kg: initial_mass_kg,
            fuel_kg,
            total_fuel_kg: fuel_kg,
            mission_type,
            time: Utc::now(),
        }
    }

    pub fn new_mars(mission_type: MissionType, initial_mass_kg: f64, fuel_kg: f64) -> Self {
        let mut sim = Self::new(mission_type, initial_mass_kg, fuel_kg);
        sim.orbital_mechanics = OrbitalMechanics::new_mars();
        sim
    }

    pub fn propagate(&mut self, dt_seconds: f64) {
        let mut new_state = self.current_state.clone();

        let r = (self.current_state.position_km[0].powi(2)
            + self.current_state.position_km[1].powi(2)
            + self.current_state.position_km[2].powi(2))
        .sqrt();

        let a_mag = self.orbital_mechanics.mu_km3_s2 / r.powi(2);
        let a = [
            -a_mag * self.current_state.position_km[0] / r,
            -a_mag * self.current_state.position_km[1] / r,
            -a_mag * self.current_state.position_km[2] / r,
        ];

        new_state.velocity_km_s[0] += a[0] * dt_seconds;
        new_state.velocity_km_s[1] += a[1] * dt_seconds;
        new_state.velocity_km_s[2] += a[2] * dt_seconds;

        new_state.position_km[0] += new_state.velocity_km_s[0] * dt_seconds;
        new_state.position_km[1] += new_state.velocity_km_s[1] * dt_seconds;
        new_state.position_km[2] += new_state.velocity_km_s[2] * dt_seconds;

        new_state.epoch = self.time + Duration::seconds(dt_seconds as i64);
        self.current_state = new_state;
        self.time = self.current_state.epoch;
    }

    pub fn apply_thrust(
        &mut self,
        thrust: &ThrustVector,
        dt_seconds: f64,
    ) -> Result<(), SimulationError> {
        if self.fuel_kg <= 0.0 {
            return Err(SimulationError::OutOfFuel);
        }

        let fuel_consumed = thrust.magnitude_n * dt_seconds * 1e-6;
        if fuel_consumed > self.fuel_kg {
            return Err(SimulationError::InsufficientFuel);
        }

        let acc = thrust.magnitude_n * thrust.throttle / self.mass_kg / 1000.0;

        self.current_state.velocity_km_s[0] += acc * dt_seconds;
        self.current_state.velocity_km_s[1] += acc * dt_seconds;
        self.current_state.velocity_km_s[2] += acc * dt_seconds;

        self.fuel_kg -= fuel_consumed;
        self.mass_kg -= fuel_consumed;

        Ok(())
    }

    pub fn execute_maneuver(
        &mut self,
        delta_v_ms: f64,
        direction: [f64; 3],
    ) -> Result<f64, SimulationError> {
        let required_fuel = (delta_v_ms * self.mass_kg / 3000.0).abs();

        if required_fuel > self.fuel_kg {
            return Err(SimulationError::InsufficientFuel);
        }

        let burn_time_s = 100.0;
        let thrust = ThrustVector::from_delta_v(delta_v_ms, self.mass_kg, burn_time_s);

        self.apply_thrust(&thrust, burn_time_s)?;

        self.fuel_kg -= required_fuel;
        self.mass_kg -= required_fuel;

        Ok(required_fuel)
    }

    pub fn calculate_mars_insertion(&self) -> ManeuverResult {
        let current_speed = self.current_state.speed_km_s();
        let mars_approach_speed = 5.0;
        let delta_v = (current_speed - mars_approach_speed).abs();

        let required_fuel = delta_v * self.mass_kg / 3000.0;

        ManeuverResult {
            delta_v_ms: delta_v * 1000.0,
            fuel_required_kg: required_fuel,
            maneuver_time_seconds: 1200.0,
            success: true,
        }
    }

    pub fn calculate_reentry(&self, target_altitude_km: f64) -> ReentryResult {
        let current_alt = self.current_state.altitude_km();
        let velocity = self.current_state.speed_km_s();

        let entry_angle = ((velocity.powi(2)
            / (self.orbital_mechanics.earth_radius_km + current_alt))
            - self.orbital_mechanics.mu_km3_s2
                / (self.orbital_mechanics.earth_radius_km + current_alt).powi(2))
        .asin();

        let heat_rate = 1.2e-4 * velocity.powi(3)
            / (self.orbital_mechanics.earth_radius_km + current_alt).sqrt();

        ReentryResult {
            entry_angle_deg: entry_angle.to_degrees(),
            peak_heat_rate_kw_m2: heat_rate,
            deceleration_g: 0.5 * velocity.powi(2)
                / (self.orbital_mechanics.earth_radius_km + target_altitude_km)
                / 9.81,
            target_altitude_km,
            landing_velocity_ms: 0.0,
            success: true,
        }
    }

    pub fn get_state(&self) -> &StateVector {
        &self.current_state
    }

    pub fn get_fuel_remaining(&self) -> f64 {
        self.fuel_kg
    }

    pub fn get_mass(&self) -> f64 {
        self.mass_kg
    }

    pub fn get_orbit_period(&self) -> f64 {
        let a = (self.current_state.position_km[0].powi(2)
            + self.current_state.position_km[1].powi(2)
            + self.current_state.position_km[2].powi(2))
        .sqrt();
        self.orbital_mechanics.period_seconds(a)
    }

    pub fn simulate_orbit(&mut self, num_orbits: u32) -> Vec<OrbitDataPoint> {
        let mut data = Vec::new();
        let period = self.get_orbit_period();
        let steps_per_orbit = 360;
        let dt = period / steps_per_orbit as f64;

        for _ in 0..(num_orbits * steps_per_orbit) {
            self.propagate(dt);
            data.push(OrbitDataPoint {
                time: self.time,
                position: self.current_state.position_km,
                velocity: self.current_state.velocity_km_s,
                altitude_km: self.current_state.altitude_km(),
                speed_km_s: self.current_state.speed_km_s(),
            });
        }

        data
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrbitDataPoint {
    pub time: DateTime<Utc>,
    pub position: [f64; 3],
    pub velocity: [f64; 3],
    pub altitude_km: f64,
    pub speed_km_s: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ManeuverResult {
    pub delta_v_ms: f64,
    pub fuel_required_kg: f64,
    pub maneuver_time_seconds: f64,
    pub success: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReentryResult {
    pub entry_angle_deg: f64,
    pub peak_heat_rate_kw_m2: f64,
    pub deceleration_g: f64,
    pub target_altitude_km: f64,
    pub landing_velocity_ms: f64,
    pub success: bool,
}

#[derive(Debug)]
pub enum SimulationError {
    OutOfFuel,
    InsufficientFuel,
    InvalidState,
    PropagationFailed,
}

impl std::fmt::Display for SimulationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SimulationError::OutOfFuel => write!(f, "Out of fuel"),
            SimulationError::InsufficientFuel => write!(f, "Insufficient fuel for maneuver"),
            SimulationError::InvalidState => write!(f, "Invalid orbital state"),
            SimulationError::PropagationFailed => write!(f, "Orbital propagation failed"),
        }
    }
}

impl std::error::Error for SimulationError {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_orbital_elements_leo() {
        let elements = OrbitalElements::new_leo(400.0);
        assert!((elements.semi_major_axis_km - 6771.0).abs() < 1.0);
    }

    #[test]
    fn test_state_vector_from_elements() {
        let elements = OrbitalElements::new_leo(400.0);
        let state = StateVector::from_orbital_elements(&elements);
        let altitude = state.altitude_km();
        assert!(altitude > 350.0 && altitude < 450.0);
    }

    #[test]
    fn test_orbital_mechanics_velocity() {
        let om = OrbitalMechanics::new();
        let v = om.velocity_circular(400.0);
        assert!(v > 7.5 && v < 8.0);
    }

    #[test]
    fn test_flight_simulation_propagation() {
        let mut sim = FlightSimulation::new(MissionType::EarthOrbit, 1000.0, 500.0);
        let initial_alt = sim.get_state().altitude_km();
        sim.propagate(60.0);
        let new_alt = sim.get_state().altitude_km();
        assert!((initial_alt - new_alt).abs() < 10.0);
    }

    #[test]
    fn test_mars_insertion_maneuver() {
        let mut sim = FlightSimulation::new(MissionType::MarsInsertion, 5000.0, 2000.0);
        let result = sim.calculate_mars_insertion();
        assert!(result.success);
    }
}
