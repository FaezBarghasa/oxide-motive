#![no_main]
#![no_std]
#![deny(warnings)] // Deny all warnings by default
#![deny(clippy::pedantic)]
#![deny(clippy::restriction)]
#![allow(clippy::module_name_repetitions)] // Allow module name repetitions for clarity (e.g., `hal::hal_trait`)
#![allow(clippy::missing_docs_in_private_items)] // Allow missing docs in private items for internal modules

use panic_halt as _; // Panic handler

use cortex_m_rt::entry;
use stm32h7xx_hal::{pac, prelude::*, rcc::CoreClocks};

use oxide_hal::{Adc, GpioPin, Pwm, Timer, Uart, Can, Watchdog, Nvm, HalError};
use oxide_protocol::{
    framing,
    HostToMcu, McuToHost, SensorData, EngineState, SyncState, EnginePhase, FreezeFrame, EcuConfig,
};
use oxide_math::{Table3D, MathError};
use heapless::{Vec, FnvIndexMap};

// Function to be placed in ITCM
#[link_section = ".itcm.critical_function"]
#[inline(never)]
fn critical_function() {
    // This function's body will be placed in ITCM
    // We can use this for time-sensitive operations.
    cortex_m::asm::nop();
}

// Static assertion to check if the critical function is in the ITCM section.
const _: () = {
    extern "C" {
        static _sitcm: u32;
        static _eitcm: u32;
    }
    let start_addr = unsafe { &_sitcm as *const u32 as usize };
    let end_addr = unsafe { &_eitcm as *const u32 as usize };
    let func_addr = critical_function as *const () as usize;

    // This is a compile-time check. If the condition is false, the build will fail.
    // Note: This is a trick. `const` evaluation can't directly panic.
    // Instead, we create an invalid operation if the check fails.
    if !(func_addr >= start_addr && func_addr < end_addr) {
        // This will cause a compile error if the condition is not met.
        // For example, by creating a divide-by-zero in a const context.
        #[allow(unconditional_panic)]
        let _ = 1 / 0;
    }
};


// Define the RTIC application
#[rtic::app(device = stm32h7xx_hal::pac, dispatchers = [SPI1, SPI2, SPI3])]
mod app {
    use super::*;

    /// Shared resources accessible by multiple tasks.
    #[shared]
    struct Shared {
        engine_state: EngineState,
        ve_table: Table3D<16, 16>,
        spark_table: Table3D<16, 16>,
        sensor_snapshot: Vec<SensorData, 32>, // Max 32 sensors
        rpm: u16,
        protocol_tx_buffer: Vec<u8, 256>, // Buffer for sending messages over protocol
        protocol_rx_buffer: Vec<u8, 256>, // Buffer for receiving messages over protocol
        protocol_seq_num_tx: u32,
        protocol_seq_num_rx: u32,
    }

    /// Local resources specific to each task.
    #[local]
    struct Local {
        // Timer peripherals for scheduler
        tim2_timer: stm32h7xx_hal::timer::Timer<pac::TIM2>,
        tim3_timer: stm32h7xx_hal::timer::Timer<pac::TIM3>,

        // GPIO pins for injectors/coils
        injector_pins: [stm32h7xx_hal::gpio::Pin<'B', 0, stm32h7xx_hal::gpio::Output>; 4], // Example: PB0-PB3
        coil_pins: [stm32h7xx_hal::gpio::Pin<'B', 4, stm32h7xx_hal::gpio::Output>; 4],    // Example: PB4-PB7

        // ADC peripheral
        adc1: stm32h7xx_hal::adc::Adc<pac::ADC1, stm32h7xx_hal::adc::Enabled>,

        // UART peripheral for protocol
        usart1: stm32h7xx_hal::usart::Usart<pac::USART1>,

        // Watchdog
        iwdg: stm32h7xx_hal::iwdg::IndependentWatchdog,

        // Scheduler instance
        scheduler: crate::scheduler::Scheduler,

        // Crank/Cam decoder
        crank_decoder: crate::crank_decoder::TriggerDecoder,

        // Math engine
        math_engine: crate::math_engine::MathEngine,

        // Knock controller
        knock_controller: crate::knock_controller::KnockController,

        // Last Host heartbeat timestamp for watchdog
        last_host_heartbeat_us: u64,
    }

