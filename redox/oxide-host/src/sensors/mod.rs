use std::collections::HashMap;
use oxide_math::Table3D;

pub enum TransferFunction {
    Linear(f32, f32),
    Polynomial(Vec<f32, 8>),
    LookupTable(Vec<(f32, f32), 16>),
}

impl TransferFunction {
    pub fn apply(&self, raw_value: f32) -> f32 {
        match self {
            TransferFunction::Linear(slope, offset) => raw_value * slope + offset,
            TransferFunction::Polynomial(coeffs) => {
                let mut result = 0.0;
                for (i, coeff) in coeffs.iter().enumerate() {
                    result += coeff * raw_value.powi(i as i32);
                }
                result
            }
            TransferFunction::LookupTable(table) => {
                // Simplified 1D interpolation
                let (x1, y1) = table.iter().find(|(x, _)| *x >= raw_value).unwrap_or(&table[table.len() - 1]);
                let (x2, y2) = table.iter().find(|(x, _)| *x < raw_value).unwrap_or(&table[0]);
                y1 + (y2 - y1) * (raw_value - x1) / (x2 - x1)
            }
        }
    }
}

pub struct SensorConfig {
    pub physical_channel_id: u8,
    pub transfer_function: TransferFunction,
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

    pub fn process_raw_values(&mut self, raw_values: &[(u8, u16)]) -> HashMap<u8, f32> {
        let mut physical_values = HashMap::new();
        for (id, raw_value) in raw_values {
            if let Some(config) = self.sensors.get(id) {
                let physical_value = config.transfer_function.apply(*raw_value as f32);
                if let Some(last_value) = self.last_values.get(id) {
                    if (physical_value - last_value).abs() / last_value > 0.2 {
                        // Implausible value, use last known good value
                        physical_values.insert(*id, *last_value);
                        continue;
                    }
                }
                physical_values.insert(*id, physical_value);
                self.last_values.insert(*id, physical_value);
            }
        }
        physical_values
    }
}
