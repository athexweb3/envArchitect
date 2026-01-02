use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BuildMetrics {
    pub ema_duration: f64,
    pub mad_duration: f64,
}

impl BuildMetrics {
    pub fn new() -> Self {
        Self {
            ema_duration: 0.0,
            mad_duration: 0.0,
        }
    }

    /// Update metrics with a new duration sample using Exponential Moving Average
    pub fn update(&mut self, duration: f64) {
        if self.ema_duration == 0.0 {
            self.ema_duration = duration;
            return;
        }

        let alpha = 0.2; // Smoothing factor

        // Update EMA
        self.ema_duration = alpha * duration + (1.0 - alpha) * self.ema_duration;

        // Update Mean Absolute Deviation (Anomaly Detection)
        let deviation = (duration - self.ema_duration).abs();
        self.mad_duration = alpha * deviation + (1.0 - alpha) * self.mad_duration;
    }

    /// Check if the current duration is anomalous (e.g. > 3 * MAD)
    pub fn is_anomaly(&self, duration: f64) -> bool {
        if self.mad_duration == 0.0 {
            return false;
        }
        let threshold = self.ema_duration + (3.0 * self.mad_duration);
        duration > threshold
    }
}