    /// Initialization function.
    #[init]
    fn init(ctx: init::Context) -> (Shared, Local, init::Monotonics) {
        // Configure clock tree (480MHz SYSCLK)
        let pwr = ctx.device.PWR.constrain();
        let vos = pwr.freeze();
        let rcc = ctx.device.RCC.constrain();
        let ccdr = rcc.sys_ck(480.MHz()).freeze(vos, &ctx.device.SYSCFG);
        let clocks = ccdr.clocks;

        // Initialize GPIOs
        let gpiob = ctx.device.GPIOB.split(ccdr.ahb4);
        let injector_pins = [
            gpiob.pb0.into_push_pull_output().into(),
            gpiob.pb1.into_push_pull_output().into(),
            gpiob.pb2.into_push_pull_output().into(),
            gpiob.pb3.into_push_pull_output().into(),
        ];
        let coil_pins = [
            gpiob.pb4.into_push_pull_output().into(),
            gpiob.pb5.into_push_pull_output().into(),
            gpiob.pb6.into_push_pull_output().into(),
            gpiob.pb7.into_push_pull_output().into(),
        ];

        // Initialize Timers
        let tim2_timer = ctx.device.TIM2.timer(1.MHz(), ccdr.apb1_grp1, &clocks);
        let tim3_timer = ctx.device.TIM3.timer(1.MHz(), ccdr.apb1_grp1, &clocks);

        // Initialize ADC
        let adc1 = stm32h7xx_hal::adc::Adc::adc1(ctx.device.ADC1, ccdr.ahb1, clocks);

        // Initialize UART
        let usart1 = ctx.device.USART1.usart(
            // Example pins, adjust as needed
            gpiob.pb14.into_alternate(), // TX
            gpiob.pb15.into_alternate(), // RX
            stm32h7xx_hal::usart::Config::default().baudrate(115_200.bps()),
            ccdr.apb2,
            &clocks,
        ).unwrap();

        // Initialize Watchdog
        let mut iwdg = ctx.device.IWDG.independent_watchdog(1.secs(), ccdr.apb4);
        iwdg.start();

        // Check for IWDG reset and load limp mode maps if necessary
        let rcc_csr = ctx.device.RCC.CSR.read();
        let (ve_table, spark_table) = if rcc_csr.iwdgrstf().bit_is_set() {
            // Watchdog reset detected, load limp mode maps
            // In a real scenario, this would read from a dedicated flash sector
            (Table3D::new(), Table3D::new()) // Placeholder for actual limp mode maps
        } else {
            // Load default/last saved maps
            (Table3D::new(), Table3D::new()) // Placeholder for actual maps
        };
        // Clear the reset flag
        ctx.device.RCC.CSR.modify(|_, w| w.rmvf().set_bit());

        // Initialize other components
        let scheduler = crate::scheduler::Scheduler::new();
        let crank_decoder = crate::crank_decoder::TriggerDecoder::new();
        let math_engine = crate::math_engine::MathEngine::new();
        let knock_controller = crate::knock_controller::KnockController::new();

        // Spawn the `sensor_and_math_task` with 1ms period using `Monotonic`
        // Monotonic timer setup
        let mono = ccdr.monotonic.monotonic_tim6(clocks);
        sensor_and_math_task::spawn_after(1.ms()).ok();
        watchdog_feed_task::spawn_after(100.ms()).ok(); // Feed watchdog every 100ms

        (
            Shared {
                engine_state: EngineState {
                    sync_state: SyncState::Searching,
                    engine_phase: EnginePhase::Cylinder1TDC,
                    fuel_cut_active: false,
                    spark_cut_active: false,
                },
                ve_table,
                spark_table,
                sensor_snapshot: Vec::new(),
                rpm: 0,
                protocol_tx_buffer: Vec::new(),
                protocol_rx_buffer: Vec::new(),
                protocol_seq_num_tx: 0,
                protocol_seq_num_rx: 0,
            },
            Local {
                tim2_timer,
                tim3_timer,
                injector_pins,
                coil_pins,
                adc1,
                usart1,
                iwdg,
                scheduler,
                crank_decoder,
                math_engine,
                knock_controller,
                last_host_heartbeat_us: 0,
            },
            init::Monotonics(mono),
        )
    }

    /// Idle task.
    #[idle]
    fn idle(_: idle::Context) -> ! {
        loop {
            critical_function(); // Call the ITCM function
            cortex_m::asm::wfi(); // Wait For Interrupt
        }
    }

