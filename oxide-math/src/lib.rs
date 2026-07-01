#![cfg_attr(not(feature = "std"), no_std)]

#[cfg(feature = "std")]
extern crate std;

use num_traits::{Float, FromPrimitive};
use core::ops::{Add, Sub, Mul, Div};
use serde::{Serialize, Deserialize};
use heapless::Vec;

/// Error type for math operations.
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum MathError {
    /// Index out of bounds for table access.
    IndexOutOfBounds,
    /// Division by zero occurred.
    DivisionByZero,
    /// Invalid input value.
    InvalidInput,
}

/// A 3D interpolation table (e.g., for VE or Spark maps).
///
/// The table is indexed by X and Y axes, and stores Z values.
///
/// # Type Parameters
/// * `X_SIZE` - The number of entries in the X-axis.
/// * `Y_SIZE` - The number of entries in the Y-axis.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Table3D<const X_SIZE: usize, const Y_SIZE: usize> {
    /// The values for the X-axis (e.g., RPM).
    pub x_axis: [f32; X_SIZE],
    /// The values for the Y-axis (e.g., MAP).
    pub y_axis: [f32; Y_SIZE],
    /// The Z-values (e.g., VE percentage, Spark Advance).
    pub z_values: [[f32; Y_SIZE]; X_SIZE],
}

impl<const X_SIZE: usize, const Y_SIZE: usize> Table3D<X_SIZE, Y_SIZE> {
    /// Creates a new `Table3D` with all values initialized to zero.
    pub fn new() -> Self {
        Self {
            x_axis: [0.0; X_SIZE],
            y_axis: [0.0; Y_SIZE],
            z_values: [[0.0; Y_SIZE]; X_SIZE],
        }
    }

    /// Interpolates a Z-value given X and Y inputs.
    ///
    /// Performs bilinear interpolation.
    ///
    /// # Arguments
    /// * `x_input` - The X-axis input value.
    /// * `y_input` - The Y-axis input value.
    ///
    /// # Returns
    /// The interpolated Z-value.
    pub fn interpolate(&self, x_input: f32, y_input: f32) -> f32 {
        // Find the bounding box in the table
        let (x1_idx, x2_idx, x_frac) = self.find_indices_and_fraction(x_input, &self.x_axis);
        let (y1_idx, y2_idx, y_frac) = self.find_indices_and_fraction(y_input, &self.y_axis);

        // Get the four corner values
        let z11 = self.z_values[x1_idx][y1_idx];
        let z12 = self.z_values[x1_idx][y2_idx];
        let z21 = self.z_values[x2_idx][y1_idx];
        let z22 = self.z_values[x2_idx][y2_idx];

        // Bilinear interpolation
        let z_x1 = z11 * (1.0 - y_frac) + z12 * y_frac;
        let z_x2 = z21 * (1.0 - y_frac) + z22 * y_frac;

        z_x1 * (1.0 - x_frac) + z_x2 * x_frac
    }

    /// Finds the two bounding indices and the fractional position between them.
    ///
    /// # Arguments
    /// * `input` - The input value.
    /// * `axis` - The axis array (x_axis or y_axis).
    ///
    /// # Returns
    /// A tuple `(idx1, idx2, fraction)` where `idx1` and `idx2` are the bounding indices
    /// and `fraction` is the position between `axis[idx1]` and `axis[idx2]`.
    fn find_indices_and_fraction(&self, input: f32, axis: &[f32]) -> (usize, usize, f32) {
        if input <= axis[0] {
            return (0, 0, 0.0);
        }
        if input >= axis[axis.len() - 1] {
            return (axis.len() - 1, axis.len() - 1, 0.0);
        }

        for i in 0..(axis.len() - 1) {
            if input >= axis[i] && input <= axis[i + 1] {
                let range = axis[i + 1] - axis[i];
                let fraction = (input - axis[i]) / range;
                return (i, i + 1, fraction);
            }
        }
        // Should not be reached due to boundary checks
        (0, 0, 0.0)
    }

    /// Sets a specific Z-value in the table.
    ///
    /// # Arguments
    /// * `x_idx` - The index on the X-axis.
    /// * `y_idx` - The index on the Y-axis.
    /// * `value` - The new Z-value.
    ///
    /// # Returns
    /// `Ok(())` if the value was set, or `MathError::IndexOutOfBounds` if indices are invalid.
    pub fn set_z_value(&mut self, x_idx: usize, y_idx: usize, value: f32) -> Result<(), MathError> {
        if x_idx >= X_SIZE || y_idx >= Y_SIZE {
            return Err(MathError::IndexOutOfBounds);
        }
        self.z_values[x_idx][y_idx] = value;
        Ok(())
    }

