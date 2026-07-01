use std::{
    fs::{File, OpenOptions},
    io::{self, Write},
    os::unix::io::AsRawFd,
    sync::atomic::Ordering,
    time::{Duration, Instant},
};
use memmap2::MmapMut;
use oxide_protocol::{SharedMemoryLayout, SpscQueue, HostToMcu, McuToHost, TableUpdate};

// Define the shared memory region. This address must match the M4 side's mapping.
const SHARED_SRAM_BASE: usize = 0x3800_0000; // Example base address for SRAM4
const SHARED_MEMORY_OFFSET: usize = 0x1000; // Offset to place our struct
const SHARED_MEMORY_ADDR: usize = SHARED_SRAM_BASE + SHARED_MEMORY_OFFSET;
const SHARED_MEMORY_SIZE: usize = 4096; // Size of the shared memory region (e.g., 4KB)

pub fn main() {
    println!("A7 Host: Starting up...");

    // Open /dev/mem to access physical memory
    let mut file = OpenOptions::new()
        .read(true)
        .write(true)
        .open("/dev/mem")
        .expect("Failed to open /dev/mem. Run with sudo or ensure permissions.");

    // Memory map the shared SRAM region
    let mmap = unsafe {
        MmapMut::map_mut(
            &mut file,
            memmap2::MmapOptions::new()
                .offset(SHARED_MEMORY_ADDR as u64)
                .len(SHARED_MEMORY_SIZE)
                .build()
                .expect("Failed to mmap shared memory"),
        )
    };

    let shared_mem_ptr = mmap.as_mut_ptr() as *mut SharedMemoryLayout;
    let shared_mem = unsafe { &mut *shared_mem_ptr };

    // A7 heartbeat bit
    const A7_HEARTBEAT_BIT: u32 = 1 << 0;

    println!("A7 Host: Shared memory mapped at {:p}", shared_mem_ptr);

    let mut last_command_time = Instant::now();
    let mut command_counter = 0;

    loop {
        // Update A7 heartbeat
        shared_mem.heartbeat_flags.fetch_or(A7_HEARTBEAT_BIT, Ordering::Relaxed);

        // Check M4 heartbeat
        let m4_heartbeat = shared_mem.heartbeat_flags.load(Ordering::Relaxed) & (1 << 1);
        if m4_heartbeat != 0 {
            // M4 is alive
            // Clear M4 heartbeat for next cycle (or M4 clears its own)
            // For now, A7 clears it for demonstration
            shared_mem.heartbeat_flags.fetch_and(!(1 << 1), Ordering::Relaxed);
        } else {
            println!("A7 Host: WARNING: M4 heartbeat lost!");
        }

        // Process telemetry from M4
        while let Some(telemetry) = shared_mem.telemetry_ring_buffer.dequeue() {
            match telemetry {
                McuToHost::TelemetryBatch { timestamp_us, rpm, .. } => {
                    // println!("A7 Host: Received Telemetry: RPM={}, Timestamp={}", rpm, timestamp_us);
                },
                _ => {
                    // println!("A7 Host: Received other telemetry: {:?}", telemetry);
                }
            }
        }

        // Send commands to M4 periodically
        if last_command_time.elapsed() >= Duration::from_secs(1) {
            let command = if command_counter % 2 == 0 {
                HostToMcu::ConfigUpdate {
                    config: oxide_protocol::EcuConfig {
                        injector_size_cc: 550 + command_counter as u16,
                        trigger_pattern: oxide_protocol::TriggerPattern::MissingTooth(36, 1),
                        num_cylinders: 4,
                        rev_limit_rpm: 8500,
                        boost_cut_kpa: 200,
                    },
                }
            } else {
                HostToMcu::TableUpdate {
                    table_id: 1,
                    x_idx: (command_counter % 10) as u8,
                    y_idx: (command_counter % 10) as u8,
                    value: command_counter as f32 * 0.1,
                }
            };

            if let Err(item) = shared_mem.command_ring_buffer.enqueue(command) {
                eprintln!("A7 Host: Command buffer full, dropped: {:?}", item);
            } else {
                // println!("A7 Host: Sent command: {:?}", command);
            }

            // Send a direct table update via mailbox
            let mailbox_update = TableUpdate {
                table_id: 2,
                x_idx: (command_counter % 5) as u8,
                y_idx: (command_counter % 5) as u8,
                value: command_counter as f32 * 1.0,
            };
            shared_mem.table_update_mailbox.replace(mailbox_update);


            last_command_time = Instant::now();
            command_counter += 1;
        }

        std::thread::sleep(Duration::from_millis(100)); // Simulate work
    }
}