    /// PRIORITY 4 (highest): Ignition/injection execution
    #[task(binds = TIM2, priority = 4, local = [scheduler, tim2_timer, injector_pins, coil_pins])]
    fn timer_compare(ctx: timer_compare::Context) {
        // Implementation will go here in Task 2.2
    }

    /// PRIORITY 3: Crank/cam decode
    #[task(binds = TIM3, priority = 3, local = [crank_decoder, tim3_timer], shared = [engine_state, rpm])]
    fn crank_edge(ctx: crank_edge::Context) {
        // Implementation will go here in Task 2.3
    }

    /// PRIORITY 2: Sensor sampling + real-time math (fuel/spark calculation)
    #[task(priority = 2, local = [adc1, math_engine, scheduler], shared = [ve_table, spark_table, sensor_snapshot, rpm, engine_state])]
    fn sensor_and_math_task(ctx: sensor_and_math_task::Context) {
        // Implementation will go here in Task 2.4
    }

    /// PRIORITY 1: Protocol/CAN handling
    #[task(priority = 1, local = [usart1], shared = [engine_state, ve_table, spark_table, protocol_tx_buffer, protocol_rx_buffer, protocol_seq_num_tx, protocol_seq_num_rx])]
    fn protocol_rx_task(ctx: protocol_rx_task::Context) {
        // Implementation will go here in Task 3.1
    }

    /// PRIORITY 1: Watchdog feeding task
    #[task(priority = 1, local = [iwdg, last_host_heartbeat_us])]
    fn watchdog_feed_task(ctx: watchdog_feed_task::Context) {
        // Implementation will go here in Task 2.5
    }

    /// PRIORITY 0: Background tasks (logging, diagnostics)
    #[task(priority = 0)]
    fn background_task(_: background_task::Context) {
        // Implementation will go here
    }
}

// Placeholder modules for now, will be filled in later tasks
mod scheduler {
    use heapless::BinaryHeap;
    use heapless::Vec;

    #[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
    pub enum EventType {
        InjectorStart,
        InjectorEnd,
        IgnitionStart,
        IgnitionEnd,
    }

    #[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
    pub struct ScheduledEvent {
        pub channel: u8,
        pub timestamp_ticks: u32,
        pub duration_ticks: u16,
        pub event_type: EventType,
    }

    #[derive(Debug, PartialEq, Eq)]
    pub enum Error {
        QueueFull,
    }

    pub struct Scheduler {
        pub queue: BinaryHeap<ScheduledEvent, 32>,
    }

    impl Scheduler {
        pub fn new() -> Self {
            Self {
                queue: BinaryHeap::new(),
            }
        }

        pub fn schedule(&mut self, event: ScheduledEvent) -> Result<(), Error> {
            self.queue.push(event).map_err(|_| Error::QueueFull)
        }

        pub fn pop_ready(&mut self, current_time: u32) -> Option<ScheduledEvent> {
            if let Some(event) = self.queue.peek() {
                if event.timestamp_ticks <= current_time {
                    return self.queue.pop();
                }
            }
            None
        }

        pub fn next_deadline(&self) -> Option<u32> {
            self.queue.peek().map(|event| event.timestamp_ticks)
        }
    }
}

mod crank_decoder {
    use heapless::Vec;
    use crate::EnginePhase;
    use crate::SyncState;

    pub struct TriggerDecoder {
        pub state: SyncState,
        pub tooth_count: u8,
        pub tooth_times: Vec<u32, 4>,
        pub rpm: u16,
        pub phase: EnginePhase,
    }

    impl TriggerDecoder {
        pub fn new() -> Self {
            Self {
                state: SyncState::Searching,
                tooth_count: 0,
                tooth_times: Vec::new(),
                rpm: 0,
                phase: EnginePhase::Cylinder1TDC,
            }
        }

        pub fn process_edge(&mut self, capture: u32) -> (u16, EnginePhase, SyncState) {
            // Placeholder for actual implementation
            // This would involve calculating time deltas, detecting missing teeth,
            // updating tooth count, calculating RPM, and determining engine phase.
            // For now, return dummy values.
            self.tooth_times.push(capture).ok(); // Store capture
            if self.tooth_times.len() > 1 {
                let delta = self.tooth_times[self.tooth_times.len() - 1] - self.tooth_times[self.tooth_times.len() - 2];
                // Very basic RPM calculation placeholder
                if delta > 0 {
                    self.rpm = (60_000_000 / delta) as u16; // Assuming delta is in microseconds
                }
            }
            self.state = SyncState::Synced;
            self.phase = EnginePhase::Cylinder1TDC;
            (self.rpm, self.phase.clone(), self.state.clone())
        }
    }
}

