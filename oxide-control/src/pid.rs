
use core::f32;

pub struct PidController {
    kp: f32,
    ki: f32,
    kd: f32,
    setpoint: f32,
    integral: f32,
    prev_error: f32,
    prev_measurement: f32,
    alpha: f32,
    filtered_d: f32,
}

impl PidController {
    pub fn new(kp: f32, ki: f32, kd: f32, setpoint: f32, alpha: f32) -> Self {
        Self {
            kp,
            ki,
            kd,
            setpoint,
            integral: 0.0,
            prev_error: 0.0,
            prev_measurement: 0.0,
            alpha,
            filtered_d: 0.0,
        }
    }

    pub fn update(&mut self, measurement: f32, dt: f32) -> f32 {
        let error = self.setpoint - measurement;

        let proportional = self.kp * error;

        self.integral += self.ki * error * dt;
        // Anti-windup
        self.integral = self.integral.max(-100.0).min(100.0);

        let raw_d = self.kd * (self.prev_measurement - measurement) / dt;
        self.filtered_d = (self.alpha * raw_d) + ((1.0 - self.alpha) * self.filtered_d);

        self.prev_error = error;
        self.prev_measurement = measurement;

        proportional + self.integral + self.filtered_d
    }

    pub fn set_setpoint(&mut self, setpoint: f32) {
        self.setpoint = setpoint;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pid_derivative_on_measurement() {
        let mut pid = PidController::new(1.0, 0.1, 0.01, 50.0, 0.5);
        let output1 = pid.update(40.0, 0.1);
        pid.set_setpoint(60.0);
        let output2 = pid.update(40.0, 0.1);
        // Without derivative kick, the output should not spike significantly
        assert!((output2 - output1).abs() < 15.0);
    }
}
