//! Mathematical utilities for Oxide Motive.
#![no_std]
#![deny(missing_docs)]

/// State estimation algorithms.
pub mod estimation;

/// Mathematical errors.
#[derive(Debug)]
pub enum MathError {
    /// Input value is out of bounds.
    OutOfBounds,
}

/// A 3D lookup table for engine calibration maps (e.g., VE, spark).
/// X_SIZE and Y_SIZE are the dimensions of the table axes.
pub struct Table3D<T, const X_SIZE: usize, const Y_SIZE: usize> {
    /// X axis calibration points.
    pub x_axis: [T; X_SIZE],
    /// Y axis calibration points.
    pub y_axis: [T; Y_SIZE],
    /// 2D grid of values corresponding to the X and Y coordinates.
    pub data: [[T; Y_SIZE]; X_SIZE],
}

impl<T: Copy + Default, const X_SIZE: usize, const Y_SIZE: usize> Table3D<T, X_SIZE, Y_SIZE> {
    /// Creates a new, zeroed lookup table.
    pub fn new() -> Self {
        Self {
            x_axis: [T::default(); X_SIZE],
            y_axis: [T::default(); Y_SIZE],
            data: [[T::default(); Y_SIZE]; X_SIZE],
        }
    }

    /// Creates a new table from the given 2D data.
    pub fn new_from_data(data: [[T; Y_SIZE]; X_SIZE]) -> Self {
        Self {
            x_axis: [T::default(); X_SIZE],
            y_axis: [T::default(); Y_SIZE],
            data,
        }
    }
}

impl<const X_SIZE: usize, const Y_SIZE: usize> Table3D<f32, X_SIZE, Y_SIZE> {
    /// Performs bilinear interpolation on the 3D table.
    ///
    /// # Arguments
    /// * `x` - The value on the x-axis (e.g., RPM).
    /// * `y` - The value on the y-axis (e.g., Engine Load).
    ///
    /// # Returns
    /// The interpolated value from the table.
    /// If the inputs are out of bounds, the value is clamped to the table edges.
    pub fn interpolate(&self, x: f32, y: f32) -> f32 {
        // Find indices for x-axis
        let (x_idx, x_frac) = self.find_index_and_fraction(x, &self.x_axis);
        // Find indices for y-axis
        let (y_idx, y_frac) = self.find_index_and_fraction(y, &self.y_axis);

        // Get the four surrounding points
        let z00 = self.data[x_idx][y_idx];
        let z10 = self.data[x_idx + 1][y_idx];
        let z01 = self.data[x_idx][y_idx + 1];
        let z11 = self.data[x_idx + 1][y_idx + 1];

        // Bilinear interpolation formula
        let z0 = z00 + x_frac * (z10 - z00);
        let z1 = z01 + x_frac * (z11 - z01);

        z0 + y_frac * (z1 - z0)
    }

    /// Helper function to find the lower-bound index and the fractional component
    /// for a given value on an axis. Clamps to the boundaries.
    fn find_index_and_fraction(&self, val: f32, axis: &[f32]) -> (usize, f32) {
        if val <= axis[0] {
            return (0, 0.0);
        }
        if val >= axis[axis.len() - 1] {
            return (axis.len() - 2, 1.0);
        }

        // Binary search to find the lower bound index
        let mut low = 0;
        let mut high = axis.len() - 1;
        while low <= high {
            let mid = low + (high - low) / 2;
            if axis[mid] < val {
                low = mid + 1;
            } else {
                high = mid - 1;
            }
        }

        let idx = high;
        let x0 = axis[idx];
        let x1 = axis[idx + 1];
        let fraction = (val - x0) / (x1 - x0);

        (idx, fraction)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_interpolation_in_bounds() {
        let table = Table3D {
            x_axis: [1000.0, 2000.0, 3000.0, 4000.0],
            y_axis: [50.0, 100.0, 150.0, 200.0],
            data: [
                [10.0, 20.0, 30.0, 40.0],
                [15.0, 25.0, 35.0, 45.0],
                [20.0, 30.0, 40.0, 50.0],
                [25.0, 35.0, 45.0, 55.0],
            ],
        };

        assert_eq!(table.interpolate(1500.0, 75.0), 17.5);
    }

    #[test]
    fn test_interpolation_out_of_bounds() {
        let table = Table3D {
            x_axis: [1000.0, 2000.0, 3000.0, 4000.0],
            y_axis: [50.0, 100.0, 150.0, 200.0],
            data: [
                [10.0, 20.0, 30.0, 40.0],
                [15.0, 25.0, 35.0, 45.0],
                [20.0, 30.0, 40.0, 50.0],
                [25.0, 35.0, 45.0, 55.0],
            ],
        };

        // Clamp to lower x boundary
        assert_eq!(table.interpolate(500.0, 75.0), 15.0);
        // Clamp to upper x boundary
        assert_eq!(table.interpolate(5000.0, 125.0), 50.0);
        // Clamp to lower y boundary
        assert_eq!(table.interpolate(1500.0, 25.0), 12.5);
        // Clamp to upper y boundary
        assert_eq!(table.interpolate(2500.0, 250.0), 45.0);
    }
}
