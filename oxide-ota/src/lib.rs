#![no_std]

use embassy_executor::task;
use embassy_sync::channel::Channel;
use mqtt_async_embedded::{MqttClient, QoS};
use heapless::Vec;
use ed25519_dalek::{Signature, Verifier, VerifyingKey};
use esp_idf_svc::ota::EspOta;
use postcard::from_bytes;
use serde::Deserialize;
use sha2::{Digest, Sha256};

#[derive(Deserialize)]
struct OtaManifest {
    version: u32,
    size: u32,
    signature: [u8; 64],
}

#[task]
pub async fn ota_task(
    mut client: MqttClient<'static, impl embedded_io_async::Read, impl embedded_io_async::Write>,
    manifest_channel: &'static Channel<embassy_sync::blocking_mutex::raw::NoopRawMutex, OtaManifest, 1>,
    chunk_channel: &'static Channel<embassy_sync::blocking_mutex::raw::NoopRawMutex, Vec<u8, 1024>, 1>,
) {
    let vin = "VIN"; // Replace with actual VIN
    let manifest_topic = format!("oxide-tech/vehicle/{}/ota/manifest", vin);
    let chunk_topic = format!("oxide-tech/vehicle/{}/ota/chunk", vin);

    // This is a simplified subscription. A real implementation would handle this in the MqttClient
    // client.subscribe(&manifest_topic, QoS::AtLeastOnce).await;
    // client.subscribe(&chunk_topic, QoS::AtLeastOnce).await;

    loop {
        let manifest = manifest_channel.receive().await;

        let mut ota = EspOta::new().unwrap();
        let mut update = ota.initiate_update().unwrap();

        let mut received_size = 0;
        let mut hasher = Sha256::new();
        while received_size < manifest.size {
            let chunk = chunk_channel.receive().await;
            update.write(&chunk).unwrap();
            hasher.update(&chunk);
            received_size += chunk.len() as u32;
        }

        let hash_result = hasher.finalize();

        // This is a placeholder for the public key
        let public_key_bytes: [u8; 32] = [0; 32];
        let public_key = VerifyingKey::from_bytes(&public_key_bytes).unwrap();
        let signature = Signature::from_bytes(&manifest.signature).unwrap();

        // The verification checks the Ed25519 signature against the SHA256 digest of the OTA image.
        let is_valid = public_key.verify(&hash_result, &signature).is_ok();

        if is_valid {
            update.finish().unwrap();
            esp_idf_svc::sys::esp_restart();
        } else {
            update.abort().unwrap();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ota_logic() {
        // Testing OTA logic requires a real device or a sophisticated mock.
    }
}