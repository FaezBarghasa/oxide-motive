use oxide_protocol::ClockSync;
use divan::{Divan, Bencher};

#[test]
fn test_clock_sync_convergence() {
    let mut clock_sync = ClockSync::new();

    // Simulate a series of sync exchanges with a fixed offset and some jitter
    let true_offset = 1_000_000; // 1ms
    let network_delay = 50_000; // 50us
    let mut mcu_time = 1_000_000_000;

    for _ in 0..100 {
        let host_tx_time = mcu_time - true_offset;
        let mcu_rx_time = mcu_time + network_delay;
        let mcu_tx_time = mcu_rx_time + 1000; // MCU processing time
        let host_rx_time = host_tx_time + (mcu_tx_time - mcu_rx_time) + 2 * network_delay;

        clock_sync.process_sync_exchange(host_tx_time, mcu_rx_time, mcu_tx_time, host_rx_time);
        mcu_time += 100_000_000; // 100ms between syncs
    }

    // After 100 iterations, the offset should be close to the true offset
    let error = (clock_sync.offset_ns - true_offset as i64).abs();
    assert!(error < 1000, "Offset error is too high: {}", error); // less than 1us error
}

#[divan::bench]
fn bench_translate_time(bencher: Bencher) {
    let clock_sync = ClockSync::new();
    let mcu_time = 1_234_567_890_123;

    bencher.bench_local(|| {
        clock_sync.translate_mcu_time_to_host_time(mcu_time)
    });
}
