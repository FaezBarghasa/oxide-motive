use std::collections::{HashMap, VecDeque};
use oxide_math::Table3D;

pub enum VirtualSensor {
    SpeedDensity {
        ve_table: Table3D<16, 16>,
        displacement: f32,
        r_gas_constant: f32,
    },
    EstimatedIntakeTemp {
        k1: f32,
        k2: f32,
    },
}

impl VirtualSensor {
    pub fn calculate(&self, inputs: &HashMap<String, f32>) -> f32 {
        match self {
            VirtualSensor::SpeedDensity { ve_table, displacement, r_gas_constant } => {
                let map = inputs["map"];
                let rpm = inputs["rpm"];
                let iat = inputs["iat"];
                let ve = ve_table.interpolate(rpm, map);
                (map * ve * *displacement * rpm) / (*r_gas_constant * (iat + 273.15))
            }
            VirtualSensor::EstimatedIntakeTemp { k1, k2 } => {
                let ambient = inputs["ambient_temp"];
                let map = inputs["map"];
                let rpm = inputs["rpm"];
                ambient + (map * *k1) + (rpm * *k2)
            }
        }
    }
}

pub struct MathBlockEngine {
    sensors: HashMap<String, VirtualSensor>,
    dependencies: HashMap<String, Vec<String>>,
}

impl MathBlockEngine {
    pub fn new() -> Self {
        Self {
            sensors: HashMap::new(),
            dependencies: HashMap::new(),
        }
    }

    pub fn resolve_and_calculate(&self, inputs: &mut HashMap<String, f32>) {
        let mut queue: VecDeque<_> = self.sensors.keys().cloned().collect();
        let mut calculated = HashSet::new();

        while let Some(sensor_name) = queue.pop_front() {
            if calculated.contains(&sensor_name) {
                continue;
            }

            if let Some(deps) = self.dependencies.get(&sensor_name) {
                if deps.iter().all(|dep| calculated.contains(dep) || inputs.contains_key(dep)) {
                    let sensor = &self.sensors[&sensor_name];
                    let value = sensor.calculate(inputs);
                    inputs.insert(sensor_name.clone(), value);
                    calculated.insert(sensor_name);
                } else {
                    queue.push_back(sensor_name);
                }
            }
        }
    }
}
use std::collections::HashSet;
