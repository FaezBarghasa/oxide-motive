
use nalgebra::{SMatrix, SVector};

pub struct ClockSyncManager {
    offset: f64,
    skew: f64,
    kalman_p: SMatrix<f64, 2, 2>,
    kalman_k: SVector<f64, 2>,
}

impl ClockSyncManager {
    pub fn new() -> Self {
        Self {
            offset: 0.0,
            skew: 1.0,
            kalman_p: SMatrix::<f64, 2, 2>::identity(),
            kalman_k: SVector::<f64, 2>::zeros(),
        }
    }

    pub fn update(&mut self, t1: u64, t2: u64, t3: u64, t4: u64) {
        let t1 = t1 as f64;
        let t2 = t2 as f64;
        let t3 = t3 as f64;
        let t4 = t4 as f64;

        let delay = ((t4 - t1) - (t3 - t2)) / 2.0;
        let offset = ((t2 - t1) + (t3 - t4)) / 2.0;

        let h = SMatrix::<f64, 1, 2>::new(1.0, delay);
        let r = 1.0; // Measurement noise covariance

        let y = offset - (self.offset + self.skew * delay);
        let s = (h * self.kalman_p * h.transpose() + r)[(0, 0)];
        self.kalman_k = self.kalman_p * h.transpose() / s;
        let x_update = self.kalman_k * y;

        self.offset += x_update[0];
        self.skew += x_update[1];

        self.kalman_p = (SMatrix::<f64, 2, 2>::identity() - self.kalman_k * h) * self.kalman_p;
    }

    pub fn corrected_time(&self, local_time: u64) -> f64 {
        self.offset + self.skew * (local_time as f64)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_clock_sync_manager() {
        let mut manager = ClockSyncManager::new();
        manager.update(1000, 1100, 1200, 1300);
        assert_ne!(manager.offset, 0.0);
    }
}
