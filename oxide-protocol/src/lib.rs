#![cfg_attr(not(feature = "std"), no_std)]

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

pub mod framing {
    use postcard::from_bytes;
    use postcard::to_vec;
    use serde::{Deserialize, Serialize};
    use heapless::Vec;


    pub fn encode_frame<T: Serialize>(msg: &T, buf: &mut [u8]) -> Result<usize, postcard::Error> {
        let encoded_vec: Vec<u8, 256> = to_vec(msg)?;
        let cobs_encoded = cobs::encode(&encoded_vec);
        if buf.len() < cobs_encoded.len() {
            return Err(postcard::Error::SerializeBufferFull);
        }
        buf[..cobs_encoded.len()].copy_from_slice(&cobs_encoded);
        Ok(cobs_encoded.len())
    }

    pub fn decode_frame<'a, T: Deserialize<'a>>(buf: &'a mut [u8]) -> Result<T, postcard::Error> {
        let mut decoded_cobs = [0u8; 256];
        let decoded_len = cobs::decode_in_place(&mut buf[..]).map_err(|_| postcard::Error::DeserializeBadEncoding)?;
        from_bytes(&buf[..decoded_len])
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::framing::{decode_frame, encode_frame};
    use heapless::Vec;

    #[test]
    fn test_host_to_mcu_sync_request_roundtrip() {
        let msg = HostToMcu::SyncRequest;
        let mut buf = [0u8; 256];
        let len = encode_frame(&msg, &mut buf).unwrap();

        let mut decode_buf = buf[..len].to_vec();
        let decoded: HostToMcu = decode_frame(&mut decode_buf).unwrap();
        assert_eq!(msg, decoded);
    }

    #[test]
    fn test_host_to_mcu_schedule_event_roundtrip() {
        let msg = HostToMcu::ScheduleEvent {
            channel: 1,
            timestamp_us: 123456789,
            duration_us: 1000,
        };
        let mut buf = [0u8; 256];
        let len = encode_frame(&msg, &mut buf).unwrap();

        let mut decode_buf = buf[..len].to_vec();
        let decoded: HostToMcu = decode_frame(&mut decode_buf).unwrap();
        assert_eq!(msg, decoded);
    }

    #[test]
    fn test_mcu_to_host_telemetry_batch_roundtrip() {
        let mut sensors: Vec<SensorData, 32> = Vec::new();
        sensors.push(SensorData { id: 1, raw_value: 1023, status: 0 }).unwrap();
        sensors.push(SensorData { id: 2, raw_value: 512, status: 1 }).unwrap();

        let msg = McuToHost::TelemetryBatch {
            timestamp_us: 987654321,
            sensors,
        };
        let mut buf = [0u8; 256];
        let len = encode_frame(&msg, &mut buf).unwrap();

        let mut decode_buf = buf[..len].to_vec();
        let decoded: McuToHost = decode_frame(&mut decode_buf).unwrap();
        assert_eq!(msg, decoded);
    }

    #[test]
    fn test_decode_corrupted_frame() {
        let mut buf = [0u8; 256];
        let msg = HostToMcu::SyncRequest;
        let len = encode_frame(&msg, &mut buf).unwrap();

        // Corrupt the buffer
        buf[2] = !buf[2];

        let mut decode_buf = buf[..len].to_vec();
        let decoded: Result<HostToMcu, _> = decode_frame(&mut decode_buf);
        assert!(decoded.is_err());
    }
}
