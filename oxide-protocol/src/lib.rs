#![no_std]

pub mod framing;
pub mod clock_sync;

use serde::{Serialize, Deserialize};
use heapless::Vec;

#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub enum HostToMcu {
    SyncRequest,
    ConfigUpdate,
    TableUpdate,
    ActuatorTest,
}

#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub struct LimitCycleData {
    pub peaks: Vec<f32, 10>,
    pub peak_times: Vec<u64, 10>,
}

#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub enum McuToHost {
    SyncResponse,
    TelemetryBatch,
    DtcEvent,
    AutotuneTelemetry(LimitCycleData),
}
