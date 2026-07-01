use rumqttc::{AsyncClient, MqttOptions, QoS};
use tokio::sync::broadcast;
use crate::TelemetrySubscriber;
use oxide_protocol::{TelemetryBatch, DtcEvent};
use ringbuffer::{RingBuffer, AllocRingBuffer};

pub struct MqttStreamer {
    client: AsyncClient,
    topic_prefix: String,
    ring_buffer: AllocRingBuffer<TelemetryBatch>,
}

impl MqttStreamer {
    pub async fn new(broker: &str, port: u16, vin: &str) -> Self {
        let mut mqttoptions = MqttOptions::new(vin, broker, port);
        mqttoptions.set_keep_alive(std::time::Duration::from_secs(5));

        let (client, mut eventloop) = AsyncClient::new(mqttoptions, 10);
        tokio::spawn(async move {
            loop {
                let _ = eventloop.poll().await;
            }
        });

        Self {
            client,
            topic_prefix: format!("oxide-motive/{}/telemetry", vin),
            ring_buffer: AllocRingBuffer::new(1000),
        }
    }

    pub async fn run(&mut self, mut telemetry_rx: broadcast::Receiver<TelemetryBatch>) {
        loop {
            match telemetry_rx.recv().await {
                Ok(telemetry) => {
                    if self.client.publish(&self.topic_prefix, QoS::AtMostOnce, false, serde_json::to_vec(&telemetry).unwrap()).await.is_err() {
                        self.ring_buffer.push(telemetry);
                    } else {
                        while let Some(buffered_telemetry) = self.ring_buffer.dequeue() {
                            if self.client.publish(&self.topic_prefix, QoS::AtMostOnce, false, serde_json::to_vec(&buffered_telemetry).unwrap()).await.is_err() {
                                self.ring_buffer.push(buffered_telemetry);
                                break;
                            }
                        }
                    }
                }
                Err(_) => break,
            }
        }
    }
}
