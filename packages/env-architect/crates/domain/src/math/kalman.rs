use serde::{Deserialize, Serialize};

/// A robust estimator for network progress speed and ETA using a 1D Kalman Filter.
/// It fuses the "Prediction" (Speed stays constant) with "Measurement" (Bytes / Time).
///
/// It also uses Welford's Algorithm to track the standard deviation (jitter) of the connection.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProgressEstimator {
    // Kalman State
    current_speed_estimate: f64, // The state (bytes per second)
    estimate_uncertainty: f64,   // P (covariance/uncertainty of estimation)

    // Kalman Tuning
    process_noise: f64,     // Q (how much we expect speed to change naturally)
    measurement_noise: f64, // R (how jittery the network measurements are)

    // Welford State for Jitter (Standard Deviation)
    n_samples: u64,
    m_old: f64, // Mean
    s_old: f64, // Sum of squares of differences
}

impl Default for ProgressEstimator {
    fn default() -> Self {
        Self {
            // Initial state
            current_speed_estimate: 0.0,
            estimate_uncertainty: 1000.0, // High uncertainty initially

            // Tuning parameters (tuned for typical internet connection profiles)
            process_noise: 100.0, // We expect moderate speed changes (Process Noise Q)
            measurement_noise: 5000.0, // Measurements can be very jittery (Measurement Noise R)

            // Welford
            n_samples: 0,
            m_old: 0.0,
            s_old: 0.0,
        }
    }
}

impl ProgressEstimator {
    pub fn new() -> Self {
        Self::default()
    }

    /// Update the estimator with a new measurement (bytes received since last start / time delta).
    /// `instant_speed` should be bytes/sec calculated from the last chunk.
    pub fn update(&mut self, instant_speed: f64) {
        // 1. Welford's Algorithm (Track Jitter/Variance)
        self.n_samples += 1;

        let x = instant_speed;
        let m_new = if self.n_samples == 1 {
            x
        } else {
            self.m_old + (x - self.m_old) / self.n_samples as f64
        };

        // s_new = s_old + (x - m_old) * (x - m_new)
        let s_new = if self.n_samples > 1 {
            self.s_old + (x - self.m_old) * (x - m_new)
        } else {
            0.0
        };

        self.m_old = m_new;
        self.s_old = s_new;

        // Dynamic Measurement Noise (R) based on observed Jitter
        // If variance is high, trust measurements LESS.
        if self.n_samples > 2 {
            let variance = self.s_old / (self.n_samples - 1) as f64;
            // R grows with variance
            self.measurement_noise = 5000.0 + variance.sqrt();
        }

        // 2. Kalman Filter Step

        // Prediction Step (Time Update)
        // Physics Model: Speed stays roughly constant (Identity transition)
        // x_pred = x_prev
        let x_pred = self.current_speed_estimate;
        // P_pred = P_prev + Q
        let p_pred = self.estimate_uncertainty + self.process_noise;

        // Measurement Update (Correction Step)
        // Kalman Gain K = P_pred / (P_pred + R)
        let k = p_pred / (p_pred + self.measurement_noise);

        // x_new = x_pred + K * (measurement - x_pred)
        self.current_speed_estimate = x_pred + k * (instant_speed - x_pred);

        // P_new = (1 - K) * P_pred
        self.estimate_uncertainty = (1.0 - k) * p_pred;
    }

    /// Returns the estimated "Smooth Speed" in bytes/sec.
    pub fn speed(&self) -> f64 {
        self.current_speed_estimate
    }

    /// Returns the standard deviation (jitter) of the connection.
    pub fn jitter(&self) -> f64 {
        if self.n_samples < 2 {
            return 0.0;
        }
        (self.s_old / (self.n_samples - 1) as f64).sqrt()
    }

    /// Returns ETA in seconds given remaining bytes.
    /// Returns None if speed is too low or uncertain.
    pub fn eta(&self, bytes_remaining: u64) -> Option<f64> {
        let speed = self.speed();
        if speed < 1.0 {
            return None; // Stalled
        }

        Some(bytes_remaining as f64 / speed)
    }
}