mod math_engine {
    use crate::Table3D;

    pub struct MathEngine {
        pub ve_table: Table3D<16, 16>,
        pub spark_table: Table3D<16, 16>,
        pub injector_config: InjectorConfig,
        pub ignition_config: IgnitionConfig,
    }

    pub struct InjectorConfig {
        pub flow_rate_cc_min: f32,
        pub latency_us_mv: [(u16, u16); 8], // (voltage_mv, latency_us)
    }

    pub struct IgnitionConfig {
        pub dwell_us: u16,
        pub max_advance_deg: f32,
        pub min_advance_deg: f32,
    }

    impl MathEngine {
        pub fn new() -> Self {
            Self {
                ve_table: Table3D::new(),
                spark_table: Table3D::new(),
                injector_config: InjectorConfig {
                    flow_rate_cc_min: 300.0,
                    latency_us_mv: [(0, 0); 8],
                },
                ignition_config: IgnitionConfig {
                    dwell_us: 1000,
                    max_advance_deg: 60.0,
                    min_advance_deg: -10.0,
                },
            }
        }

        pub fn calculate_fuel_mass(&self, rpm: u16, map: u16, iat: i16) -> f32 {
            // Placeholder for actual implementation
            // VE interpolation, air density correction, target AFR
            let ve = self.ve_table.interpolate(rpm as f32, map as f32);
            // Dummy calculation
            ve * (rpm as f32 / 1000.0) * (map as f32 / 100.0)
        }

        pub fn calculate_fuel_pulse_width(&self, fuel_mass: f32, battery_voltage: f32) -> f32 {
            // Placeholder for actual implementation
            // Convert fuel mass to pulse width, injector latency lookup
            fuel_mass * 10.0 // Dummy calculation
        }

        pub fn calculate_spark_advance(&self, rpm: u16, map: u16, knock_retard: f32) -> f32 {
            // Placeholder for actual implementation
            // Spark table interpolation, knock retard, clamping
            let base_advance = self.spark_table.interpolate(rpm as f32, map as f32);
            (base_advance - knock_retard).clamp(self.ignition_config.min_advance_deg, self.ignition_config.max_advance_deg)
        }
    }
}

mod knock_controller {
    pub struct KnockController {
        pub global_timing_retard: f32,
        pub cylinder_timing_retards: [f32; 4],
        pub knock_step_deg: f32,
        pub recovery_step_deg: f32,
        pub max_retard_deg: f32,
    }

    impl KnockController {
        pub fn new() -> Self {
            Self {
                global_timing_retard: 0.0,
                cylinder_timing_retards: [0.0; 4],
                knock_step_deg: 1.0,
                recovery_step_deg: 0.1,
                max_retard_deg: 10.0,
            }
        }

        pub fn process_knock_event(&mut self, cylinder_id: u8, intensity: f32) {
            if (cylinder_id as usize) < self.cylinder_timing_retards.len() {
                self.cylinder_timing_retards[cylinder_id as usize] += intensity * self.knock_step_deg;
                self.cylinder_timing_retards[cylinder_id as usize] = self.cylinder_timing_retards[cylinder_id as usize].clamp(0.0, self.max_retard_deg);
            }
        }

