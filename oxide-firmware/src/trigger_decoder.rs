use heapless::Deque;

#[derive(Debug, PartialEq)]
pub enum SyncState {
    NoSignal,
    Searching,
    Synced,
}

pub struct TriggerDecoder {
    state: SyncState,
    last_tooth_time: u32,
    tooth_times: Deque<u32, 64>,
    missing_tooth_detected: bool,
}

impl TriggerDecoder {
    pub fn new() -> Self {
        Self {
            state: SyncState::NoSignal,
            last_tooth_time: 0,
            tooth_times: Deque::new(),
            missing_tooth_detected: false,
        }
    }

    pub fn handle_interrupt_pulse(&mut self, timestamp: u32) -> (u16, u32, SyncState) {
        let delta = timestamp.wrapping_sub(self.last_tooth_time);
        self.last_tooth_time = timestamp;

        if self.state == SyncState::NoSignal {
            self.state = SyncState::Searching;
            self.tooth_times.clear();
        }

        if self.tooth_times.len() > 2 {
            let avg_delta = self.tooth_times.iter().sum::<u32>() / self.tooth_times.len() as u32;
            if delta > avg_delta * 1.5 {
                self.missing_tooth_detected = true;
                self.state = SyncState::Synced;
                self.tooth_times.clear();
            }
        }

        if self.tooth_times.is_full() {
            self.tooth_times.pop_front();
        }
        self.tooth_times.push_back(delta).ok();

        let rpm = if self.state == SyncState::Synced {
            let revolution_time: u32 = self.tooth_times.iter().sum();
            (60_000_000 / revolution_time) as u16
        } else {
            0
        };

        (rpm, 0, self.state.clone())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_trigger_decoder() {
        let mut decoder = TriggerDecoder::new();
        let mut timestamp = 0;
        // Simulate 60-2 wheel at 1200 RPM (50ms per revolution, 0.833ms per tooth)
        for i in 0..58 {
            timestamp += 833;
            decoder.handle_interrupt_pulse(timestamp);
        }
        timestamp += 833 * 3; // Missing teeth
        let (rpm, _, state) = decoder.handle_interrupt_pulse(timestamp);

        assert_eq!(state, SyncState::Synced);
        assert!(rpm > 1100 && rpm < 1300);
    }
}
