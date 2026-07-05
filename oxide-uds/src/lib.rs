#![no_std]

use embassy_executor::task;
use embassy_sync::channel::Channel;
use embassy_time::{Duration, with_timeout};
use mqtt_async_embedded::{MqttClient, QoS};
use oxide_core::{UdsRequest, UdsResponse};
use postcard::{to_vec, from_bytes};
use heapless::Vec;

#[task]
pub async fn uds_task(
    mut client: MqttClient<'static, impl embedded_io_async::Read, impl embedded_io_async::Write>,
    request_channel: &'static Channel<embassy_sync::blocking_mutex::raw::NoopRawMutex, UdsRequest, 1>,
) {
    let vin = "VIN"; // Replace with actual VIN
    let response_topic = format!("oxide-tech/vehicle/{}/uds/response", vin);

    defmt::info!("UDS: task started for VIN={}", vin);

    loop {
        let request = request_channel.receive().await;
        defmt::info!("UDS: received request: {:?}", request);

        let response = match with_timeout(Duration::from_secs(2), handle_uds_request(request)).await {
            Ok(response) => {
                defmt::info!("UDS: request processed successfully");
                response
            }
            Err(_) => {
                defmt::warn!("UDS: request timed out or failed");
                UdsResponse::NegativeResponse(0, 0x22) // ConditionsNotCorrect
            }
        };

        let mut buf = [0u8; 128];
        if let Ok(serialized) = to_vec::<_, Vec<u8, 128>>(&response, &mut buf) {
            defmt::info!("UDS: publishing response, size={}", serialized.len());
            let _ = client.publish(&response_topic, &serialized, QoS::AtLeastOnce).await;
        } else {
            defmt::error!("UDS: serialization of response failed");
        }
    }
}

async fn handle_uds_request(request: UdsRequest) -> UdsResponse {
    // In a real implementation, this would interact with the vehicle HAL.
    // This is a mock implementation.
    match request {
        UdsRequest::DiagnosticSessionControl(session) => {
            UdsResponse::DiagnosticSessionControl(session)
        }
        UdsRequest::ReadDataByIdentifier(did) => {
            let mut data = Vec::new();
            data.extend_from_slice(&[0xDE, 0xAD, 0xBE, 0xEF]).unwrap();
            UdsResponse::ReadDataByIdentifier(did, data)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_uds_handler() {
        // This requires an async test runner.
    }
}