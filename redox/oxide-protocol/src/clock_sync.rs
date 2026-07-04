pub struct ClockSyncManager {
    alpha: f32,
    filtered_offset: f32,
}

impl ClockSyncManager {
    pub fn new(alpha: f32) -> Self {
        Self {
            alpha,
            filtered_offset: 0.0,
        }
    }

    pub fn calculate_offset(&mut self, t1: u64, t2: u64, t3: u64, t4: u64) -> f32 {
        let delay = (t4 - t1) - (t3 - t2);
        let offset = ((t2 as i64 - t1 as i64) + (t3 as i64 - t4 as i64)) / 2;

        self.filtered_offset = (self.alpha * offset as f32) + ((1.0 - self.alpha) * self.filtered_offset);
        self.filtered_offset
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_clock_sync() {
        let mut manager = ClockSyncManager::new(0.1);
        let true_offset = 1000;
        let mut t1 = 10000;

        for i in 0..20 {
            let delay1 = 100 + i * 10; // Varying delay
            let delay2 = 120 - i * 5;

            let t2 = t1 + delay1 + true_offset;
            let t3 = t2 + 500;
            let t4 = t3 + delay2 - true_offset;

            let offset = manager.calculate_offset(t1, t2, t3, t4);
            t1 += 2000;

            if i > 10 {
                assert!((offset - true_offset as f32).abs() < 50.0);
            }
        }
    }
}
