use embassy_stm32::interrupt;

#[derive(Debug, PartialEq)]
pub enum SyncState {
    Searching,
    Synced,
}

pub struct TriggerDecoder {
    pub tooth_count: u32,
    pub sync_state: SyncState,
    pub last_tooth_times: [u32; 2],
    pub rpm: u32,
}

impl TriggerDecoder {
    pub fn new() -> Self {
        Self {
            tooth_count: 0,
            sync_state: SyncState::Searching,
            last_tooth_times: [0, 0],
            rpm: 0,
        }
    }

    pub fn process_edge(&mut self, timestamp: u32) {
        let delta = timestamp.wrapping_sub(self.last_tooth_times[0]);
        self.last_tooth_times[1] = self.last_tooth_times[0];
        self.last_tooth_times[0] = timestamp;

        if self.sync_state == SyncState::Searching {
            let last_delta = self.last_tooth_times[0].wrapping_sub(self.last_tooth_times[1]);
            if delta > last_delta * 3 / 2 {
                self.sync_state = SyncState::Synced;
                self.tooth_count = 0;
            }
        } else {
            self.tooth_count += 1;
            if self.tooth_count == 35 {
                let rev_time = timestamp.wrapping_sub(self.last_tooth_times[1]);
                self.rpm = (60_000_000 * 36) / rev_time;
            }
        }
    }
}

#[interrupt]
fn TIM3() {
    // TODO: Get timestamp from input capture register
    // TODO: Call trigger_decoder.process_edge(timestamp)
}
