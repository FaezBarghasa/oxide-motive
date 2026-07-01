
#[derive(Debug, Default)]
pub struct ClockSync {
    offset_ns: i64,
    skew_ppm: f64,
    last_sync_time: u64,
    filter_alpha: f64,
}

#[derive(Debug)]
pub struct ClockSyncResult {
    pub offset_ns: i64,
    pub skew_ppm: f64,
    pub delay_ns: u64,
    pub quality: f32,
}

impl ClockSync {
    pub fn new() -> Self {
        Self {
            filter_alpha: 0.1, // Start with a fairly responsive filter
            ..Default::default()
        }
    }

    pub fn process_sync_exchange(
        &mut self,
        host_tx_time: u64,
        mcu_rx_time: u64,
        mcu_tx_time: u64,
        host_rx_time: u64,
    ) -> ClockSyncResult {
        let delay = ((host_rx_time as i64 - host_tx_time as i64) - (mcu_tx_time as i64 - mcu_rx_time as i64)) / 2;
        let offset = ((mcu_rx_time as i64 - host_tx_time as i64) + (mcu_tx_time as i64 - host_rx_time as i64)) / 2;

        if self.last_sync_time > 0 {
            let dt = (host_rx_time - self.last_sync_time) as f64;
            let skew = (offset - self.offset_ns) as f64 / dt;
            self.skew_ppm = (1.0 - self.filter_alpha) * self.skew_ppm + self.filter_alpha * skew * 1_000_000.0;
        }

        self.offset_ns = ((1.0 - self.filter_alpha) * self.offset_ns as f64 + self.filter_alpha * offset as f64) as i64;
        self.last_sync_time = host_rx_time;

        // Quality is inversely proportional to delay
        let quality = 1.0 / (1.0 + delay.abs() as f32 / 1000.0);

        ClockSyncResult {
            offset_ns: self.offset_ns,
            skew_ppm: self.skew_ppm,
            delay_ns: delay as u64,
            quality,
        }
    }

    pub fn translate_mcu_time_to_host_time(&self, mcu_time: u64) -> u64 {
        let estimated_offset = self.offset_ns as f64 + self.skew_ppm * (mcu_time - self.last_sync_time) as f64 / 1_000_000.0;
        (mcu_time as i64 + estimated_offset as i64) as u64
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_clock_sync_convergence() {
        let mut clock_sync = ClockSync::new();
        let mcu_clock_skew_ppm = 100.0; // MCU clock is 100ppm fast
        let mut mcu_time = 1_000_000_000;

        for i in 0..20 {
            let host_tx_time = i * 1_000_000_000;

            // Simulate MCU time advancing faster
            mcu_time += (1_000_000_000.0 * (1.0 + mcu_clock_skew_ppm / 1_000_000.0)) as u64;

            let mcu_rx_time = mcu_time;
            let mcu_tx_time = mcu_time + 1000; // 1us processing delay
            let host_rx_time = host_tx_time + 1_000_000_000 + 2000; // 1s round trip + 2us network delay

            clock_sync.process_sync_exchange(host_tx_time, mcu_rx_time, mcu_tx_time, host_rx_time);
        }

        // After 20 iterations, skew should be close to the simulated skew
        assert!((clock_sync.skew_ppm - mcu_clock_skew_ppm).abs() < 10.0);
    }

    #[test]
    fn test_translation() {
        let mut clock_sync = ClockSync::new();
        clock_sync.offset_ns = 1_000_000; // 1ms offset
        clock_sync.last_sync_time = 1_000_000_000;

        let mcu_time = 1_000_000_000 + 500_000_000;
        let host_time = clock_sync.translate_mcu_time_to_host_time(mcu_time);

        assert_eq!(host_time, mcu_time + 1_000_000);
    }
}
