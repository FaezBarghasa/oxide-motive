#![no_std]

use embassy_executor::task;
use embassy_sync::channel::Channel;
use embedded_io_async::Read;
use cobs::decode_in_place;
use postcard::from_bytes;
use serde_json_core::to_string;
use oxide_core::VehicleTelemetry;
use mqtt_async_embedded::{MqttClient, QoS};

const CHANNEL_CAPACITY: usize = 16;
type TelemetryChannel = Channel<embassy_sync::blocking_mutex::raw::NoopRawMutex, VehicleTelemetry, CHANNEL_CAPACITY>;

#[task]
pub async fn uart_reader_task(
    mut reader: impl Read,
    channel: &'static TelemetryChannel,
) {
    let mut buf = [0u8; 256];
    loop {
        match reader.read_exact(&mut buf).await {
            Ok(_) => {
                let decoded_len = decode_in_place(&mut buf).unwrap_or(0);
                if decoded_len > 0 {
                    if let Ok(telemetry) = from_bytes::<VehicleTelemetry>(&buf[..decoded_len]) {
                        channel.send(telemetry).await;
                    }
                }
            }
            Err(_) => {
                // Handle read error
            }
        }
    }
}

#[task]
pub async fn mqtt_publisher_task(
    mut client: MqttClient<'static, impl Read, impl embedded_io_async::Write>,
    channel: &'static TelemetryChannel,
) {
    loop {
        let telemetry = channel.receive().await;
        if let Ok(json_payload) = to_string::<_, 256>(&telemetry) {
            let topic = "oxide-tech/vehicle/VIN/telemetry"; // Replace VIN
            let _ = client.publish(topic, json_payload.as_bytes(), QoS::ExactlyOnce).await;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use embassy_sync::blocking_mutex::raw::NoopRawMutex;
    use embassy_executor::Executor;
    use static_cell::StaticCell;

    struct MockReader {
        data: &'static [u8],
        pos: usize,
    }

    impl Read for MockReader {
        async fn read(&mut self, buf: &mut [u8]) -> Result<usize, embedded_io_async::ReadExactError<()>> {
            let bytes_to_read = core::cmp::min(buf.len(), self.data.len() - self.pos);
            buf[..bytes_to_read].copy_from_slice(&self.data[self.pos..self.pos + bytes_to_read]);
            self.pos += bytes_to_read;
            Ok(bytes_to_read)
        }
    }

    #[test]
    fn test_bridge() {
        static EXECUTOR: StaticCell<Executor> = StaticCell::new();
        let executor = EXECUTOR.init(Executor::new());

        static CHANNEL: StaticCell<TelemetryChannel> = StaticCell::new();
        let channel = CHANNEL.init(Channel::new());

        // This requires a more complex setup to run async tasks in a test environment.
        // The basic structure is here, but a full integration test is non-trivial.
    }
}