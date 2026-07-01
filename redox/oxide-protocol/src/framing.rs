use postcard::to_slice;
use serde::Serialize;

pub fn encode_frame<T: Serialize>(msg: &T, buf: &mut [u8]) -> Result<usize, postcard::Error> {
    let serialized = to_slice(msg, buf)?;
    let encoded_len = cobs::encode(serialized, &mut buf[1..]);
    buf[0] = 0x00;
    Ok(encoded_len + 1)
}

use postcard::from_bytes;
use serde::de::DeserializeOwned;

pub fn decode_frame<T: DeserializeOwned>(buf: &[u8]) -> Result<T, postcard::Error> {
    let mut decoded_buf = [0u8; 256];
    let decoded_len = cobs::decode(buf, &mut decoded_buf).unwrap();
    from_bytes(&decoded_buf[..decoded_len])
}
