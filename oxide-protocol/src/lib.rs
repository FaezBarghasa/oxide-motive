#![no_std]

use heapless;
use serde::{Serialize, Deserialize};

pub mod clock_sync;

#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub enum HostToMcu {
    SyncRequest { timestamp_us: u64 },
    ConfigUpdate { config: EcuConfig },
    TableUpdate { table_id: u8, x_idx: u8, y_idx: u8, value: f32 },
    ActuatorTest { channel: u8, duration_ms: u16 },
    Heartbeat,
}

#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub enum McuToHost {
    SyncResponse { timestamp_us: u64 },
    TelemetryBatch {
        timestamp_us: u64,
        sensors: heapless::Vec<SensorData, 32>,
        state: EngineState,
        rpm: u16,
    },
    DtcEvent { dtc_code: u16, freeze_frame: FreezeFrame },
    Ack { seq: u32 },
}

#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub struct SensorData {
    pub id: u8,
    pub raw_value: u16,
    pub physical_value: f32,
    pub status: u8,
}

#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub struct FreezeFrame {
    pub rpm: u16,
    pub map: u16,
    pub tps: u16,
    pub iat: i16,
    pub ect: i16,
}

#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub struct EcuConfig {
    // Placeholder
    pub injector_size: f32,
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone, Copy)]
pub enum EngineState {
    Offline,
    Cranking,
    Running,
    Stalled,
    Limp,
}

pub mod framing {
    use super::*;
    use cobs;
    use postcard::{from_bytes, to_slice};

    #[derive(Debug)]
    pub enum EncodeError {
        PostcardError(postcard::Error),
        CobsError,
    }

    #[derive(Debug)]
    pub enum DecodeError {
        PostcardError(postcard::Error),
        CobsError,
    }

    pub fn encode_frame<'a, T: Serialize>(
        msg: &T,
        buf: &'a mut [u8],
        seq: u32,
    ) -> Result<usize, EncodeError> {
        let mut frame_buf = [0u8; 1024]; // Temp buffer for postcard + seq
        let seq_bytes = seq.to_le_bytes();
        frame_buf[..4].copy_from_slice(&seq_bytes);

        let serialized = to_slice(msg, &mut frame_buf[4..]).map_err(EncodeError::PostcardError)?;
        let len_with_seq = serialized.len() + 4;

        let encoded_len = cobs::encode(&frame_buf[..len_with_seq], buf);
        Ok(encoded_len)
    }

    pub fn decode_frame<'a, T: serde::de::DeserializeOwned>(
        buf: &'a mut [u8],
    ) -> Result<(T, u32), DecodeError> {
        let decoded_len = cobs::decode_in_place(buf).map_err(|_| DecodeError::CobsError)?;
        let decoded_slice = &buf[..decoded_len];

        if decoded_slice.len() < 4 {
            return Err(DecodeError::CobsError); // Not enough data for sequence number
        }

        let seq_bytes: [u8; 4] = decoded_slice[..4].try_into().unwrap();
        let seq = u32::from_le_bytes(seq_bytes);

        let msg = from_bytes(&decoded_slice[4..]).map_err(DecodeError::PostcardError)?;
        Ok((msg, seq))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use framing::{decode_frame, encode_frame};
    use heapless::Vec;

    #[test]
    fn test_host_to_mcu_roundtrip() {
        let config = EcuConfig { injector_size: 550.0 };
        let msg = HostToMcu::ConfigUpdate { config };
        let mut buf = [0u8; 256];

        let encoded_len = encode_frame(&msg, &mut buf, 123).unwrap();
        let (decoded_msg, seq): (HostToMcu, u32) = decode_frame(&mut buf[..encoded_len]).unwrap();

        assert_eq!(msg, decoded_msg);
        assert_eq!(seq, 123);
    }

    #[test]
    fn test_mcu_to_host_roundtrip() {
        let mut sensors: Vec<SensorData, 32> = Vec::new();
        sensors.push(SensorData { id: 1, raw_value: 1023, physical_value: 5.0, status: 0 }).unwrap();
        let msg = McuToHost::TelemetryBatch {
            timestamp_us: 123456789,
            sensors,
            state: EngineState::Running,
            rpm: 3000,
        };
        let mut buf = [0u8; 256];

        let encoded_len = encode_frame(&msg, &mut buf, 456).unwrap();
        let (decoded_msg, seq): (McuToHost, u32) = decode_frame(&mut buf[..encoded_len]).unwrap();

        assert_eq!(msg, decoded_msg);
        assert_eq!(seq, 456);
    }

    #[test]
    fn test_cobs_edge_cases() {
        let msg = HostToMcu::Heartbeat;
        let mut buf = [0u8; 256];

        // Test with buffer full of zeros
        let mut zero_buf = [0u8; 64];
        let encoded_len = encode_frame(&msg, &mut zero_buf, 0).unwrap();
        let (decoded_msg, _): (HostToMcu, _) = decode_frame(&mut zero_buf[..encoded_len]).unwrap();
        assert_eq!(msg, decoded_msg);

        // Test with buffer full of 0xFF
        let mut ff_buf = [0xFFu8; 64];
        let encoded_len = encode_frame(&msg, &mut ff_buf, 0).unwrap();
        let (decoded_msg, _): (HostToMcu, _) = decode_frame(&mut ff_buf[..encoded_len]).unwrap();
        assert_eq!(msg, decoded_msg);
    }

    #[test]
    fn test_decode_corrupted() {
        let mut buf = [1, 2, 3, 4, 5, 0]; // Invalid COBS frame
        let result: Result<(HostToMcu, u32), _> = decode_frame(&mut buf);
        assert!(result.is_err());
    }
}
