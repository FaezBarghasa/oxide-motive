//! 60-2 missing-tooth crank trigger decoder.

use heapless::spsc::{Queue, Producer, Consumer};

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum EngineSyncState {
    NoSignal,
    SearchingForGap,
    GapDetected,
    FullySynchronized,
}

pub struct TriggerDecoder {
    pub sync_state: EngineSyncState,
    last_tooth_time: u32,
    expected_tooth_interval: u32,
    current_tooth_index: u8,
    pub engine_speed_rpm: f32,
    timer_freq: u32,
    timestamp_producer: Producer<'static, u32, 256>,
}

impl TriggerDecoder {
    pub fn new(timer_freq: u32, producer: Producer<'static, u32, 256>) -> Self {
        Self {
            sync_state: EngineSyncState::NoSignal,
            last_tooth_time: 0,
            expected_tooth_interval: 0,
            current_tooth_index: 0,
            engine_speed_rpm: 0.0,
            timer_freq,
            timestamp_producer: producer,
        }
    }

    /// Handles a hardware input capture interrupt for a crank tooth.
    /// Returns the calculated RPM if a full rotation is detected.
    pub fn handle_interrupt_pulse(&mut self, timestamp: u32) -> Option<f32> {
        self.timestamp_producer.enqueue(timestamp).ok();

        if self.last_tooth_time == 0 {
            self.last_tooth_time = timestamp;
            return None;
        }

        let dt = timestamp.wrapping_sub(self.last_tooth_time);
        self.last_tooth_time = timestamp;

        match self.sync_state {
            EngineSyncState::NoSignal | EngineSyncState::SearchingForGap => {
                if self.expected_tooth_interval > 0 && dt > (self.expected_tooth_interval * 25 / 10) { // 2.5x expected
                    // Gap detected
                    self.sync_state = EngineSyncState::GapDetected;
                    self.current_tooth_index = 0;
                    // Calculate RPM based on the time for 58 teeth
                    let time_for_58_teeth = self.expected_tooth_interval * 58;
                    if time_for_58_teeth > 0 {
                        self.engine_speed_rpm = (60.0 * self.timer_freq as f32) / time_for_58_teeth as f32;
                    }
                    return Some(self.engine_speed_rpm);
                } else {
                    // Update expected interval with a simple moving average
                    self.expected_tooth_interval = (self.expected_tooth_interval * 7 + dt) / 8;
                    self.sync_state = EngineSyncState::SearchingForGap;
                }
            }
            EngineSyncState::GapDetected | EngineSyncState::FullySynchronized => {
                self.current_tooth_index += 1;
                if dt > (self.expected_tooth_interval * 25 / 10) { // 2.5x expected
                    // Found the gap again
                    self.sync_state = EngineSyncState::FullySynchronized;
                    self.current_tooth_index = 0;
                    let time_for_58_teeth = self.expected_tooth_interval * 58;
                     if time_for_58_teeth > 0 {
                        self.engine_speed_rpm = (60.0 * self.timer_freq as f32) / time_for_58_teeth as f32;
                    }
                    return Some(self.engine_speed_rpm);
                }
                // Update expected interval
                self.expected_tooth_interval = (self.expected_tooth_interval * 7 + dt) / 8;
            }
        }
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use heapless::spsc::Queue;

    const TIMER_FREQ: u32 = 1_000_000; // 1MHz timer

    fn generate_60_2_pulses(rpm: u32, num_cycles: usize) -> Vec<u32> {
        let mut pulses = Vec::new();
        let teeth_per_rev = 60;
        let total_teeth_in_cycle = teeth_per_rev - 2;
        let time_per_rev_s = 60.0 / rpm as f32;
        let time_per_tooth_s = time_per_rev_s / teeth_per_rev as f32;
        let time_per_tooth_ticks = (time_per_tooth_s * TIMER_FREQ as f32) as u32;

        let mut current_time = 0;
        for _ in 0..num_cycles {
            for tooth in 0..total_teeth_in_cycle {
                pulses.push(current_time);
                current_time += time_per_tooth_ticks;
            }
            // Skip 2 teeth for the gap
            current_time += time_per_tooth_ticks * 2;
        }
        pulses
    }

    #[test]
    fn test_decoder_syncs_and_calculates_rpm() {
        static mut Q: Queue<u32, 256> = Queue::new();
        let (p, _c) = unsafe { Q.split() };
        let mut decoder = TriggerDecoder::new(TIMER_FREQ, p);
        let pulses = generate_60_2_pulses(1000, 3);

        let mut final_rpm = 0.0;
        for pulse in pulses {
            if let Some(rpm) = decoder.handle_interrupt_pulse(pulse) {
                final_rpm = rpm;
            }
        }

        assert_eq!(decoder.sync_state, EngineSyncState::FullySynchronized);
        assert!(final_rpm > 990.0 && final_rpm < 1010.0);
    }

    #[test]
    fn test_decoder_handles_acceleration() {
        static mut Q: Queue<u32, 256> = Queue::new();
        let (p, _c) = unsafe { Q.split() };
        let mut decoder = TriggerDecoder::new(TIMER_FREQ, p);

        let mut pulses = generate_60_2_pulses(1000, 2);
        let accel_pulses = generate_60_2_pulses(2000, 2);
        pulses.extend(accel_pulses.iter().map(|p| p + pulses.last().unwrap_or(&0)));

        let mut final_rpm = 0.0;
        for pulse in pulses {
            if let Some(rpm) = decoder.handle_interrupt_pulse(pulse) {
                final_rpm = rpm;
            }
        }

        assert_eq!(decoder.sync_state, EngineSyncState::FullySynchronized);
        assert!(final_rpm > 1980.0 && final_rpm < 2020.0);
    }

    #[test]
    fn test_decoder_handles_noisy_signal() {
        static mut Q: Queue<u32, 256> = Queue::new();
        let (p, _c) = unsafe { Q.split() };
        let mut decoder = TriggerDecoder::new(TIMER_FREQ, p);

        let mut pulses = generate_60_2_pulses(1000, 3);
        // Add a noise pulse
        pulses.insert(10, pulses[9] + 50);

        let mut final_rpm = 0.0;
        for pulse in pulses {
            if let Some(rpm) = decoder.handle_interrupt_pulse(pulse) {
                final_rpm = rpm;
            }
        }

        assert_eq!(decoder.sync_state, EngineSyncState::FullySynchronized);
        assert!(final_rpm > 990.0 && final_rpm < 1010.0);
    }
}
