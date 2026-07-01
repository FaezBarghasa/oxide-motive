use esp_idf_hal::prelude::*;
use esp_idf_svc::{
    eventloop::EspSystemEventLoop,
    mqtt::client::{EspMqttClient, MqttClientConfiguration},
    nvs::EspNvs,
    wifi::{AuthMethod, ClientConfiguration, Configuration, EspWifi},
};
use log::{info, error};
use std::{thread, time::Duration};

const SSID: &str = env!("WIFI_SSID");
const PASS: &str = env!("WIFI_PASS");
const MQTT_URL: &str = env!("MQTT_URL");

fn main() -> anyhow::Result<()> {
    esp_idf_svc::sys::link_patches();
    esp_idf_svc::log::EspLogger::initialize_default();

    let peripherals = Peripherals::take().unwrap();
    let sys_loop = EspSystemEventLoop::take()?;
    let nvs = EspNvs::new(peripherals.nvs)?;

    let mut wifi = EspWifi::new(peripherals.modem, sys_loop.clone(), Some(nvs))?;

    info!("Connecting to Wi-Fi...");
    wifi.set_configuration(&Configuration::Client(ClientConfiguration {
        ssid: SSID.into(),
        password: PASS.into(),
        auth_method: AuthMethod::WPA2Personal,
        ..Default::default()
    }))?;

    wifi.start()?;
    wifi.connect()?;
    wifi.wait_netif_up()?;

    info!("Wi-Fi connected!");

    let mqtt_config = MqttClientConfiguration::default();
    let mut mqtt_client = EspMqttClient::new(MQTT_URL, &mqtt_config)?;

    mqtt_client.subscribe("oxide_motive/commands", esp_idf_svc::mqtt::client::QoS::AtMostOnce)?;

    let mut telemetry_counter = 0;
    loop {
        let payload = format!("{{\"rpm\": {}, \"timestamp\": {}}}", telemetry_counter * 100, telemetry_counter);
        if let Err(e) = mqtt_client.publish(
            "oxide_motive/telemetry",
            esp_idf_svc::mqtt::client::QoS::AtMostOnce,
            false,
            payload.as_bytes(),
        ) {
            error!("Failed to publish MQTT message: {:?}", e);
        } else {
            info!("Published MQTT message: {}", payload);
        }

        telemetry_counter += 1;
        thread::sleep(Duration::from_secs(5));
    }
}
