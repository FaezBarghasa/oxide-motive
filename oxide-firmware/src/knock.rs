pub struct KnockController {
    global_timing_retard: f32,
    cylinder_timing_retards: [f32; 4],
    knock_step_deg: f32,
    recovery_step_deg: f32,
    max_retard_deg: f32,
    last_knock_time: [u64; 4],
}

impl KnockController {
    pub fn new() -> Self {
        Self {
            global_timing_retard: 0.0,
            cylinder_timing_retards: [0.0; 4],
            knock_step_deg: 1.0,
            recovery_step_deg: 0.1,
            max_retard_deg: 15.0,
            last_knock_time: [0; 4],
        }
    }

    pub fn process_knock_event(&mut self, cylinder_id: u8, intensity: f32, now: u64) {
        if cylinder_id < 4 {
            let retard = self.cylinder_timing_retards[cylinder_id as usize] + intensity * self.knock_step_deg;
            self.cylinder_timing_retards[cylinder_id as usize] = retard.min(self.max_retard_deg);
            self.last_knock_time[cylinder_id as usize] = now;
        }
    }

    pub fn recover(&mut self, now: u64) {
        for i in 0..4 {
            if now - self.last_knock_time[i] > 500_000_000 { // 500ms
                let advance = self.cylinder_timing_retards[i] - self.recovery_step_deg;
                self.cylinder_timing_retards[i] = advance.max(0.0);
            }
        }
    }

    pub fn get_total_retard(&self, cylinder_id: u8) -> f32 {
        if cylinder_id < 4 {
            self.global_timing_retard + self.cylinder_timing_retards[cylinder_id as usize]
        } else {
            self.global_timing_retard
        }
    }
}
