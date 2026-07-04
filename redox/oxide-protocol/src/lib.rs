#![no_std]

use serde::{Serialize, Deserialize};

pub mod framing;

#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub struct TelemetryFrame {
    pub rpm: u16,
    pub map: u16,
    pub tps: u16,
    pub afr: u16,
    pub advance: i16,
}

#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub struct TuningCommand {
    pub table_id: u8,
    pub x: u8,
    pub y: u8,
    pub value: f32,
}

#[cfg(test)]
mod tests {
    use super::*;
    use postcard::{to_slice, from_bytes};

    #[test]
    fn test_telemetry_roundtrip() {
        let frame = TelemetryFrame {
            rpm: 6000,
            map: 1013,
            tps: 50,
            afr: 147,
            advance: 30,
        };

        let mut buf = [0u8; 128];
        let serialized = to_slice(&frame, &mut buf).unwrap();

        let mut encoded = [0u8; 128];
        let encoded_len = framing::cobs_encode(serialized, &mut encoded).unwrap();

        let mut decoded = [0u8; 128];
        let decoded_len = framing::cobs_decode(&encoded[..encoded_len], &mut decoded).unwrap();

        let deserialized: TelemetryFrame = from_bytes(&decoded[..decoded_len]).unwrap();

        assert_eq!(frame, deserialized);
    }

    #[test]
    fn test_cobs_mutation() {
        let data = [1, 2, 3, 0, 4, 5, 6];
        let mut encoded = [0u8; 16];
        let encoded_len = framing::cobs_encode(&data, &mut encoded).unwrap();

        // Mutate a byte
        encoded[3] = encoded[3].wrapping_add(1);

        let mut decoded = [0u8; 16];
        let result = framing::cobs_decode(&encoded[..encoded_len], &mut decoded);
        assert!(result.is_err());
    }
}
