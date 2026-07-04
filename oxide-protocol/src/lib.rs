#![no_std]

pub mod framing;

use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub struct TelemetryFrame {
    pub rpm: u16,
    pub map_kpa: u16,
    pub tps_percent: u8,
    pub afr: f32,
    pub ignition_advance_deg: f32,
    pub dwell_ms: f32,
    pub core_temp_c: f32,
}

#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub enum TuningCommand {
    WriteTableValue {
        table_id: u8, // 0 for VE, 1 for Spark
        row: u8,
        col: u8,
        value: f32,
    },
    SwitchActiveBank {
        bank_id: u8, // 0 for A, 1 for B
    },
}

#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub enum HostToMcu {
    Tuning(TuningCommand),
    Heartbeat,
}

#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub enum McuToHost {
    Telemetry(TelemetryFrame),
    TuningAck,
    TuningNak,
    Log(heapless::String<64>),
}

#[cfg(test)]
mod tests {
    use super::*;
    use postcard::{to_vec, from_bytes};
    use heapless::Vec;

    #[test]
    fn test_telemetry_frame_serialization() {
        let frame = TelemetryFrame {
            rpm: 3000,
            map_kpa: 101,
            tps_percent: 50,
            afr: 14.7,
            ignition_advance_deg: 25.5,
            dwell_ms: 2.5,
            core_temp_c: 85.0,
        };

        let mut buf: Vec<u8, 256> = to_vec(&frame).unwrap();
        let decoded: TelemetryFrame = from_bytes(&mut buf).unwrap();

        assert_eq!(frame, decoded);
    }

    #[test]
    fn test_tuning_command_serialization() {
        let cmd = TuningCommand::WriteTableValue {
            table_id: 0,
            row: 5,
            col: 10,
            value: 123.45,
        };

        let mut buf: Vec<u8, 256> = to_vec(&cmd).unwrap();
        let decoded: TuningCommand = from_bytes(&mut buf).unwrap();

        assert_eq!(cmd, decoded);
    }

    #[test]
    fn test_cobs_and_postcard_integration() {
        let frame = McuToHost::Telemetry(TelemetryFrame {
            rpm: 1500,
            map_kpa: 200,
            tps_percent: 100,
            afr: 11.5,
            ignition_advance_deg: 10.0,
            dwell_ms: 3.0,
            core_temp_c: 90.0,
        });

        let serialized: Vec<u8, 256> = to_vec(&frame).unwrap();

        let mut encoded = [0u8; 512];
        let encoded_len = framing::cobs_encode(&serialized, &mut encoded).unwrap();

        let mut decoded_cobs = [0u8; 512];
        let decoded_cobs_len = framing::cobs_decode(&encoded[..encoded_len-1], &mut decoded_cobs).unwrap();

        let final_frame: McuToHost = from_bytes(&mut decoded_cobs[..decoded_cobs_len]).unwrap();

        assert_eq!(frame, final_frame);
    }
}
