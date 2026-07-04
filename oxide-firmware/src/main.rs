#![no_main]
#![no_std]

use panic_halt as _;

mod scheduler;
mod trigger_decoder;

#[rtic::app(device = stm32h7xx_hal::pac, dispatchers = [SPI1, SPI2])]
mod app {
    use cortex_m_rt::entry;
    use stm32h7xx_hal::{pac, prelude::*, timer::Timer};
    use oxide_hal::{EngineTimer, Adc, Pwm, CanFd, ExternalFlash, Transport};
    use oxide_math::Table3D;
    use heapless::Vec;
    use crate::scheduler::{AngularScheduler, ScheduledEvent, EventType};
    use crate::trigger_decoder::{TriggerDecoder, SyncState};

    #[shared]
    struct Shared {
        engine_state: SyncState,
        ve_table: Table3D<f32, 16, 16>,
        spark_table: Table3D<f32, 16, 16>,
        rpm: u16,
        angle: u32,
    }

    #[local]
    struct Local {
        timer: Timer<pac::TIM2>,
        gpio: pac::GPIOB,
        scheduler: AngularScheduler,
        trigger_decoder: TriggerDecoder,
    }

    #[init]
    fn init(ctx: init::Context) -> (Shared, Local, init::Monotonics) {
        let pwr = ctx.device.PWR.constrain();
        let vos = pwr.freeze();
        let rcc = ctx.device.RCC.constrain();
        let ccdr = rcc.sys_ck(400.MHz()).freeze(vos, &ctx.device.SYSCFG);

        let timer = ctx.device.TIM2.timer(1.MHz(), ccdr.peripheral.TIM2, &ccdr.clocks);

        (
            Shared {
                engine_state: SyncState::NoSignal,
                ve_table: Table3D::new_from_data([[0.0; 16]; 16]),
                spark_table: Table3D::new_from_data([[0.0; 16]; 16]),
                rpm: 0,
                angle: 0,
            },
            Local {
                timer,
                gpio: ctx.device.GPIOB,
                scheduler: AngularScheduler::new(),
                trigger_decoder: TriggerDecoder::new(),
            },
            init::Monotonics(),
        )
    }

    #[idle]
    fn idle(_: idle::Context) -> ! {
        loop {
            cortex_m::asm::wfi();
        }
    }

    #[task(binds = TIM2, priority = 4, local = [timer, gpio, scheduler])]
    fn timer_compare(ctx: timer_compare::Context) {
        let angle = 0; // Will be updated from a timer
        if let Some(event) = ctx.local.scheduler.pop_ready(angle) {
            let pin = match event.channel {
                0 => &mut ctx.local.gpio.bsrr,
                _ => return,
            };
            match event.event_type {
                EventType::Ignition => {
                    pin.write(|w| w.bs0().set_bit());
                    // Schedule the end of the event
                }
                EventType::Injection => {
                    pin.write(|w| w.bs1().set_bit());
                    // Schedule the end of the event
                }
            }
        }
    }

    #[task(binds = TIM3_CH1, priority = 3, local = [trigger_decoder], shared = [rpm, angle, engine_state])]
    fn crank_edge(mut ctx: crank_edge::Context) {
        let timestamp = 0; // Will be updated from a timer
        let (rpm, angle, state) = ctx.local.trigger_decoder.handle_interrupt_pulse(timestamp);
        ctx.shared.rpm.lock(|r| *r = rpm);
        ctx.shared.angle.lock(|a| *a = angle);
        ctx.shared.engine_state.lock(|s| *s = state);
    }

    #[task(priority = 2)]
    fn sensor_and_math_task(_: sensor_and_math_task::Context) {}

    #[task(priority = 1)]
    fn protocol_rx_task(_: protocol_rx_task::Context) {}
}
