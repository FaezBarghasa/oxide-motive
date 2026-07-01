use crate::logger::LogPlayback;
use oxide_protocol::{HostToMcu, McuConnection, TelemetryBatch};
use oxide_math::Table3D;

pub struct AutoVeLearner {
    target_lambda: f32,
    learning_rate: f32,
    deadband: (f32, f32),
    min_conditions: LearningConditions,
}

pub struct LearningConditions {
    pub rpm_stable_threshold: u16,
    pub tps_stable_threshold: f32,
    pub min_coolant_temp: f32,
}

pub struct TableUpdate {
    pub table_id: u8,
    pub x_idx: u8,
    pub y_idx: u8,
    pub value: f32,
}

impl AutoVeLearner {
    pub fn new() -> Self {
        Self {
            target_lambda: 1.0,
            learning_rate: 0.1,
            deadband: (0.98, 1.02),
            min_conditions: LearningConditions {
                rpm_stable_threshold: 100,
                tps_stable_threshold: 1.0,
                min_coolant_temp: 80.0,
            },
        }
    }

    pub fn analyze_log(&self, log: &LogPlayback) -> Vec<TableUpdate> {
        let mut updates = Vec::new();
        let mut last_rpm = 0;
        let mut last_tps = 0.0;

        for entry in log {
            let ect = entry.sensors.iter().find(|s| s.id == 4).map_or(0.0, |s| s.physical_value);
            let tps = entry.sensors.iter().find(|s| s.id == 2).map_or(0.0, |s| s.physical_value);
            let lambda = entry.sensors.iter().find(|s| s.id == 6).map_or(1.0, |s| s.physical_value);

            if ect > self.min_conditions.min_coolant_temp &&
               (entry.rpm as i16 - last_rpm as i16).abs() < self.min_conditions.rpm_stable_threshold as i16 &&
               (tps - last_tps).abs() < self.min_conditions.tps_stable_threshold {

                let error = lambda / self.target_lambda;
                if error < self.deadband.0 || error > self.deadband.1 {
                    let correction = 1.0 + (error - 1.0) * self.learning_rate;
                    // In a real scenario, we'd get the current VE value and apply the correction
                    let new_ve = 50.0 * correction;
                    updates.push(TableUpdate {
                        table_id: 0,
                        x_idx: (entry.rpm / 500) as u8,
                        y_idx: (entry.sensors.iter().find(|s| s.id == 1).map_or(0, |s| s.raw_value) / 10) as u8,
                        value: new_ve,
                    });
                }
            }

            last_rpm = entry.rpm;
            last_tps = tps;
        }
        updates
    }

    pub async fn apply_updates(&self, updates: &[TableUpdate], connection: &mut McuConnection) -> Result<(), ()> {
        for update in updates {
            let msg = HostToMcu::TableUpdate {
                table_id: update.table_id,
                x_idx: update.x_idx,
                y_idx: update.y_idx,
                value: update.value,
            };
            let mut buf = [0u8; 64];
            let len = oxide_protocol::framing::encode_frame(&msg, &mut buf, 0).unwrap();
            connection.stream.write_all(&buf[..len]).await.map_err(|_| ())?;
        }
        Ok(())
    }
}
