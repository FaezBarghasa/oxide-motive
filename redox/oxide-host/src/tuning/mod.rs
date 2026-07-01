use oxide_math::Table3D;
use std::time::{Duration, Instant};

pub struct AutoVeLearner<'a> {
    ve_table: &'a mut Table3D<16, 16>,
    learning_rate: f32,
    deadband: (f32, f32),
}

impl<'a> AutoVeLearner<'a> {
    pub fn new(ve_table: &'a mut Table3D<16, 16>) -> Self {
        Self {
            ve_table,
            learning_rate: 0.05,
            deadband: (0.98, 1.02),
        }
    }

    pub fn update(
        &mut self,
        target_lambda: f32,
        actual_lambda: f32,
        rpm: f32,
        map: f32,
        rpm_stable: bool,
        tps_stable: bool,
        coolant_temp_ok: bool,
    ) {
        if !rpm_stable || !tps_stable || !coolant_temp_ok {
            return;
        }

        let error = actual_lambda / target_lambda;
        if error < self.deadband.0 || error > self.deadband.1 {
            let current_ve = self.ve_table.interpolate(rpm, map);
            let new_ve = current_ve * error;
            let final_ve = current_ve + (new_ve - current_ve) * self.learning_rate;
            // This is a simplified update. A real implementation would need to find the indices and update the table data directly.
            // self.ve_table.update_cell(rpm, map, final_ve);
        }
    }
}

pub struct KnockController {
    pub global_timing_retard: f32,
    pub cylinder_timing_retards: [f32; 4],
    recovery_timers: [Instant; 4],
    knock_step_deg: f32,
    recovery_step_deg: f32,
    max_retard: f32,
}

impl KnockController {
    pub fn new() -> Self {
        Self {
            global_timing_retard: 0.0,
            cylinder_timing_retards: [0.0; 4],
            recovery_timers: [Instant::now(); 4],
            knock_step_deg: 1.5,
            recovery_step_deg: 0.1,
            max_retard: 10.0,
        }
    }

    pub fn on_knock_event(&mut self, cylinder_id: usize, intensity: f32) {
        if cylinder_id < 4 {
            self.cylinder_timing_retards[cylinder_id] += intensity * self.knock_step_deg;
            self.cylinder_timing_retards[cylinder_id] = self.cylinder_timing_retards[cylinder_id].min(self.max_retard);
            self.recovery_timers[cylinder_id] = Instant::now();
        }
    }

    pub fn update(&mut self) {
        for i in 0..4 {
            if self.recovery_timers[i].elapsed() > Duration::from_millis(100) {
                self.cylinder_timing_retards[i] -= self.recovery_step_deg;
                self.cylinder_timing_retards[i] = self.cylinder_timing_retards[i].max(0.0);
            }
        }
    }
}

pub struct FlexFuelBlender<'a> {
    e0_fuel_table: &'a Table3D<16, 16>,
    e85_fuel_table: &'a Table3D<16, 16>,
    e0_spark_table: &'a Table3D<16, 16>,
    e85_spark_table: &'a Table3D<16, 16>,
}

impl<'a> FlexFuelBlender<'a> {
    pub fn new(
        e0_fuel_table: &'a Table3D<16, 16>,
        e85_fuel_table: &'a Table3D<16, 16>,
        e0_spark_table: &'a Table3D<16, 16>,
        e85_spark_table: &'a Table3D<16, 16>,
    ) -> Self {
        Self {
            e0_fuel_table,
            e85_fuel_table,
            e0_spark_table,
            e85_spark_table,
        }
    }

    pub fn get_blended_fuel(&self, rpm: f32, map: f32, ethanol_percent: f32) -> f32 {
        let e0_fuel = self.e0_fuel_table.interpolate(rpm, map);
        let e85_fuel = self.e85_fuel_table.interpolate(rpm, map);
        let ratio = ethanol_percent / 85.0;
        e0_fuel + (e85_fuel - e0_fuel) * ratio
    }

    pub fn get_blended_spark(&self, rpm: f32, map: f32, ethanol_percent: f32) -> f32 {
        let e0_spark = self.e0_spark_table.interpolate(rpm, map);
        let e85_spark = self.e85_spark_table.interpolate(rpm, map);
        let ratio = ethanol_percent / 85.0;
        e0_spark + (e85_spark - e0_spark) * ratio
    }

    pub fn get_cold_start_enrichment(&self, temp_celsius: f32, ethanol_percent: f32) -> f32 {
        if temp_celsius < 20.0 && ethanol_percent > 50.0 {
            (20.0 - temp_celsius) * 0.05 * (ethanol_percent / 85.0)
        } else {
            0.0
        }
    }
}
