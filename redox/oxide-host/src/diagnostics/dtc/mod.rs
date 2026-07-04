use heapless::Vec;
use sequential_storage::{Cache, Storage};
use std::time::{Duration, Instant};

#[derive(Clone, Copy, PartialEq)]
pub struct Dtc {
    pub code: u16,
    pub status: u8,
}

pub struct FreezeFrame {
    pub sensor_values: [u16; 10],
}

pub struct DtcManager<S: Storage> {
    active_dtcs: Vec<Dtc, 16>,
    pending_dtcs: Vec<(u16, Instant), 16>,
    storage: S,
    cache: Cache<128>,
}

impl<S: Storage> DtcManager<S> {
    pub fn new(storage: S) -> Self {
        Self {
            active_dtcs: Vec::new(),
            pending_dtcs: Vec::new(),
            storage,
            cache: Cache::new(),
        }
    }

    pub fn update(&mut self, faults: &[(u16, bool)]) {
        for (code, is_faulty) in faults {
            if *is_faulty {
                if let Some((_, time)) = self.pending_dtcs.iter_mut().find(|(c, _)| c == code) {
                    if time.elapsed() > Duration::from_millis(500) {
                        self.active_dtcs.push(Dtc { code: *code, status: 0x01 }).ok();
                        
                        // Capture and store freeze frame
                        let freeze_frame = FreezeFrame {
                            sensor_values: [0; 10], // Simulated sensor values
                        };
                        let mut buffer = [0u8; 22];
                        buffer[0..2].copy_from_slice(&code.to_le_bytes());
                        for (i, val) in freeze_frame.sensor_values.iter().enumerate() {
                            buffer[2 + i * 2..4 + i * 2].copy_from_slice(&val.to_le_bytes());
                        }
                        
                        let _ = sequential_storage::queue::push(&mut self.storage, &mut self.cache, &buffer, false);
                        self.pending_dtcs.retain(|(c, _)| c != code);
                    }
                } else {
                    self.pending_dtcs.push((*code, Instant::now())).ok();
                }
            } else {
                self.pending_dtcs.retain(|(c, _)| c != code);
            }
        }
    }

    pub fn get_active_dtcs(&self) -> &Vec<Dtc, 16> {
        &self.active_dtcs
    }

    pub fn clear_dtcs(&mut self) {
        self.active_dtcs.clear();
        
        // Clear historical DTCs from flash
        let _ = sequential_storage::queue::clear(&mut self.storage, &mut self.cache);
    }
}