    /// Gets a specific Z-value from the table.
    ///
    /// # Arguments
    /// * `x_idx` - The index on the X-axis.
    /// * `y_idx` - The index on the Y-axis.
    ///
    /// # Returns
    /// The Z-value, or `MathError::IndexOutOfBounds` if indices are invalid.
    pub fn get_z_value(&self, x_idx: usize, y_idx: usize) -> Result<f32, MathError> {
        if x_idx >= X_SIZE || y_idx >= Y_SIZE {
            return Err(MathError::IndexOutOfBounds);
        }
        Ok(self.z_values[x_idx][y_idx])
    }
}

/// Fixed-point number representation for integer-only math.
///
/// This struct represents a number with a fixed number of fractional bits.
///
/// # Type Parameters
/// * `FRAC_BITS` - The number of bits used for the fractional part.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct FixedPoint<const FRAC_BITS: u8> {
    value: i32,
}

impl<const FRAC_BITS: u8> FixedPoint<FRAC_BITS> {
    /// Creates a new `FixedPoint` number from an integer.
    pub fn from_int(int_part: i32) -> Self {
        Self {
            value: int_part << FRAC_BITS,
        }
    }

    /// Creates a new `FixedPoint` number from a float.
    pub fn from_float(float_part: f32) -> Self {
        Self {
            value: (float_part * (1 << FRAC_BITS) as f32) as i32,
        }
    }

    /// Converts the `FixedPoint` number to an integer (truncates fractional part).
    pub fn to_int(&self) -> i32 {
        self.value >> FRAC_BITS
    }

    /// Converts the `FixedPoint` number to a float.
    pub fn to_float(&self) -> f32 {
        self.value as f32 / (1 << FRAC_BITS) as f32
    }
}

impl<const FRAC_BITS: u8> Add for FixedPoint<FRAC_BITS> {
    type Output = Self;

    fn add(self, other: Self) -> Self::Output {
        Self {
            value: self.value + other.value,
        }
    }
}

impl<const FRAC_BITS: u8> Sub for FixedPoint<FRAC_BITS> {
    type Output = Self;

    fn sub(self, other: Self) -> Self::Output {
        Self {
            value: self.value - other.value,
        }
    }
}

impl<const FRAC_BITS: u8> Mul for FixedPoint<FRAC_BITS> {
    type Output = Self;

    fn mul(self, other: Self) -> Self::Output {
        Self {
            value: (self.value as i64 * other.value as i64 >> FRAC_BITS) as i32,
        }
    }
}

impl<const FRAC_BITS: u8> Div for FixedPoint<FRAC_BITS> {
    type Output = Result<Self, MathError>;

