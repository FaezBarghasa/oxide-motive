//! High-Performance Network Gateway for Oxide-Motive Host
use tokio::sync::mpsc;
use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct TelemetryFrame {
    pub timestamp_us: u64,
    pub rpm: f32,
    pub map_kpa: f32,
    pub tps_pct: f32,
    pub afr: f32,
    pub knock_retard: f32,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct AutotuneCycle {
    pub peaks: Vec<f32>,
    pub peak_times: Vec<f32>,
    pub is_active: bool,
}

pub struct NetworkGateway {
    mqtt_tx: mpsc::Sender<TelemetryFrame>,
    ws_tx: mpsc::Sender<AutotuneCycle>,
}

impl NetworkGateway {
    pub fn new(mqtt_tx: mpsc::Sender<TelemetryFrame>, ws_tx: mpsc::Sender<AutotuneCycle>) -> Self {
        Self { mqtt_tx, ws_tx }
    }

    pub async fn route_telemetry(&self, frame: TelemetryFrame) {
        // QoS 0 equivalent: Fire and forget for high-frequency data
        let _ = self.mqtt_tx.try_send(frame);
    }

    pub async fn route_autotune_cycle(&self, cycle: AutotuneCycle) {
        // QoS 1 equivalent: Await ack for critical tuning data
        let _ = self.ws_tx.send(cycle).await;
    }
}