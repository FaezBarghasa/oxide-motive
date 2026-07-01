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
                        // TODO: Capture and store freeze frame
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
        // TODO: Clear historical DTCs from flash
    }
}
