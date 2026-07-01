use oxide_math::Table3D;

pub struct MathEngine {
    ve_table: Table3D<16, 16>,
    spark_table: Table3D<16, 16>,
    injector_config: InjectorConfig,
    ignition_config: IgnitionConfig,
}

pub struct InjectorConfig {
    pub flow_rate_cc_min: f32,
    pub dead_time_ms: f32,
}

pub struct IgnitionConfig {
    pub dwell_time_ms: f32,
}

impl MathEngine {
    pub fn new(
        ve_table: Table3D<16, 16>,
        spark_table: Table3D<16, 16>,
        injector_config: InjectorConfig,
        ignition_config: IgnitionConfig,
    ) -> Self {
        Self {
            ve_table,
            spark_table,
            injector_config,
            ignition_config,
        }
    }

    pub fn calculate_fuel_mass(&self, rpm: u16, map: u16, iat: i16) -> f32 {
        let ve = self.ve_table.get(rpm as f32, map as f32);
        let iat_k = iat as f32 + 273.15;
        let displacement_cc = 2000.0 / 4.0; // 2L 4-cyl
        let r_specific = 287.0;

        let rho = map as f32 * 1000.0 / (r_specific * iat_k);
        let air_mass_g = (ve / 100.0 * displacement_cc / 1_000_000.0 * rho) * 1000.0;

        let target_afr = 14.7;
        air_mass_g / target_afr
    }

    pub fn calculate_fuel_pulse_width(&self, fuel_mass_g: f32, battery_voltage: f32) -> f32 {
        let fuel_density_g_cc = 0.74;
        let fuel_volume_cc = fuel_mass_g / fuel_density_g_cc;
        let pulse_width_s = (fuel_volume_cc / self.injector_config.flow_rate_cc_min) * 60.0;

        // Simplified dead time for now
        pulse_width_s * 1_000_000.0 + self.injector_config.dead_time_ms * 1000.0
    }

    pub fn calculate_spark_advance(&self, rpm: u16, map: u16, knock_retard: f32) -> f32 {
        let base_advance = self.spark_table.get(rpm as f32, map as f32);
        base_advance - knock_retard
    }
}
