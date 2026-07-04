//! 60-2 Missing Tooth Crank Decoder
#![no_std]
use heapless::spsc::Queue;

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum EngineSyncState {
    NoSignal,
    Searching,
    Synced,
}

pub struct TriggerDecoder {
    pub state: EngineSyncState,
    pub rpm: f32,
    pub crank_angle_deg: f32,

    last_tooth_time_us: u32,
    tooth_count: u8,
    expected_interval_us: u32,

    // Lock-free ring buffer for Host telemetry logging
    timestamp_log: Queue<u32, 64>,
}

impl TriggerDecoder {
    pub fn new() -> Self {
        Self {
            state: EngineSyncState::NoSignal,
            rpm: 0.0,
            crank_angle_deg: 0.0,
            last_tooth_time_us: 0,
            tooth_count: 0,
            expected_interval_us: 0,
            timestamp_log: Queue::new(),
        }
    }

    /// Called directly from the Hardware Timer Input Capture ISR.
    /// Returns the calculated RPM if the engine is fully synced.
    pub fn handle_edge(&mut self, timestamp_us: u32) -> Option<f32> {
        // Log timestamp for Host (non-blocking, drops if full)
        let _ = self.timestamp_log.enqueue(timestamp_us);

        let dt = timestamp_us.wrapping_sub(self.last_tooth_time_us);
        self.last_tooth_time_us = timestamp_us;

        if self.state == EngineSyncState::NoSignal {
            self.expected_interval_us = dt;
            self.state = EngineSyncState::Searching;
            self.tooth_count = 1;
            return None;
        }

        // Detect missing tooth gap (dt > 2.5x expected interval)
        let threshold = self.expected_interval_us + (self.expected_interval_us >> 1) + (self.expected_interval_us >> 2);

        if dt > threshold {
            // Gap detected
            self.state = EngineSyncState::Synced;
            self.tooth_count = 0;
            // Recalculate expected interval based on the gap (which represents 2 teeth)
            self.expected_interval_us = dt / 2;
        } else {
            self.tooth_count += 1;
            // Update running average of tooth interval for jitter rejection
            self.expected_interval_us = (self.expected_interval_us * 3 + dt) >> 2;

            if self.tooth_count >= 60 {
                self.tooth_count = 0; // Reset for next revolution
            }
        }

        // Calculate RPM: 60 teeth per rev.
        // Time for one full revolution = 60 * expected_interval_us
        let rev_time_us = (self.expected_interval_us as f32) * 60.0;
        if rev_time_us > 0.0 {
            // RPM = 60,000,000 us/min / rev_time_us
            self.rpm = 60_000_000.0 / rev_time_us;
        }

        // Calculate instantaneous crank angle (0 to 720 degrees for 4-stroke)
        // Angle per tooth = 360 / 60 = 6 degrees
        self.crank_angle_deg = (self.tooth_count as f32) * 6.0;

        if self.state == EngineSyncState::Synced {
            Some(self.rpm)
        } else {
            None
        }
    }

    pub fn get_timestamp_log(&mut self) -> &mut Queue<u32, 64> {
        &mut self.timestamp_log
    }
}
