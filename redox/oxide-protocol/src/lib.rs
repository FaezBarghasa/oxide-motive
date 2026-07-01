pub mod framing;

use heapless::Vec;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub enum HostToMcu {
    SyncRequest,
    ScheduleEvent {
        channel: u8,
        timestamp_us: u64,
        duration_us: u16,
    },
    ConfigUpdate,
    Heartbeat,
}

#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub enum McuToHost {
    SyncResponse,
    TelemetryBatch {
        timestamp_us: u64,
        sensors: Vec<SensorData, 32>,
    },
    Ack,
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
pub struct SensorData {
    pub id: u8,
    pub raw_value: u16,
    pub status: u8,
}

pub struct ClockSync {
    pub offset_ns: i64,
    pub skew_ppm: f64,
    alpha: f64, // for exponential moving average
}

impl ClockSync {
    pub fn new() -> Self {
        Self {
            offset_ns: 0,
            skew_ppm: 0.0,
            alpha: 0.1, // Adjust this for more or less smoothing
        }
    }

    pub fn process_sync_exchange(
        &mut self,
        host_tx_time: u64,
        mcu_rx_time: u64,
        mcu_tx_time: u64,
        host_rx_time: u64,
    ) {
        let delay = ((host_rx_time as i64 - host_tx_time as i64) - (mcu_tx_time as i64 - mcu_rx_time as i64)) / 2;
        let offset = ((mcu_rx_time as i64 - host_tx_time as i64) + (mcu_tx_time as i64 - host_rx_time as i64)) / 2;

        let new_offset = offset - delay;
        self.offset_ns = (self.offset_ns as f64 * (1.0 - self.alpha) + new_offset as f64 * self.alpha) as i64;

        // Skew calculation is more involved, so I'll leave a TODO for now.
        // self.skew_ppm = ...
    }

    pub fn translate_mcu_time_to_host_time(&self, mcu_time: u64) -> u64 {
        (mcu_time as i64 + self.offset_ns) as u64
    }
}
