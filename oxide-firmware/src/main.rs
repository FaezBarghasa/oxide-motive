#![no_std]
#![no_main]

use panic_halt as _;

mod scheduler;
mod trigger_decoder;
mod math_engine;

// This is a placeholder and will not compile without a concrete HAL implementation
// and feature flag. The `check.sh` script will verify this.
#[cfg(feature = "stm32h7")]
#[rtic::app(device = stm32h7xx_hal::pac, dispatchers = [SPI1, SPI2, SPI3])]
mod app {
    use crate::scheduler::{Scheduler, ScheduledEvent, EventType};
    use crate::trigger_decoder::{TriggerDecoder, EnginePhase};
    use crate::math_engine::{MathEngine, InjectorConfig, IgnitionConfig};
    use oxide_math::Table3D;
    use oxide_protocol::EngineState;

    // Placeholder for sensor data
    pub struct SensorSnapshot {
        pub rpm: u16,
        pub map: u16,
        pub iat: i16,
        pub battery_voltage: f32,
        pub knock_retard: f32,
        pub crank_angle: f32,
    }

    #[shared]
    struct Shared {
       engine_state: EngineState,
       ve_table: Table3D<16, 16>,
       spark_table: Table3D<16, 16>,
       sensor_snapshot: SensorSnapshot,
       rpm: u16,
    }

    #[local]
    struct Local {
        scheduler: Scheduler<32>,
        trigger_decoder: TriggerDecoder<36>,
        math_engine: MathEngine,
    }

    #[init]
    fn init(_: init::Context) -> (Shared, Local) {
        let ve_table = Table3D::new();
        let spark_table = Table3D::new();
        let injector_config = InjectorConfig { flow_rate_cc_min: 550.0, dead_time_ms: 0.8 };
        let ignition_config = IgnitionConfig { dwell_time_ms: 2.5 };

        (
            Shared {
                engine_state: EngineState::Offline,
                ve_table,
                spark_table,
                sensor_snapshot: SensorSnapshot { rpm: 0, map: 100, iat: 25, battery_voltage: 12.0, knock_retard: 0.0, crank_angle: 0.0 },
                rpm: 0,
            },
            Local {
                scheduler: Scheduler::new(),
                trigger_decoder: TriggerDecoder::new(),
                math_engine: MathEngine::new(ve_table, spark_table, injector_config, ignition_config),
            },
        )
    }

    #[idle]
    fn idle(_: idle::Context) -> ! {
        loop {
            cortex_m::asm::wfi();
        }
    }

    // PRIORITY 4 (highest): Ignition/injection execution
    #[task(binds = TIM2, priority = 4, local = [scheduler])]
    fn timer_compare(ctx: timer_compare::Context) {
        let current_time = 0; // Placeholder for timer read
        while let Some(event) = ctx.local.scheduler.pop_ready(current_time) {
            // Actuate GPIOs based on event
        }
        if let Some(next) = ctx.local.scheduler.next_deadline() {
            // Set next timer compare value
        }
    }

    // PRIORITY 3: Crank/cam decode
    #[task(binds = TIM3, priority = 3, local = [trigger_decoder], shared = [engine_state, rpm])]
    fn crank_edge(mut ctx: crank_edge::Context) {
        let capture = 0; // Placeholder
        let (rpm, _phase, sync_state) = ctx.local.trigger_decoder.process_edge(capture);

        ctx.shared.engine_state.lock(|s| *s = if sync_state == crate::trigger_decoder::SyncState::Synced { EngineState::Running } else { EngineState::Cranking });
        ctx.shared.rpm.lock(|r| *r = rpm);
    }

    // PRIORITY 2: Sensor sampling + real-time math (fuel/spark calculation)
    #[task(priority = 2, local=[scheduler, math_engine], shared=[sensor_snapshot])]
    fn sensor_and_math_task(mut ctx: sensor_and_math_task::Context) {
        ctx.shared.sensor_snapshot.lock(|sensors| {
            let fuel_pw = ctx.local.math_engine.calculate_fuel_pulse_width(
                ctx.local.math_engine.calculate_fuel_mass(sensors.rpm, sensors.map, sensors.iat),
                sensors.battery_voltage,
            );
            let spark_advance = ctx.local.math_engine.calculate_spark_advance(
                sensors.rpm,
                sensors.map,
                sensors.knock_retard,
            );

            let event = ScheduledEvent {
                channel: 0,
                timestamp_ticks: 12345,
                duration_ticks: fuel_pw as u16,
                event_type: EventType::InjectorStart,
            };
            ctx.local.scheduler.schedule(event).ok();
        });
    }

    // PRIORITY 1: Protocol/CAN handling
    #[task(priority = 1)]
    async fn protocol_rx_task(_: protocol_rx_task::Context) { }

    // PRIORITY 0: Background tasks (logging, diagnostics)
    #[task(priority = 0)]
    async fn background_task(_: background_task::Context) { }
}

// Dummy main for when no feature is selected
#[cfg(not(feature = "stm32h7"))]
fn main() {}