    fn div(self, other: Self) -> Self::Output {
        if other.value == 0 {
            return Err(MathError::DivisionByZero);
        }
        Ok(Self {
            value: ((self.value as i64) << FRAC_BITS) / other.value as i64 as i32,
        })
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_table3d_new() {
        let table: Table3D<2, 2> = Table3D::new();
        assert_eq!(table.x_axis, [0.0, 0.0]);
        assert_eq!(table.y_axis, [0.0, 0.0]);
        assert_eq!(table.z_values, [[0.0, 0.0], [0.0, 0.0]]);
    }

    #[test]
    fn test_table3d_set_get_z_value() {
        let mut table: Table3D<2, 2> = Table3D::new();
        assert!(table.set_z_value(0, 0, 10.0).is_ok());
        assert_eq!(table.get_z_value(0, 0).unwrap(), 10.0);
        assert!(table.set_z_value(1, 1, 20.0).is_ok());
        assert_eq!(table.get_z_value(1, 1).unwrap(), 20.0);
    }

    #[test]
    fn test_table3d_set_get_z_value_out_of_bounds() {
        let mut table: Table3D<2, 2> = Table3D::new();
        assert_eq!(table.set_z_value(2, 0, 10.0), Err(MathError::IndexOutOfBounds));
        assert_eq!(table.get_z_value(0, 2), Err(MathError::IndexOutOfBounds));
    }

    #[test]
    fn test_table3d_interpolate_simple() {
        let mut table: Table3D<2, 2> = Table3D::new();
        table.x_axis = [0.0, 100.0];
        table.y_axis = [0.0, 100.0];
        table.z_values = [[0.0, 10.0], [20.0, 30.0]];

        // Exact points
        assert_eq!(table.interpolate(0.0, 0.0), 0.0);
        assert_eq!(table.interpolate(100.0, 100.0), 30.0);

        // Mid-points
        assert_eq!(table.interpolate(50.0, 50.0), 15.0); // (0+10)/2 * 0.5 + (20+30)/2 * 0.5 = 5*0.5 + 25*0.5 = 2.5 + 12.5 = 15
        assert_eq!(table.interpolate(0.0, 50.0), 5.0); // (0+10)/2 = 5
        assert_eq!(table.interpolate(50.0, 0.0), 10.0); // (0+20)/2 = 10
    }

    #[test]
    fn test_table3d_interpolate_clamped() {
        let mut table: Table3D<2, 2> = Table3D::new();
        table.x_axis = [0.0, 100.0];
        table.y_axis = [0.0, 100.0];
        table.z_values = [[0.0, 10.0], [20.0, 30.0]];

        // Below min
        assert_eq!(table.interpolate(-10.0, 50.0), 5.0);
        assert_eq!(table.interpolate(50.0, -10.0), 10.0);

        // Above max
        assert_eq!(table.interpolate(110.0, 50.0), 25.0);
        assert_eq!(table.interpolate(50.0, 110.0), 20.0);
    }

    #[test]
    fn test_fixed_point_from_int_to_int() {
        let fp = FixedPoint::<8>::from_int(5);
        assert_eq!(fp.to_int(), 5);
        assert_eq!(fp.value, 5 << 8);
    }

    #[test]
    fn test_fixed_point_from_float_to_float() {
        let fp = FixedPoint::<8>::from_float(5.5);
        assert!((fp.to_float() - 5.5).abs() < 0.001);
        assert_eq!(fp.value, (5.5 * 256.0) as i32);

        let fp_neg = FixedPoint::<8>::from_float(-3.25);
        assert!((fp_neg.to_float() - -3.25).abs() < 0.001);
        assert_eq!(fp_neg.value, (-3.25 * 256.0) as i32);
    }

    #[test]
    fn test_fixed_point_add() {
        let fp1 = FixedPoint::<8>::from_float(1.5);
        let fp2 = FixedPoint::<8>::from_float(2.75);
        let sum = fp1 + fp2;
        assert!((sum.to_float() - 4.25).abs() < 0.001);
    }

    #[test]
    fn test_fixed_point_sub() {
        let fp1 = FixedPoint::<8>::from_float(5.0);
        let fp2 = FixedPoint::<8>::from_float(1.25);
        let diff = fp1 - fp2;
        assert!((diff.to_float() - 3.75).abs() < 0.001);
    }

    #[test]
    fn test_fixed_point_mul() {
        let fp1 = FixedPoint::<8>::from_float(1.5);
        let fp2 = FixedPoint::<8>::from_float(2.0);
        let prod = fp1 * fp2;
        assert!((prod.to_float() - 3.0).abs() < 0.001);

        let fp3 = FixedPoint::<8>::from_float(0.5);
        let fp4 = FixedPoint::<8>::from_float(0.5);
        let prod2 = fp3 * fp4;
        assert!((prod2.to_float() - 0.25).abs() < 0.001);
    }

    #[test]
    fn test_fixed_point_div() {
        let fp1 = FixedPoint::<8>::from_float(5.0);
        let fp2 = FixedPoint::<8>::from_float(2.0);
        let div_res = fp1 / fp2;
        assert!(div_res.is_ok());
        assert!((div_res.unwrap().to_float() - 2.5).abs() < 0.001);
    }

    #[test]
    fn test_fixed_point_div_by_zero() {
        let fp1 = FixedPoint::<8>::from_float(5.0);
        let fp2 = FixedPoint::<8>::from_int(0);
        let div_res = fp1 / fp2;
        assert_eq!(div_res, Err(MathError::DivisionByZero));
    }
}

#[cfg(all(feature = "std", feature = "divan"))]
mod benches {
    use super::*;
    use divan::{black_box, Bencher};

    #[divan::bench]
    fn bench_table3d_interpolate(bencher: Bencher) {
        let mut table: Table3D<16, 16> = Table3D::new();
        for i in 0..16 {
            table.x_axis[i] = i as f32 * 100.0; // 0 to 1500
            table.y_axis[i] = i as f32 * 10.0; // 0 to 150
            for j in 0..16 {
                table.z_values[i][j] = (i as f32 * 0.5) + (j as f32 * 0.2);
            }
        }

        bencher.iter(|| {
            black_box(table.interpolate(black_box(750.0), black_box(75.0)));
        });
    }

    #[divan::bench]
    fn bench_fixed_point_mul(bencher: Bencher) {
        let fp1 = FixedPoint::<16>::from_float(123.45);
        let fp2 = FixedPoint::<16>::from_float(67.89);
        bencher.iter(|| {
            black_box(fp1 * fp2);
        });
    }

    #[divan::bench]
    fn bench_fixed_point_div(bencher: Bencher) {
        let fp1 = FixedPoint::<16>::from_float(12345.67);
        let fp2 = FixedPoint::<16>::from_float(89.12);
        bencher.iter(|| {
            black_box(fp1 / fp2);
        });
    }
}
