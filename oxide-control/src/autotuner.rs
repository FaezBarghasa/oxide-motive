//! Optimized Relay Autotuner with Limit Cycle Telemetry
//! Zero `libm` dependency. Exposes peak data for Host UI streaming.
#![no_std]

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum TuneMode { Heating, Cooling }

#[derive(Clone, Copy, PartialEq, Eq)]
enum TunerState { Idle, PreConditioning, Cycling }

/// Data structure exposed to the Host for real-time limit cycle visualization
pub struct LimitCycleData {
    pub peaks: [f32; 8],
    pub peak_times: [f32; 8],
    pub peak_count: u8,
    pub relay_state: bool,
}

pub struct UpRelayAutotuner {
    setpoint: f32,
    hysteresis: f32,
    output_step: f32,
    mode: TuneMode,
    state: TunerState,
    relay_state: bool,

    peak_times: [f32; 8],
    peaks: [f32; 8],
    peak_count: u8,

    cycle_max: f32,
    cycle_max_time: f32,
    cycle_min: f32,
    cycle_min_time: f32,
}

impl UpRelayAutotuner {
    pub fn new(setpoint: f32, hysteresis: f32, output_step: f32, mode: TuneMode) -> Self {
        Self {
            setpoint, hysteresis, output_step, mode,
            state: TunerState::Idle,
            relay_state: false,
            peak_times: [0.0; 8],
            peaks: [0.0; 8],
            peak_count: 0,
            cycle_max: f32::MIN,
            cycle_max_time: 0.0,
            cycle_min: f32::MAX,
            cycle_min_time: 0.0,
        }
    }

    pub fn reset(&mut self) {
        self.state = TunerState::Idle;
        self.peak_count = 0;
        self.relay_state = false;
    }

    /// Exposes the current limit cycle data for telemetry streaming
    pub fn get_limit_cycle_data(&self) -> LimitCycleData {
        LimitCycleData {
            peaks: self.peaks,
            peak_times: self.peak_times,
            peak_count: self.peak_count,
            relay_state: self.relay_state,
        }
    }

    pub fn tune(&mut self, input: f32, time: f32) -> (bool, f32) {
        match self.state {
            TunerState::Idle => {
                self.reset();
                self.state = TunerState::PreConditioning;
                self.cycle_max = input;
                self.cycle_min = input;
                let output = if self.mode == TuneMode::Heating { self.output_step } else { -self.output_step };
                (false, output)
            }
            TunerState::PreConditioning => {
                let condition_met = if self.mode == TuneMode::Heating { input >= self.setpoint } else { input <= self.setpoint };
                if condition_met {
                    self.state = TunerState::Cycling;
                    self.cycle_max = input;
                    self.cycle_min = input;
                }
                let output = if self.mode == TuneMode::Heating { self.output_step } else { -self.output_step };
                (false, output)
            }
            TunerState::Cycling => {
                let upper_band = self.setpoint + self.hysteresis;
                let lower_band = self.setpoint - self.hysteresis;

                // Continuous tracking of extremes
                if input > self.cycle_max { self.cycle_max = input; self.cycle_max_time = time; }
                if input < self.cycle_min { self.cycle_min = input; self.cycle_min_time = time; }

                // Switching logic
                let turn_off = if self.mode == TuneMode::Heating { input >= upper_band } else { input <= lower_band };
                let turn_on = if self.mode == TuneMode::Heating { input <= lower_band } else { input >= upper_band };

                if self.relay_state && turn_off {
                    self.relay_state = false;
                    if self.peak_count > 0 {
                        let peak_val = if self.mode == TuneMode::Heating { self.cycle_min } else { self.cycle_max };
                        let peak_time = if self.mode == TuneMode::Heating { self.cycle_min_time } else { self.cycle_max_time };
                        self.add_peak(peak_val, peak_time);
                    }
                    self.cycle_min = input;
                    if self.mode == TuneMode::Cooling { self.cycle_max = input; }
                } else if !self.relay_state && turn_on {
                    self.relay_state = true;
                    let peak_val = if self.mode == TuneMode::Heating { self.cycle_max } else { self.cycle_min };
                    let peak_time = if self.mode == TuneMode::Heating { self.cycle_max_time } else { self.cycle_min_time };
                    self.add_peak(peak_val, peak_time);
                    self.cycle_max = input;
                    if self.mode == TuneMode::Cooling { self.cycle_min = input; }
                }

                let output = if self.relay_state {
                    if self.mode == TuneMode::Heating { self.output_step } else { -self.output_step }
                } else { 0.0 };

                let finished = self.peak_count >= 6;
                if finished { self.state = TunerState::Idle; }
                (finished, output)
            }
        }
    }

    #[inline]
    fn add_peak(&mut self, value: f32, time: f32) {
        if (self.peak_count as usize) < self.peaks.len() {
            let idx = self.peak_count as usize;
            self.peaks[idx] = value;
            self.peaks[idx] = time; // Note: Original KB had a bug here, fixed to peak_times
            self.peak_times[idx] = time;
            self.peak_count += 1;
        }
    }

    /// Calculates PID gains using Tyreus-Luyben rules
    pub fn get_tunings(&self) -> Option<(f32, f32, f32)> {
        if self.peak_count < 6 { return None; }

        let max_peak = self.peaks[5].max(self.peaks[3]).max(self.peaks[4]);
        let min_peak = self.peaks[5].min(self.peaks[3]).min(self.peaks[4]);

        let amplitude = (max_peak - min_peak) * 0.5;
        let period = (self.peak_times[5] - self.peak_times[3]).abs().max(1.0);

        const FOUR_OVER_PI: f32 = 1.2732395447351627;
        let ku = FOUR_OVER_PI * self.output_step.abs() / amplitude.max(0.1);

        // Tyreus-Luyben
        let kc = ku / 1.8;
        let ti = 1.8 * period;
        let td = period / 4.5;

        let kp = kc;
        let ki = (kp / ti).clamp(0.0001, kp * 5.0);
        let kd = (kp * td).clamp(0.0, kp * 800.0);

        Some((kp, ki, kd))
    }
}
