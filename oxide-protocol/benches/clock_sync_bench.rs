
use divan::{Divan, Bencher};
use oxide_protocol::clock_sync::ClockSync;

fn main() {
    Divan::main();
}

#[divan::bench]
fn translate_time_bench(bencher: Bencher) {
    let mut clock_sync = ClockSync::new();
    clock_sync.offset_ns = 1_000_000; // 1ms offset
    let mcu_time = 1_000_000_000_u64; // 1s

    bencher.bench_local(|| {
        clock_sync.translate_mcu_time_to_host_time(mcu_time);
    });
}
