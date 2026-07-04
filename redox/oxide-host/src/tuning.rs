use tokio::time::{timeout, Duration};
use oxide_protocol::{TuningCommand, postcard, framing};
use oxide_hal::transport::Transport;
use embedded_io_async::{Read, Write};

pub enum CommError {
    TuningFailed,
    Timeout,
}

pub struct TuningManager<T: Transport> {
    transport: T,
    cache: [[f32; 16]; 16], // Assuming 16x16 tables
}

impl<T: Transport> TuningManager<T> {
    pub fn new(transport: T) -> Self {
        Self {
            transport,
            cache: [[0.0; 16]; 16],
        }
    }

    pub async fn update_map_cell(
        &mut self,
        table_id: u8,
        x: u8,
        y: u8,
        value: f32,
    ) -> Result<(), CommError> {
        let command = TuningCommand { table_id, x, y, value };
        let mut buf = [0u8; 128];
        let serialized = postcard::to_slice(&command, &mut buf).unwrap();
        let mut encoded = [0u8; 128];
        let encoded_len = framing::cobs_encode(serialized, &mut encoded).unwrap();

        self.transport.write_all(&encoded[..encoded_len]).await.map_err(|_| CommError::TuningFailed)?;

        let mut response_buf = [0u8; 1];
        match timeout(Duration::from_millis(50), self.transport.read(&mut response_buf)).await {
            Ok(Ok(1)) if response_buf[0] == 1 => { // 1 = ACK
                self.cache[x as usize][y as usize] = value;
                Ok(())
            }
            _ => {
                // Rollback is implicit as the cache is not updated
                Err(CommError::TuningFailed)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io;

    struct MockTransport {
        drop_packets: bool,
    }

    impl Read for MockTransport {
        async fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
            if self.drop_packets {
                // Simulate a timeout by never returning
                futures::future::pending().await
            } else {
                buf[0] = 1; // ACK
                Ok(1)
            }
        }
    }

    impl Write for MockTransport {
        async fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
            Ok(buf.len())
        }
        async fn flush(&mut self) -> io::Result<()> {
            Ok(())
        }
    }

    impl Transport for MockTransport {}

    #[tokio::test]
    async fn test_tuning_timeout() {
        let transport = MockTransport { drop_packets: true };
        let mut manager = TuningManager::new(transport);
        let result = manager.update_map_cell(0, 0, 0, 1.0).await;
        assert!(matches!(result, Err(CommError::TuningFailed)));
    }
}
