#![no_std]

use heapless::Vec;

pub struct Table3D<const X_SIZE: usize, const Y_SIZE: usize> {
    x_axis: [f32; X_SIZE],
    y_axis: [f32; Y_SIZE],
    data: [[f32; Y_SIZE]; X_SIZE],
}

impl<const X_SIZE: usize, const Y_SIZE: usize> Table3D<X_SIZE, Y_SIZE> {
    pub fn new(x_axis: [f32; X_SIZE], y_axis: [f32; Y_SIZE], data: [[f32; Y_SIZE]; X_SIZE]) -> Self {
        Self {
            x_axis,
            y_axis,
            data,
        }
    }

    pub fn interpolate(&self, x: f32, y: f32) -> f32 {
        let x_idx = self.find_idx(&self.x_axis, x);
        let y_idx = self.find_idx(&self.y_axis, y);

        let x1 = self.x_axis[x_idx];
        let x2 = self.x_axis[x_idx + 1];
        let y1 = self.y_axis[y_idx];
        let y2 = self.y_axis[y_idx + 1];

        let q11 = self.data[x_idx][y_idx];
        let q12 = self.data[x_idx][y_idx + 1];
        let q21 = self.data[x_idx + 1][y_idx];
        let q22 = self.data[x_idx + 1][y_idx + 1];

        let r1 = ((x2 - x) / (x2 - x1)) * q11 + ((x - x1) / (x2 - x1)) * q21;
        let r2 = ((x2 - x) / (x2 - x1)) * q12 + ((x - x1) / (x2 - x1)) * q22;

        ((y2 - y) / (y2 - y1)) * r1 + ((y - y1) / (y2 - y1)) * r2
    }

    fn find_idx(&self, axis: &[f32], value: f32) -> usize {
        axis.iter()
            .position(|&v| v >= value)
            .unwrap_or(axis.len() - 2)
            .saturating_sub(1)
    }
}

pub struct PidController {
    pub kp: f32,
    pub ki: f32,
    pub kd: f32,
    pub output_min: f32,
    pub output_max: f32,
    integral: f32,
    prev_error: f32,
}

impl PidController {
    pub fn new(kp: f32, ki: f32, kd: f32, output_min: f32, output_max: f32) -> Self {
        Self {
            kp,
            ki,
            kd,
            output_min,
            output_max,
            integral: 0.0,
            prev_error: 0.0,
        }
    }

    pub fn update(&mut self, setpoint: f32, measurement: f32, dt: f32) -> f32 {
        let error = setpoint - measurement;
        self.integral += error * dt;
        self.integral = self.integral.clamp(self.output_min, self.output_max); // Anti-windup

        let derivative = (error - self.prev_error) / dt;
        self.prev_error = error;

        let output = self.kp * error + self.ki * self.integral + self.kd * derivative;
        output.clamp(self.output_min, self.output_max)
    }
}
