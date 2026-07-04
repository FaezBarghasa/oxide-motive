use divan::{Divan, black_box};
use oxide_protocol::clock_sync::ClockSyncManager;
use oxide_protocol::ClockSync;

fn main() {
    // Run benchmarks
    Divan::main();
}

#[divan::bench]
fn bench_clock_sync_manager_update() {
    let mut manager = ClockSyncManager::new();
    let message = ClockSync {
        origin_timestamp: 1000,
        receive_timestamp: 1100,
        transmit_timestamp: 1200,
    };
    manager.update(&message, black_box(1300));
}
