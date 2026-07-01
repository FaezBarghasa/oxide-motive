use oxide_math::{PidController, Table3D};
use divan::{Divan, Bencher};

#[test]
fn test_table_3d_interpolation() {
    let x_axis = [0.0, 1.0];
    let y_axis = [0.0, 1.0];
    let data = [[0.0, 1.0], [2.0, 3.0]];
    let table = Table3D::new(x_axis, y_axis, data);

    assert_eq!(table.interpolate(0.5, 0.5), 1.5);
}

#[divan::bench]
fn bench_table_3d_interpolation(bencher: Bencher) {
    let x_axis = [0.0, 1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0, 9.0, 10.0, 11.0, 12.0, 13.0, 14.0, 15.0];
    let y_axis = [0.0, 1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0, 9.0, 10.0, 11.0, 12.0, 13.0, 14.0, 15.0];
    let mut data = [[0.0; 16]; 16];
    for i in 0..16 {
        for j in 0..16 {
            data[i][j] = (i * 16 + j) as f32;
        }
    }
    let table = Table3D::new(x_axis, y_axis, data);

    bencher.bench_local(|| {
        table.interpolate(7.5, 7.5)
    });
}

#[test]
fn test_pid_controller() {
    let mut pid = PidController::new(0.1, 0.01, 0.05, -1.0, 1.0);
    let setpoint = 100.0;
    let mut measurement = 0.0;
    let dt = 0.1;

    for _ in 0..100 {
        let output = pid.update(setpoint, measurement, dt);
        measurement += output; // Simplified system model
    }

    assert!((measurement - setpoint).abs() < 1.0);
}
