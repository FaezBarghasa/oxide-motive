#![no_std]

pub struct LookupTable3D<const X_SIZE: usize, const Y_SIZE: usize> {
    pub x_axis: [f32; X_SIZE],
    pub y_axis: [f32; Y_SIZE],
    pub data: [[f32; Y_SIZE]; X_SIZE],
}

impl<const X_SIZE: usize, const Y_SIZE: usize> LookupTable3D<X_SIZE, Y_SIZE> {
    #[inline(always)]
    pub fn interpolate(&self, x: f32, y: f32) -> f32 {
        let x_idx = self.x_axis.binary_search_by(|probe| probe.partial_cmp(&x).unwrap());
        let y_idx = self.y_axis.binary_search_by(|probe| probe.partial_cmp(&y).unwrap());

        let (x1_idx, x2_idx, t) = match x_idx {
            Ok(idx) => (idx, idx, 0.0),
            Err(idx) => {
                if idx == 0 {
                    (0, 0, 0.0)
                } else if idx >= X_SIZE {
                    (X_SIZE - 1, X_SIZE - 1, 0.0)
                } else {
                    let x1 = self.x_axis[idx - 1];
                    let x2 = self.x_axis[idx];
                    (idx - 1, idx, (x - x1) / (x2 - x1))
                }
            }
        };

        let (y1_idx, y2_idx, u) = match y_idx {
            Ok(idx) => (idx, idx, 0.0),
            Err(idx) => {
                if idx == 0 {
                    (0, 0, 0.0)
                } else if idx >= Y_SIZE {
                    (Y_SIZE - 1, Y_SIZE - 1, 0.0)
                } else {
                    let y1 = self.y_axis[idx - 1];
                    let y2 = self.y_axis[idx];
                    (idx - 1, idx, (y - y1) / (y2 - y1))
                }
            }
        };

        let z00 = self.data[x1_idx][y1_idx];
        let z10 = self.data[x2_idx][y1_idx];
        let z01 = self.data[x1_idx][y2_idx];
        let z11 = self.data[x2_idx][y2_idx];

        let z0 = z00 * (1.0 - t) + z10 * t;
        let z1 = z01 * (1.0 - t) + z11 * t;

        z0 * (1.0 - u) + z1 * u
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_interpolation() {
        let table = LookupTable3D {
            x_axis: [1000.0, 2000.0, 3000.0, 4000.0],
            y_axis: [20.0, 40.0, 60.0, 80.0],
            data: [
                [10.0, 20.0, 30.0, 40.0],
                [15.0, 25.0, 35.0, 45.0],
                [20.0, 30.0, 40.0, 50.0],
                [25.0, 35.0, 45.0, 55.0],
            ],
        };

        // Test at grid intersection
        assert_eq!(table.interpolate(2000.0, 40.0), 25.0);

        // Test at midpoint
        assert_eq!(table.interpolate(1500.0, 30.0), 17.5);

        // Test clamping
        assert_eq!(table.interpolate(500.0, 10.0), 10.0);
        assert_eq!(table.interpolate(5000.0, 90.0), 55.0);
    }
}
