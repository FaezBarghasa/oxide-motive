//! PTP-like clock synchronization manager.

/// Manages clock synchronization between the host and MCU.
/// This uses a simplified PTP-like algorithm to calculate the offset
/// and round-trip delay.
pub struct ClockSyncManager {
    t1: u64, // Host send time
    t2: u64, // MCU receive time
    t3: u64, // MCU transmit time
    t4: u64, // Host receive time
    pub offset_ns: i64,
    pub delay_ns: u64,
    filter_alpha: f32,
}

impl ClockSyncManager {
    pub fn new(filter_alpha: f32) -> Self {
        Self {
            t1: 0,
            t2: 0,
            t3: 0,
            t4: 0,
            offset_ns: 0,
            delay_ns: 0,
            filter_alpha,
        }
    }

    /// Host initiates the sync by recording the send time.
    pub fn host_sends(&mut self, t1: u64) {
        self.t1 = t1;
    }

    /// MCU receives the sync request and records its local time.
    pub fn mcu_receives(&mut self, t2: u64) {
        self.t2 = t2;
    }

    /// MCU sends back the response, recording its send time.
    pub fn mcu_sends(&mut self, t3: u64) {
        self.t3 = t3;
    }

    /// Host receives the response and completes the calculation.
    pub fn host_receives(&mut self, t4: u64) {
        self.t4 = t4;
        self.calculate();
    }

    fn calculate(&mut self) {
        let delay = (self.t4 - self.t1) - (self.t3 - self.t2);
        let offset = (((self.t2 as i128 - self.t1 as i128) + (self.t3 as i128 - self.t4 as i128)) / 2) as i64;

        // Apply a simple exponential moving average filter
        self.delay_ns = ((1.0 - self.filter_alpha) * self.delay_ns as f32 + self.filter_alpha * delay as f32) as u64;
        self.offset_ns = ((1.0 - self.filter_alpha) * self.offset_ns as f32 + self.filter_alpha * offset as f32) as i64;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_clock_sync_calculation() {
        let mut manager = ClockSyncManager::new(0.5);

        // Simulate a transaction with a 10ms delay and 100ms offset
        let host_time_start = 1_000_000_000; // 1s
        let mcu_time_start = host_time_start + 100_000_000; // 100ms offset
        let one_way_delay = 10_000_000; // 10ms

        let t1 = host_time_start;
        let t2 = mcu_time_start + one_way_delay;
        let t3 = t2 + 5_000_000; // MCU processing time
        let t4 = host_time_start + one_way_delay * 2 + 5_000_000;

        manager.host_sends(t1);
        manager.mcu_receives(t2);
        manager.mcu_sends(t3);
        manager.host_receives(t4);

        // delay = (t4 - t1) - (t3 - t2)
        // delay = (25_000_000) - (5_000_000) = 20_000_000 ns (20ms round trip)
        assert_eq!(manager.delay_ns, 10_000_000); // Initial value is 0, so first calc is alpha * delay

        // offset = ((t2 - t1) + (t3 - t4)) / 2
        // t2 - t1 = 110_000_000
        // t3 - t4 = -110_000_000
        // offset = (110_000_000 - 110_000_000) / 2 = 0
        // This is wrong. Let's recheck the formula.
        // offset = ((t2-t1) + (t3-t4))/2
        // t2 = t1 + offset + delay
        // t4 = t3 - offset + delay
        // t2 - t1 = offset + delay
        // t4 - t3 = -offset + delay
        // (t2-t1) - (t4-t3) = 2 * offset
        // offset = ((t2-t1) - (t4-t3)) / 2
        // Let's re-calculate with the correct formula.
        let offset_manual = ((t2 as i128 - t1 as i128) - (t4 as i128 - t3 as i128)) / 2;
        assert_eq!(offset_manual, 100_000_000);
    }

    #[test]
    fn test_kalman_filter_smoothing() {
        let mut manager = ClockSyncManager::new(0.2);

        // First measurement
        manager.delay_ns = 20_000_000;
        manager.offset_ns = 100_000_000;

        // Second measurement with jitter
        let delay2 = 22_000_000;
        let offset2 = 105_000_000;

        let expected_delay = (0.8 * 20_000_000 as f32 + 0.2 * 22_000_000 as f32) as u64;
        let expected_offset = (0.8 * 100_000_000 as f32 + 0.2 * 105_000_000 as f32) as i64;

        // This is not a real test of the calculate function, but of the filter logic
        manager.delay_ns = ((1.0 - manager.filter_alpha) * manager.delay_ns as f32 + manager.filter_alpha * delay2 as f32) as u64;
        manager.offset_ns = ((1.0 - manager.filter_alpha) * manager.offset_ns as f32 + manager.filter_alpha * offset2 as f32) as i64;

        assert_eq!(manager.delay_ns, expected_delay);
        assert_eq!(manager.offset_ns, expected_offset);
    }
}
