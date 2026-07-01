use std::collections::HashMap;
use oxide_protocol::SensorData;

pub struct SensorConfig {
    pub physical_channel_id: u8,
    pub name: String,
    pub unit: String,
    pub transfer_function: TransferFunction,
    pub min_value: f32,
    pub max_value: f32,
    pub plausibility_check: PlausibilityCheck,
}

pub enum TransferFunction {
    Linear { slope: f32, offset: f32 },
    Polynomial { coeffs: Vec<f32> },
    LookupTable { points: Vec<(f32, f32)> },
}

impl TransferFunction {
    pub fn apply(&self, raw_value: f32) -> f32 {
        match self {
            TransferFunction::Linear { slope, offset } => raw_value * slope + offset,
            TransferFunction::Polynomial { coeffs } => {
                let mut y = 0.0;
                for (i, c) in coeffs.iter().enumerate() {
                    y += c * raw_value.powi(i as i32);
                }
                y
            }
            TransferFunction::LookupTable { points } => {
                // Simple 1D interpolation
                if points.is_empty() {
                    return raw_value;
                }
                if raw_value <= points[0].0 {
                    return points[0].1;
                }
                if raw_value >= points.last().unwrap().0 {
                    return points.last().unwrap().1;
                }
                for i in 0..points.len() - 1 {
                    if raw_value >= points[i].0 && raw_value <= points[i+1].0 {
                        let t = (raw_value - points[i].0) / (points[i+1].0 - points[i].0);
                        return points[i].1 + t * (points[i+1].1 - points[i].1);
                    }
                }
                raw_value
            }
        }
    }
}

pub enum PlausibilityCheck {
    None,
    Range { min: f32, max: f32 },
    Rate { max_change: f32 },
}

pub struct SensorManager {
    sensors: HashMap<u8, SensorConfig>,
    last_values: HashMap<u8, f32>,
}

impl SensorManager {
    pub fn new() -> Self {
        Self {
            sensors: HashMap::new(),
            last_values: HashMap::new(),
        }
    }

    pub fn add_sensor(&mut self, config: SensorConfig) {
        self.sensors.insert(config.physical_channel_id, config);
    }

    pub fn process_raw_data(&mut self, raw_data: &[SensorData]) -> Vec<SensorData> {
        let mut physical_data = Vec::new();
        for raw in raw_data {
            if let Some(config) = self.sensors.get(&raw.id) {
                let mut physical_value = config.transfer_function.apply(raw.raw_value as f32);
                let mut status = 0;

                if let PlausibilityCheck::Range { min, max } = config.plausibility_check {
                    if physical_value < min || physical_value > max {
                        status = 1; // Out of range
                    }
                }
                if let PlausibilityCheck::Rate { max_change } = config.plausibility_check {
                    if let Some(last) = self.last_values.get(&raw.id) {
                        if (physical_value - last).abs() > max_change {
                            status = 2; // Rate of change exceeded
                        }
                    }
                }

                self.last_values.insert(raw.id, physical_value);
                physical_data.push(SensorData {
                    id: raw.id,
                    raw_value: raw.raw_value,
                    physical_value,
                    status,
                });
            }
        }
        physical_data
    }
}
