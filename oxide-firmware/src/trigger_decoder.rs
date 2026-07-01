use heapless::Vec;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum SyncState {
    Searching,
    Synced,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum EnginePhase {
    Cylinder1TDC,
    Cylinder2TDC,
    Cylinder3TDC,
    Cylinder4TDC,
}

pub struct TriggerDecoder<const TEETH: usize> {
    state: SyncState,
    tooth_count: u8,
    last_capture: u32,
    avg_tooth_time: u32,
    rpm: u16,
    phase: EnginePhase,
}

impl<const TEETH: usize> TriggerDecoder<TEETH> {
    pub fn new() -> Self {
        Self {
            state: SyncState::Searching,
            tooth_count: 0,
            last_capture: 0,
            avg_tooth_time: 0,
            rpm: 0,
            phase: EnginePhase::Cylinder1TDC,
        }
    }

    pub fn process_edge(&mut self, capture: u32) -> (u16, EnginePhase, SyncState) {
        let delta = capture.wrapping_sub(self.last_capture);

        if self.state == SyncState::Searching {
            if delta > self.avg_tooth_time * 3 / 2 && self.avg_tooth_time > 0 {
                // Found missing tooth
                self.state = SyncState::Synced;
                self.tooth_count = 0;
                let rev_time = delta; // Approximation
                self.rpm = (60_000_000 * (TEETH as u32 -1)) as u16 / rev_time as u16;
            } else if self.avg_tooth_time == 0 {
                self.avg_tooth_time = delta;
            } else {
                self.avg_tooth_time = (self.avg_tooth_time * 7 + delta) / 8;
            }
        } else {
            // Synced
            self.tooth_count = (self.tooth_count + 1) % (TEETH as u8 - 1);
            if delta > self.avg_tooth_time * 3 / 2 {
                // Re-sync
                let rev_time = delta;
                self.rpm = (60_000_000 * (TEETH as u32 -1)) as u16 / rev_time as u16;
                self.tooth_count = 0;
            }
        }

        self.last_capture = capture;
        (self.rpm, self.phase, self.state)
    }
}

impl<const TEETH: usize> Default for TriggerDecoder<TEETH> {
    fn default() -> Self {
        Self::new()
    }
}
