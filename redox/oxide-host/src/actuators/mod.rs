use heapless::Vec;

pub struct InjectorConfig {
    pub latency_table: Vec<(f32, f32), 8>, // (Voltage, Delay ms)
}

impl InjectorConfig {
    pub fn get_latency(&self, voltage: f32) -> f32 {
        // Simplified 1D interpolation
        let (v1, d1) = self.latency_table.iter().find(|(v, _)| *v >= voltage).unwrap_or(&self.latency_table[self.latency_table.len() - 1]);
        let (v2, d2) = self.latency_table.iter().find(|(v, _)| *v < voltage).unwrap_or(&self.latency_table[0]);
        d1 + (d2 - d1) * (voltage - v1) / (v2 - v1)
    }
}

pub struct PwmConfig {
    pub initial_duty: f32,
    pub hold_duty: f32,
    pub frequency: u32,
}

pub fn calculate_compensated_pulse_width(base_pw: f32, voltage: f32, config: &InjectorConfig) -> f32 {
    base_pw + config.get_latency(voltage)
}
