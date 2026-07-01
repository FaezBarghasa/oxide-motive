use divan::{Bencher, black_box};
use oxide_protocol::ClockSync;

fn main() {
    divan::main();
}

#[divan::bench]
fn translate_time(bencher: Bencher) {
    let mut clock_sync = ClockSync::new();
    // Simulate a sync event to populate the struct
    clock_sync.process_sync_exchange(1_000_000, 1_010_000, 1_011_000, 1_021_000);

    bencher.bench_local(|| {
        black_box(clock_sync.translate_mcu_time_to_host_time(black_box(2_000_000)));
    });
}