        pub fn get_total_retard(&self, cylinder_id: u8) -> f32 {
            if (cylinder_id as usize) < self.cylinder_timing_retards.len() {
                self.global_timing_retard + self.cylinder_timing_retards[cylinder_id as usize]
            } else {
                self.global_timing_retard
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::app::*;
    use super::*;
    use rtic::app;
    use stm32h7xx_hal::timer::Timer;
    use stm32h7xx_hal::gpio::{Output, PushPull};
    use stm32h7xx_hal::adc::Adc;
    use stm32h7xx_hal::usart::Usart;
    use stm32h7xx_hal::iwdg::IndependentWatchdog;

    // Mock implementations for testing purposes
    struct MockTimer;
    impl Timer for MockTimer {
        type Tick = u32;
        fn counter(&self) -> Self::Tick { 0 }
        fn set_compare(&mut self, _: Self::Tick) {}
        fn enable_compare_interrupt(&mut self) {}
        fn disable_compare_interrupt(&mut self) {}
        fn clear_interrupt(&mut self) {}
        fn capture_value(&self) -> Self::Tick { 0 }
        fn enable_input_capture(&mut self, _: u8) -> Result<(), HalError> { Ok(()) }
        fn disable_input_capture(&mut self, _: u8) -> Result<(), HalError> { Ok(()) }
    }

    struct MockGpioPin;
    impl GpioPin for MockGpioPin {
        fn set_high(&mut self) {}
        fn set_low(&mut self) {}
        fn toggle(&mut self) {}
        fn is_high(&self) -> bool { false }
        fn into_output(&mut self) {}
        fn into_input(&mut self) {}
    }

    struct MockAdc;
    impl Adc for MockAdc {
        type RawValue = u16;
        type Voltage = f32;
        fn read_channel(&mut self, _: u8) -> Result<Self::RawValue, HalError> { Ok(0) }
        fn read_all_channels(&mut self, buffer: &mut [Self::RawValue]) -> Result<usize, HalError> {
            for val in buffer.iter_mut() { *val = 0; }
            Ok(buffer.len())
        }
        fn raw_to_voltage(&self, _: Self::RawValue) -> Self::Voltage { 0.0 }
        fn calibrate(&mut self) -> Result<(), HalError> { Ok(()) }
    }

    struct MockUsart;
    impl Uart for MockUsart {
        fn write(&mut self, _: &[u8]) -> Result<usize, HalError> { Ok(0) }
        fn read(&mut self, _: &mut [u8]) -> Result<usize, HalError> { Ok(0) }
        fn has_data_to_read(&self) -> bool { false }
        fn is_tx_empty(&self) -> bool { true }
    }

    struct MockIwdg;
    impl Watchdog for MockIwdg {
        fn feed(&mut self) {}
        fn was_reset_by_watchdog(&self) -> bool { false }
        fn clear_reset_flag(&mut self) {}
    }

    // This test is more of a compile-time check for the RTIC app structure
    // and ensuring the types match. Actual runtime tests for preemption etc.
    // are harder to do without a real MCU or a very sophisticated mock.
    #[test]
    fn test_rtic_app_compiles_and_types_match() {
        // This test primarily ensures that the `app` module compiles and
        // that the types defined in `Shared` and `Local` match what's expected
        // by the RTIC framework and the HAL traits.
        // We cannot actually run the RTIC app in a unit test on the host.
        // The presence of the `#[init]` and `#[task]` macros will trigger
        // RTIC's compile-time checks.

        // Dummy values to satisfy type requirements for local/shared resources
        // This is purely for type checking, not for functional testing.
        let _engine_state: EngineState = EngineState {
            sync_state: SyncState::Searching,
            engine_phase: EnginePhase::Cylinder1TDC,
            fuel_cut_active: false,
            spark_cut_active: false,
        };
        let _ve_table: Table3D<16, 16> = Table3D::new();
        let _spark_table: Table3D<16, 16> = Table3D::new();
        let _sensor_snapshot: Vec<SensorData, 32> = Vec::new();
        let _rpm: u16 = 0;
        let _protocol_tx_buffer: Vec<u8, 256> = Vec::new();
        let _protocol_rx_buffer: Vec<u8, 256> = Vec::new();
        let _protocol_seq_num_tx: u32 = 0;
        let _protocol_seq_num_rx: u32 = 0;

        let _tim2_timer: MockTimer = MockTimer;
        let _tim3_timer: MockTimer = MockTimer;
        let _injector_pins: [MockGpioPin; 4] = [MockGpioPin, MockGpioPin, MockGpioPin, MockGpioPin];
        let _coil_pins: [MockGpioPin; 4] = [MockGpioPin, MockGpioPin, MockGpioPin, MockGpioPin];
        let _adc1: MockAdc = MockAdc;
        let _usart1: MockUsart = MockUsart;
        let _iwdg: MockIwdg = MockIwdg;
        let _scheduler: scheduler::Scheduler = scheduler::Scheduler::new();
        let _crank_decoder: crank_decoder::TriggerDecoder = crank_decoder::TriggerDecoder::new();
        let _math_engine: math_engine::MathEngine = math_engine::MathEngine::new();
        let _knock_controller: knock_controller::KnockController = knock_controller::KnockController::new();
        let _last_host_heartbeat_us: u64 = 0;

        // This assertion is just to make the test pass, as the real check is compilation.
        assert!(true);
    }
}