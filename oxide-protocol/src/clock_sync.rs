//! PTP-like Clock Synchronization with Kalman Filter
#![no_std]

pub struct ClockSyncManager {
    offset_ns: f64,
    skew_ppm: f64,
    // Kalman Filter state
    kalman_p: f64, // Estimate error covariance
    kalman_q: f64, // Process noise covariance
    kalman_r: f64, // Measurement noise covariance
}

impl ClockSyncManager {
    pub fn new() -> Self {
        Self {
            offset_ns: 0.0,
            skew_ppm: 0.0,
            kalman_p: 1.0,
            kalman_q: 1e-9, // Low process noise (clocks are stable)
            kalman_r: 1e-5, // Measurement noise (network jitter)
        }
    }

    /// Updates the clock offset using a 4-way PTP handshake.
    /// t1: Host send time
    /// t2: MCU receive time
    /// t3: MCU send time
    /// t4: Host receive time
    pub fn update(&mut self, t1: f64, t2: f64, t3: f64, t4: f64) {
        // Network delay (assumed symmetric)
        let delay = ((t4 - t1) - (t3 - t2)) * 0.5;

        // Raw clock offset measurement
        let raw_offset = ((t2 - t1) + (t3 - t4)) * 0.5;

        // Kalman Filter Update Step
        // 1. Predict (skipped here as we assume constant offset between fast updates)
        // 2. Calculate Kalman Gain
        let kalman_gain = self.kalman_p / (self.kalman_p + self.kalman_r);

        // 3. Update Estimate
        self.offset_ns += kalman_gain * (raw_offset - self.offset_ns);

        // 4. Update Covariance
        self.kalman_p = (1.0 - kalman_gain) * self.kalman_p + self.kalman_q.abs();
    }

    /// Converts a local MCU timestamp to the globally synchronized Host time.
    pub fn corrected_time_ns(&self, local_time_ns: f64) -> f64 {
        local_time_ns + self.offset_ns
    }
}