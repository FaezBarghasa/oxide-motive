#[derive(Debug, PartialEq)]
pub enum FramingError {
    BufferTooSmall,
    InvalidData,
}

pub fn cobs_encode(input: &[u8], output: &mut [u8]) -> Result<usize, FramingError> {
    if output.len() < input.len() + 2 {
        return Err(FramingError::BufferTooSmall);
    }

    let mut write_idx = 1;
    let mut code_idx = 0;
    let mut code = 1;

    for &byte in input {
        if byte == 0 {
            output[code_idx] = code;
            code = 1;
            code_idx = write_idx;
            write_idx += 1;
        } else {
            output[write_idx] = byte;
            write_idx += 1;
            code += 1;
            if code == 0xFF {
                output[code_idx] = code;
                code = 1;
                code_idx = write_idx;
                write_idx += 1;
            }
        }
    }

    output[code_idx] = code;
    output[write_idx] = 0;
    Ok(write_idx + 1)
}

pub fn cobs_decode(input: &[u8], output: &mut [u8]) -> Result<usize, FramingError> {
    if input.is_empty() || input.last() != Some(&0) {
        return Err(FramingError::InvalidData);
    }

    let mut read_idx = 0;
    let mut write_idx = 0;

    while read_idx < input.len() - 1 {
        let code = input[read_idx];
        read_idx += 1;

        if read_idx + code as usize - 1 > input.len() - 1 {
            return Err(FramingError::InvalidData);
        }

        if write_idx + code as usize - 1 > output.len() {
            return Err(FramingError::BufferTooSmall);
        }

        for i in 0..(code - 1) {
            output[write_idx] = input[read_idx];
            read_idx += 1;
            write_idx += 1;
        }

        if code < 0xFF && read_idx < input.len() - 1 {
            if write_idx >= output.len() {
                return Err(FramingError::BufferTooSmall);
            }
            output[write_idx] = 0;
            write_idx += 1;
        }
    }

    Ok(write_idx)
}
