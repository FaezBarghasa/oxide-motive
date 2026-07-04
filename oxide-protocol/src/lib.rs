#![cfg_attr(not(feature = "std"), no_std)]

use heapless::Vec;
use serde::{Deserialize, Serialize};

/// Messages sent from the host (e.g., a Raspberry Pi) to the MCU (STM32H750).
#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub enum HostToMcu {
    /// Command to set the throttle position.
    SetThrottle(f32),
    /// Command to start the engine.
    StartEngine,
    /// Command to stop the engine.
    StopEngine,
    /// Request for telemetry data.
    RequestTelemetry,
    /// Clock synchronization message.
    ClockSync(ClockSync),
}

/// Messages sent from the MCU to the host.
#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub enum McuToHost {
    /// Telemetry data.
    Telemetry(Telemetry),
    /// Acknowledge a command from the host.
    Ack,
    /// Negative acknowledge a command from the host.
    Nak,
    /// Clock synchronization message.
    ClockSync(ClockSync),
    /// Telemetry data for autotuning.
    AutotuneTelemetry(AutotuneTelemetry),
}

/// Telemetry data from the MCU.
#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub struct Telemetry {
    /// Engine RPM.
    pub rpm: u16,
    /// Throttle position.
    pub throttle: f32,
    /// Engine temperature.
    pub temperature: f32,
}

/// Clock synchronization data.
#[derive(Serialize, Deserialize, Debug, PartialEq, Copy, Clone)]
pub struct ClockSync {
    /// The timestamp of the sender.
    pub origin_timestamp: u64,
    /// The timestamp of the receiver.
    pub receive_timestamp: u64,
    /// The timestamp of the sender when it sends the response.
    pub transmit_timestamp: u64,
}

/// Telemetry data for autotuning.
#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub struct AutotuneTelemetry {
    /// A vector of limit cycle data.
    pub limit_cycle_data: Vec<(f32, f32), 32>,
}
