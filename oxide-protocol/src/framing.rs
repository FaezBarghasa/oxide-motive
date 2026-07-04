//! Consistent Overhead Byte Stuffing (COBS) framing implementation.

#[derive(Debug, PartialEq)]
pub enum FramingError {
    OutputBufferTooSmall,
    InputBufferTooSmall,
    InvalidEncoding,
}

/// Encodes a byte slice using COBS.
/// The output buffer must be at least `input.len() + 2` bytes long.
pub fn cobs_encode(input: &[u8], output: &mut [u8]) -> Result<usize, FramingError> {
    if output.len() < input.len() + 1 {
        return Err(FramingError::OutputBufferTooSmall);
    }

    let mut write_idx = 1;
    let mut code_idx = 0;
    let mut code = 1;

    for &byte in input {
        if byte == 0 {
            output[code_idx] = code;
            code_idx = write_idx;
            code = 1;
            write_idx += 1;
        } else {
            if write_idx >= output.len() {
                return Err(FramingError::OutputBufferTooSmall);
            }
            output[write_idx] = byte;
            code += 1;
            write_idx += 1;
            if code == 0xFF {
                output[code_idx] = code;
                code_idx = write_idx -1; // This is subtle
                code = 1;
            }
        }
    }

    output[code_idx] = code;
    output[write_idx] = 0; // End of packet marker

    Ok(write_idx + 1)
}

/// Decodes a COBS-encoded byte slice.
/// The output buffer should be at least `input.len() - 1` bytes long.
pub fn cobs_decode(input: &[u8], output: &mut [u8]) -> Result<usize, FramingError> {
    if input.is_empty() {
        return Ok(0);
    }
    if output.len() < input.len() -1 {
        return Err(FramingError::OutputBufferTooSmall);
    }

    let mut read_idx = 0;
    let mut write_idx = 0;

    while read_idx < input.len() {
        let code = input[read_idx];
        if code == 0 {
            return Err(FramingError::InvalidEncoding);
        }
        read_idx += 1;

        for _ in 1..code {
            if read_idx >= input.len() || write_idx >= output.len() {
                return Err(FramingError::InvalidEncoding);
            }
            output[write_idx] = input[read_idx];
            read_idx += 1;
            write_idx += 1;
        }

        if code < 0xFF && read_idx < input.len() {
            if write_idx >= output.len() {
                return Err(FramingError::OutputBufferTooSmall);
            }
            output[write_idx] = 0;
            write_idx += 1;
        }
    }

    // The last byte is a zero, so we remove it from the output
    Ok(write_idx - 1)
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_encode_decode_simple() {
        let data = b"\x11\x22\x00\x33";
        let mut encoded = [0u8; 16];
        let mut decoded = [0u8; 16];

        let encoded_len = cobs_encode(data, &mut encoded).unwrap();
        assert_eq!(&encoded[..encoded_len], b"\x03\x11\x22\x02\x33\x00");

        let decoded_len = cobs_decode(&encoded[..encoded_len-1], &mut decoded).unwrap();
        assert_eq!(decoded_len, data.len());
        assert_eq!(&decoded[..decoded_len], data);
    }

    #[test]
    fn test_encode_decode_no_zeros() {
        let data = b"\x11\x22\x33\x44";
        let mut encoded = [0u8; 16];
        let mut decoded = [0u8; 16];

        let encoded_len = cobs_encode(data, &mut encoded).unwrap();
        assert_eq!(&encoded[..encoded_len], b"\x05\x11\x22\x33\x44\x00");

        let decoded_len = cobs_decode(&encoded[..encoded_len-1], &mut decoded).unwrap();
        assert_eq!(decoded_len, data.len());
        assert_eq!(&decoded[..decoded_len], data);
    }

    #[test]
    fn test_encode_decode_all_zeros() {
        let data = b"\x00\x00\x00";
        let mut encoded = [0u8; 16];
        let mut decoded = [0u8; 16];

        let encoded_len = cobs_encode(data, &mut encoded).unwrap();
        assert_eq!(&encoded[..encoded_len], b"\x01\x01\x01\x01\x00");

        let decoded_len = cobs_decode(&encoded[..encoded_len-1], &mut decoded).unwrap();
        assert_eq!(decoded_len, data.len());
        assert_eq!(&decoded[..decoded_len], data);
    }
}
