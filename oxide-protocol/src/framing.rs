//! Zero-Allocation COBS Framing for Oxide-Motive
#![no_std]

#[derive(Debug, PartialEq)]
pub enum FramingError {
    BufferTooSmall,
    InvalidFrame,
    ZeroByteInPayload,
}

/// Encodes a payload using COBS.
/// Output buffer must be at least `input.len() + (input.len() / 254) + 2` bytes.
pub fn cobs_encode(input: &[u8], output: &mut [u8]) -> Result<usize, FramingError> {
    if output.len() < input.len() + 2 {
        return Err(FramingError::BufferTooSmall);
    }

    let mut code_idx = 0;
    let mut code = 1u8;
    let mut out_idx = 1;

    output[0] = 0; // Placeholder for the first code byte

    for &byte in input {
        if byte == 0 {
            if out_idx >= output.len() { return Err(FramingError::BufferTooSmall); }
            output[code_idx] = code;
            code_idx = out_idx;
            out_idx += 1;
            code = 1;
        } else {
            if out_idx >= output.len() { return Err(FramingError::BufferTooSmall); }
            output[out_idx] = byte;
            out_idx += 1;
            code += 1;
            if code == 0xFF {
                output[code_idx] = 0xFF;
                code_idx = out_idx;
                if out_idx >= output.len() { return Err(FramingError::BufferTooSmall); }
                out_idx += 1;
                code = 1;
            }
        }
    }
    output[code_idx] = code;
    Ok(out_idx)
}

/// Decodes a COBS frame.
/// Output buffer must be at least `input.len()` bytes.
pub fn cobs_decode(input: &[u8], output: &mut [u8]) -> Result<usize, FramingError> {
    if input.is_empty() || input[input.len() - 1] != 0 {
        return Err(FramingError::InvalidFrame); // Must be null-terminated
    }

    let mut in_idx = 0;
    let mut out_idx = 0;

    while in_idx < input.len() - 1 {
        let code = input[in_idx];
        if code == 0 {
            return Err(FramingError::ZeroByteInPayload);
        }
        in_idx += 1;

        for _ in 1..code {
            if in_idx >= input.len() - 1 {
                return Err(FramingError::InvalidFrame);
            }
            if out_idx >= output.len() {
                return Err(FramingError::BufferTooSmall);
            }
            output[out_idx] = input[in_idx];
            out_idx += 1;
            in_idx += 1;
        }

        if code < 0xFF && in_idx < input.len() - 1 {
            if out_idx >= output.len() {
                return Err(FramingError::BufferTooSmall);
            }
            output[out_idx] = 0;
            out_idx += 1;
        }
    }
    Ok(out_idx)
}
