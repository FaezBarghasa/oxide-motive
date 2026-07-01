use rumqttc::{AsyncClient, MqttOptions, QoS};
use std::time::Duration;
use tokio::time::sleep;
use oxide_protocol::McuToHost;
use heapless::spsc::Queue;

pub async fn run_mqtt_telemetry(mut rx: tokio::sync::mpsc::Receiver<McuToHost>) {
    let mut mqttoptions = MqttOptions::new("oxide-motive", "test.mosquitto.org", 1883);
    mqttoptions.set_keep_alive(Duration::from_secs(5));

    let (client, mut eventloop) = AsyncClient::new(mqttoptions, 10);
    let mut fallback_buffer: Queue<McuToHost, 600> = Queue::new(); // 10 minutes of data at 1Hz

    tokio::spawn(async move {
        loop {
            match eventloop.poll().await {
                Ok(_) => {}
                Err(e) => {
                    println!("MQTT Error: {:?}", e);
                    sleep(Duration::from_secs(1)).await;
                }
            }
        }
    });

    loop {
        if let Some(telemetry) = rx.recv().await {
            let payload = postcard::to_stdvec(&telemetry).unwrap();
            match client.publish("oxide-motive/test/telemetry", QoS::AtMostOnce, false, payload).await {
                Ok(_) => {
                    // Flush fallback buffer if any
                    while let Some(fallback_telemetry) = fallback_buffer.dequeue() {
                        let fallback_payload = postcard::to_stdvec(&fallback_telemetry).unwrap();
                        client.publish("oxide-motive/test/telemetry", QoS::AtMostOnce, false, fallback_payload).await.ok();
                    }
                }
                Err(_) => {
                    // Network down, store in fallback buffer
                    fallback_buffer.enqueue(telemetry).ok();
                }
            }
        }
    }
}
