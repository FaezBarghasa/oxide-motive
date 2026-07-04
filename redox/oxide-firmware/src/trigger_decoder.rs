const TIMER_FREQ: f32 = 100_000_000.0; // 100MHz
const TEETH_PER_REV: u8 = 58; // 60-2

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum EngineSyncState {
    NoSignal,
    SearchingForGap,
    GapDetected,
    FullySynchronized,
}

pub struct TriggerDecoder {
    sync_state: EngineSyncState,
    last_tooth_time: u32,
    expected_tooth_interval: u32,
    current_tooth_index: u8,
    engine_speed_rpm: f32,
}

impl TriggerDecoder {
    pub fn new() -> Self {
        Self {
            sync_state: EngineSyncState::NoSignal,
            last_tooth_time: 0,
            expected_tooth_interval: 0,
            current_tooth_index: 0,
            engine_speed_rpm: 0.0,
        }
    }

    pub fn handle_interrupt_pulse(&mut self, timestamp: u32) -> Option<f32> {
        let dt = timestamp.wrapping_sub(self.last_tooth_time);
        self.last_tooth_time = timestamp;

        match self.sync_state {
            EngineSyncState::NoSignal => {
                self.sync_state = EngineSyncState::SearchingForGap;
                self.expected_tooth_interval = dt;
            }
            EngineSyncState::SearchingForGap => {
                if dt > (self.expected_tooth_interval * 5) / 2 {
                    self.sync_state = EngineSyncState::GapDetected;
                    self.current_tooth_index = 0;
                } else {
                    // Simple moving average
                    self.expected_tooth_interval = (self.expected_tooth_interval + dt) / 2;
                }
            }
            EngineSyncState::GapDetected => {
                self.sync_state = EngineSyncState::FullySynchronized;
                self.expected_tooth_interval = dt;
            }
            EngineSyncState::FullySynchronized => {
                if dt > (self.expected_tooth_interval * 5) / 2 {
                    self.current_tooth_index = 0;
                } else {
                    self.current_tooth_index += 1;
                    if self.current_tooth_index >= TEETH_PER_REV {
                        self.current_tooth_index = 0;
                    }
                    self.expected_tooth_interval = (self.expected_tooth_interval * 7 + dt) / 8; // EMA
                }
            }
        }

        if self.sync_state == EngineSyncState::FullySynchronized {
            self.engine_speed_rpm = (TIMER_FREQ * 60.0) / (self.expected_tooth_interval as f32 * TEETH_PER_REV as f32);
            Some(self.engine_speed_rpm)
        } else {
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_steady_state() {
        let mut decoder = TriggerDecoder::new();
        let mut timestamp = 0;
        let dt = 1_000_000; // 6000 RPM

        // Initial sync
        for _ in 0..60 {
            timestamp += dt;
            decoder.handle_interrupt_pulse(timestamp);
        }
        timestamp += dt * 3; // Gap
        decoder.handle_interrupt_pulse(timestamp);

        // Steady state
        for _ in 0..120 {
            timestamp += dt;
            let rpm = decoder.handle_interrupt_pulse(timestamp);
            if rpm.is_some() {
                assert!((rpm.unwrap() - 6000.0).abs() < 100.0);
            }
        }
    }

    #[test]
    fn test_acceleration() {
        let mut decoder = TriggerDecoder::new();
        let mut timestamp = 0;
        let mut dt = 1_000_000; // 6000 RPM

        // Initial sync
        for _ in 0..60 {
            timestamp += dt;
            decoder.handle_interrupt_pulse(timestamp);
        }
        timestamp += dt * 3; // Gap
        decoder.handle_interrupt_pulse(timestamp);

        // Acceleration
        for _ in 0..120 {
            dt -= 1000;
            timestamp += dt;
            decoder.handle_interrupt_pulse(timestamp);
        }
        assert!(decoder.engine_speed_rpm > 6000.0);
    }

    #[test]
    fn test_glitch_rejection() {
        let mut decoder = TriggerDecoder::new();
        let mut timestamp = 0;
        let dt = 1_000_000; // 6000 RPM

        // Initial sync
        for _ in 0..60 {
            timestamp += dt;
            decoder.handle_interrupt_pulse(timestamp);
        }
        timestamp += dt * 3; // Gap
        decoder.handle_interrupt_pulse(timestamp);

        // Glitch
        timestamp += dt / 4;
        let rpm_before = decoder.engine_speed_rpm;
        decoder.handle_interrupt_pulse(timestamp);
        let rpm_after = decoder.engine_speed_rpm;

        assert_eq!(rpm_before, rpm_after);
    }
}
