//! Automotive-Grade PID Controller with Derivative-on-Measurement
//! Zero `libm` dependency. Uses hardware FPU via `core::f32`.
#![no_std]

pub struct PidController {
    // Gains
    kp: f32,
    ki: f32,
    kd: f32,

    // Limits & Setpoint
    setpoint: f32,
    output_min: f32,
    output_max: f32,

    // State
    integral: f32,
    prev_measurement: f32,
    prev_d_term_filtered: f32,
    prev_time: f32,

    // Derivative Low-Pass Filter coefficient (0.0 to 1.0)
    // Lower = more filtering. Typically 0.1 for noisy automotive sensors.
    d_filter_alpha: f32,
}

impl PidController {
    pub fn new(kp: f32, ki: f32, kd: f32, output_min: f32, output_max: f32) -> Self {
        Self {
            kp, ki, kd,
            setpoint: 0.0,
            output_min,
            output_max,
            integral: 0.0,
            prev_measurement: 0.0,
            prev_d_term_filtered: 0.0,
            prev_time: 0.0,
            d_filter_alpha: 0.1, // Default LPF for derivative
        }
    }

    pub fn set_setpoint(&mut self, setpoint: f32) { self.setpoint = setpoint; }
    pub fn set_gains(&mut self, kp: f32, ki: f32, kd: f32) { self.kp = kp; self.ki = ki; self.kd = kd; }
    pub fn set_d_filter(&mut self, alpha: f32) { self.d_filter_alpha = alpha.clamp(0.0, 1.0); }

    pub fn reset(&mut self) {
        self.integral = 0.0;
        self.prev_measurement = 0.0;
        self.prev_d_term_filtered = 0.0;
        self.prev_time = 0.0;
    }

    /// Computes the PID output.
    /// CRITICAL: Uses Derivative-on-Measurement to prevent derivative kick on setpoint changes.
    pub fn compute(&mut self, measurement: f32, current_time: f32) -> f32 {
        let dt = current_time - self.prev_time;

        // Prevent division by zero or erratic behavior on first call
        if dt <= 0.001 {
            return self.output_min.max(self.output_min).min(self.output_max);
        }

        let error = self.setpoint - measurement;

        // --- Proportional Term ---
        let p_term = self.kp * error;

        // --- Integral Term with Anti-Windup ---
        self.integral += error * dt;
        // Conditional anti-windup: clamp integral based on output limits
        if self.ki != 0.0 {
            let i_max = self.output_max / self.ki;
            let i_min = self.output_min / self.ki;
            self.integral = self.integral.max(i_min).min(i_max);
        }
        let i_term = self.ki * self.integral;

        // --- Derivative Term (Derivative-on-Measurement + LPF) ---
        // Derivative of measurement (negative because error = setpoint - measurement)
        let raw_d_measurement = (self.prev_measurement - measurement) / dt;

        // Apply First-Order Low-Pass Filter to prevent high-frequency noise spikes
        let filtered_d = (self.d_filter_alpha * raw_d_measurement) +
                         ((1.0 - self.d_filter_alpha) * self.prev_d_term_filtered);
        self.prev_d_term_filtered = filtered_d;

        let d_term = self.kd * filtered_d;

        // --- Total Output ---
        let mut output = p_term + i_term + d_term;

        // Final output clamping
        output = output.max(self.output_min).min(self.output_max);

        // Update state for next iteration
        self.prev_measurement = measurement;
        self.prev_time = current_time;

        output
    }
}
