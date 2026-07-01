
pub struct AirDensityCompensation {
    reference_temp_k: f32,
    reference_pressure_kpa: f32,
}

impl AirDensityCompensation {
    pub fn new() -> Self {
        Self {
            reference_temp_k: 293.15, // 20°C
            reference_pressure_kpa: 101.3,
        }
    }

    pub fn calculate_density_ratio(&self, iat_kelvin: f32, map_kpa: f32) -> f32 {
        let current_density = map_kpa / (287.05 * iat_kelvin);
        let reference_density = self.reference_pressure_kpa / (287.05 * self.reference_temp_k);
        current_density / reference_density
    }

    pub fn ignition_correction(&self, iat_celsius: f32) -> f32 {
        // Simplified IAT-based timing retard
        if iat_celsius > 40.0 {
            (iat_celsius - 40.0) * -0.1 // Retard 0.1 degrees for every degree over 40
        } else {
            0.0
        }
    }
}
