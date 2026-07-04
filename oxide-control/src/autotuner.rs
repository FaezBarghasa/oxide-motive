use heapless::Vec;
use crate::pid::PidController;

#[derive(Debug, PartialEq)]
pub enum TunerState {
    Idle,
    Cycling,
    Done,
}

pub struct UpRelayAutotuner {
    state: TunerState,
    setpoint: f32,
    relay_amplitude: f32,
    peaks: Vec<f32, 10>,
    peak_times: Vec<u64, 10>,
    last_peak_time: u64,
    last_output: f32,
}

impl UpRelayAutotuner {
    pub fn new(setpoint: f32, relay_amplitude: f32) -> Self {
        Self {
            state: TunerState::Idle,
            setpoint,
            relay_amplitude,
            peaks: Vec::new(),
            peak_times: Vec::new(),
            last_peak_time: 0,
            last_output: 0.0,
        }
    }

    pub fn update(&mut self, measurement: f32, time: u64) -> f32 {
        if self.state == TunerState::Idle {
            self.state = TunerState::Cycling;
        }

        if self.state == TunerState::Cycling {
            let output = if measurement < self.setpoint {
                self.relay_amplitude
            } else {
                -self.relay_amplitude
            };

            if (output > 0.0 && self.last_output < 0.0) || (output < 0.0 && self.last_output > 0.0) {
                if self.peaks.is_full() {
                    self.peaks.pop_front();
                    self.peak_times.pop_front();
                }
                self.peaks.push_back(measurement).ok();
                self.peak_times.push_back(time).ok();
                self.last_peak_time = time;
            }

            self.last_output = output;

            if self.peaks.len() >= 4 {
                self.state = TunerState::Done;
            }
            output
        } else {
            0.0
        }
    }

    pub fn get_limit_cycle_data(&self) -> Option<(Vec<f32, 10>, Vec<u64, 10>)> {
        if self.state == TunerState::Cycling || self.state == TunerState::Done {
            Some((self.peaks.clone(), self.peak_times.clone()))
        } else {
            None
        }
    }

    pub fn get_gains(&self) -> Option<(f32, f32, f32)> {
        if self.state == TunerState::Done {
            let a = self.peaks.iter().sum::<f32>() / self.peaks.len() as f32;
            let t = (self.peak_times[self.peak_times.len() - 1] - self.peak_times[0]) as f32 / ((self.peaks.len() - 1) as f32 * 1_000_000.0);
            let ku = 4.0 * self.relay_amplitude / (core::f32::consts::PI * a);
            let pu = t;

            // Tyreus-Luyben tuning rules
            let kp = ku / 2.2;
            let ki = kp / (2.2 * pu);
            let kd = kp * pu / 6.3;

            Some((kp, ki, kd))
        } else {
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_autotuner() {
        let mut autotuner = UpRelayAutotuner::new(50.0, 10.0);
        let mut measurement = 40.0;
        let mut time = 0;

        for _ in 0..100 {
            let output = autotuner.update(measurement, time);
            measurement += output * 0.1;
            time += 100;
        }

        assert_eq!(autotuner.state, TunerState::Done);
        let gains = autotuner.get_gains();
        assert!(gains.is_some());
    }
}
