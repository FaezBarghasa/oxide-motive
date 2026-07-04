use serde::{Serialize, de::DeserializeOwned};
use postcard::{to_slice, from_bytes};
use heapless::Vec;

#[derive(Debug, PartialEq)]
pub enum Error {
    EncodeError,
    DecodeError,
    PostcardError,
}

pub fn cobs_encode(data: &[u8], buffer: &mut [u8]) -> Result<usize, Error> {
    let mut write_idx = 1;
    let mut code_idx = 0;
    let mut code = 1;

    for &byte in data {
        if byte == 0 {
            buffer[code_idx] = code;
            code_idx = write_idx;
            code = 1;
            write_idx += 1;
        } else {
            if write_idx >= buffer.len() {
                return Err(Error::EncodeError);
            }
            buffer[write_idx] = byte;
            code += 1;
            write_idx += 1;
            if code == 0xFF {
                buffer[code_idx] = code;
                code_idx = write_idx;
                code = 1;
                write_idx += 1;
            }
        }
    }

    if code_idx >= buffer.len() || write_idx >= buffer.len() {
        return Err(Error::EncodeError);
    }

    buffer[code_idx] = code;
    buffer[write_idx] = 0;

    Ok(write_idx + 1)
}

pub fn cobs_decode(data: &[u8], buffer: &mut [u8]) -> Result<usize, Error> {
    let mut read_idx = 0;
    let mut write_idx = 0;

    while read_idx < data.len() {
        let code = data[read_idx];
        read_idx += 1;

        if code == 0 {
            return Err(Error::DecodeError);
        }

        for _ in 1..code {
            if read_idx >= data.len() || write_idx >= buffer.len() {
                return Err(Error::DecodeError);
            }
            buffer[write_idx] = data[read_idx];
            read_idx += 1;
            write_idx += 1;
        }

        if code < 0xFF && read_idx < data.len() {
            if write_idx >= buffer.len() {
                return Err(Error::DecodeError);
            }
            buffer[write_idx] = 0;
            write_idx += 1;
        }
    }

    Ok(write_idx)
}

pub fn encode_frame<T: Serialize>(frame: &T, buffer: &mut [u8]) -> Result<usize, Error> {
    let mut postcard_buffer = [0u8; 256];
    let used = to_slice(frame, &mut postcard_buffer).map_err(|_| Error::PostcardError)?;
    cobs_encode(used, buffer)
}

pub fn decode_frame<'a, T: DeserializeOwned<'a>>(data: &'a [u8], buffer: &'a mut [u8]) -> Result<T, Error> {
    let decoded_len = cobs_decode(data, buffer)?;
    from_bytes(&buffer[..decoded_len]).map_err(|_| Error::PostcardError)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{HostToMcu, McuToHost, LimitCycleData};
    use heapless::Vec;

    #[test]
    fn test_cobs_encode_decode() {
        let data = b"hello\x00world";
        let mut encoded = [0u8; 20];
        let encoded_len = cobs_encode(data, &mut encoded).unwrap();
        assert_eq!(&encoded[..encoded_len], b"\x06hello\x06world\x00");

        let mut decoded = [0u8; 20];
        let decoded_len = cobs_decode(&encoded[..encoded_len - 1], &mut decoded).unwrap();
        assert_eq!(&decoded[..decoded_len], data);
    }

    #[test]
    fn test_cobs_encode_decode_edge_cases() {
        // Test with a buffer full of zeros
        let data = [0u8; 10];
        let mut encoded = [0u8; 20];
        let encoded_len = cobs_encode(&data, &mut encoded).unwrap();
        let mut decoded = [0u8; 20];
        let decoded_len = cobs_decode(&encoded[..encoded_len - 1], &mut decoded).unwrap();
        assert_eq!(&decoded[..decoded_len], &data[..]);

        // Test with a buffer full of non-zero values
        let data = [1u8; 255];
        let mut encoded = [0u8; 300];
        let encoded_len = cobs_encode(&data, &mut encoded).unwrap();
        let mut decoded = [0u8; 300];
        let decoded_len = cobs_decode(&encoded[..encoded_len - 1], &mut decoded).unwrap();
        assert_eq!(&decoded[..decoded_len], &data[..]);
    }

    #[test]
    fn test_frame_encoding_decoding() {
        let frame = HostToMcu::SyncRequest;
        let mut encoded = [0u8; 20];
        let encoded_len = encode_frame(&frame, &mut encoded).unwrap();

        let mut decoded_buffer = [0u8; 20];
        let decoded_frame: HostToMcu = decode_frame(&encoded[..encoded_len - 1], &mut decoded_buffer).unwrap();
        assert_eq!(frame, decoded_frame);
    }

    #[test]
    fn test_autotune_telemetry_serialization() {
        let mut peaks = Vec::new();
        peaks.push(1.0).unwrap();
        peaks.push(2.0).unwrap();
        let mut peak_times = Vec::new();
        peak_times.push(100).unwrap();
        peak_times.push(200).unwrap();

        let frame = McuToHost::AutotuneTelemetry(LimitCycleData {
            peaks,
            peak_times,
        });

        let mut encoded = [0u8; 100];
        let encoded_len = encode_frame(&frame, &mut encoded).unwrap();

        let mut decoded_buffer = [0u8; 100];
        let decoded_frame: McuToHost = decode_frame(&encoded[..encoded_len - 1], &mut decoded_buffer).unwrap();
        assert_eq!(frame, decoded_frame);
    }
}
