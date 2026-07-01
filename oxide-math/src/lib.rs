#![no_std]

use heapless::Vec;

#[derive(Debug, Clone, Copy)]
pub struct Table3D<const X: usize, const Y: usize> {
    pub x_axis: [f32; X],
    pub y_axis: [f32; Y],
    pub values: [[f32; Y]; X],
}

impl<const X: usize, const Y: usize> Table3D<X, Y> {
    pub fn new() -> Self {
        Self {
            x_axis: [0.0; X],
            y_axis: [0.0; Y],
            values: [[0.0; Y]; X],
        }
    }

    // 2D interpolation
    pub fn get(&self, x: f32, y: f32) -> f32 {
        // Find indices
        let x_idx = self.x_axis.iter().position(|&v| v >= x).unwrap_or(X - 1);
        let y_idx = self.y_axis.iter().position(|&v| v >= y).unwrap_or(Y - 1);

        // For simplicity, just returning the closest value.
        // A real implementation would interpolate.
        self.values[x_idx][y_idx]
    }
}

impl<const X: usize, const Y: usize> Default for Table3D<X,Y> {
    fn default() -> Self {
        Self::new()
    }
}
