
use divan::{Bencher, black_box};
use oxide_math::Table3D;

fn main() {
    divan::main();
}

#[divan::bench]
fn bench_interpolate_16x16(bencher: Bencher) {
    let table: Table3D<f32, 16, 16> = Table3D::new();
    bencher.bench(|| {
        black_box(table.interpolate(1500.0, 75.0));
    });
}
