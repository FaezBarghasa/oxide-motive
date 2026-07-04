
use divan::{Divan, Bencher};
use oxide_math::Table3D;

fn main() {
    Divan::main();
}

#[divan::bench]
fn interpolate_16x16(bencher: Bencher) {
    let table = Table3D::<f32, 16, 16>::new();
    bencher.bench_local(|| {
        table.interpolate(1500.0, 75.0)
    });
}

// Placeholder for UKF and SMC benchmarks.
// These will be added once the `estimation` module is more mature
// and doesn't rely on any std or libm features.

/*
use oxide_math::estimation::UnscentedKalmanFilter;
use nalgebra::{Vector3, Vector2, Matrix3};

#[divan::bench]
fn ukf_predict(bencher: Bencher) {
    let mut ukf = UnscentedKalmanFilter::new(
        Vector3::new(0.0, 0.0, 0.0),
        Matrix3::identity() * 0.1,
        Matrix3::identity() * 0.01,
        nalgebra::Matrix2::identity() * 0.05,
    );

    let control_input = 0.0;
    let dt = 0.001; // 1ms

    bencher.bench_local(|| {
        ukf.predict(control_input, dt);
    });
}
*/
