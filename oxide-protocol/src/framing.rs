
use serde::{Serialize, Deserialize};
use postcard::{to_slice, from_bytes};
use cobs;

const MAX_FRAME_SIZE: usize = 256;

pub fn encode_frame<'a, T: Serialize>(msg: &T, buf: &'a mut [u8]) -> Result<usize, postcard::Error> {
    let mut temp_buf = [0u8; MAX_FRAME_SIZE];
    let serialized = to_slice(msg, &mut temp_buf)?;
    let encoded_len = cobs::encode(serialized, buf);
    Ok(encoded_len)
}

pub fn decode_frame<'a, T: Deserialize<'a>>(buf: &'a mut [u8]) -> Result<T, postcard::Error> {
    let decoded_len = cobs::decode_in_place(buf).map_err(|_| postcard::Error::DecodeBadRustEnum)?;
    let msg = from_bytes(&buf[..decoded_len])?;
    Ok(msg)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{HostToMcu, McuToHost, SensorData};
    use heapless::Vec;

    #[test]
    fn test_host_to_mcu_framing() {
        let msg = HostToMcu::ScheduleEvent {
            channel: 1,
            timestamp_us: 123456,
            duration_us: 1000,
        };

        let mut buf = [0u8; MAX_FRAME_SIZE];
        let encoded_len = encode_frame(&msg, &mut buf).unwrap();

        let mut decode_buf = buf;
        let decoded_msg: HostToMcu = decode_frame(&mut decode_buf[..encoded_len]).unwrap();

        assert_eq!(msg, decoded_msg);
    }

    #[test]
    fn test_mcu_to_host_framing() {
        let mut sensors: Vec<SensorData, 32> = Vec::new();
        sensors.push(SensorData { id: 1, raw_value: 1024, status: 0 }).unwrap();
        sensors.push(SensorData { id: 2, raw_value: 2048, status: 0 }).unwrap();

        let msg = McuToHost::TelemetryBatch {
            timestamp_us: 987654,
            sensors,
        };

        let mut buf = [0u8; MAX_FRAME_SIZE];
        let encoded_len = encode_frame(&msg, &mut buf).unwrap();

        let mut decode_buf = buf;
        let decoded_msg: McuToHost = decode_frame(&mut decode_buf[..encoded_len]).unwrap();

        assert_eq!(msg, decoded_msg);
    }

    #[test]
    fn test_corrupted_frame() {
        let mut buf = [0x05, 0x04, 0x03, 0x02, 0x01]; // Invalid COBS frame
        let result: Result<HostToMcu, _> = decode_frame(&mut buf);
        assert!(result.is_err());
    }
}
