#![no_main]
#![no_std]

use panic_halt as _;

#[rtic::app(device = stm32h7xx_hal::pac, dispatchers = [SPI1, SPI2])]
mod app {
    use cortex_m_rt::entry;
    use stm32h7xx_hal::{pac, prelude::*};
    use oxide_hal::{EngineTimer, Adc, Pwm, CanFd, ExternalFlash, Transport};
    use oxide_math::Table3D;
    use heapless::Vec;

    #[shared]
    struct Shared {
        engine_state: (),
        ve_table: Table3D<f32, 16, 16>,
        spark_table: Table3D<f32, 16, 16>,
    }

    #[local]
    struct Local {}

    #[init]
    fn init(ctx: init::Context) -> (Shared, Local, init::Monotonics) {
        let pwr = ctx.device.PWR.constrain();
        let vos = pwr.freeze();
        let rcc = ctx.device.RCC.constrain();
        let ccdr = rcc.sys_ck(400.MHz()).freeze(vos, &ctx.device.SYSCFG);

        (
            Shared {
                engine_state: (),
                ve_table: Table3D::new_from_data([[0.0; 16]; 16]),
                spark_table: Table3D::new_from_data([[0.0; 16]; 16]),
            },
            Local {},
            init::Monotonics(),
        )
    }

    #[idle]
    fn idle(_: idle::Context) -> ! {
        loop {
            cortex_m::asm::wfi();
        }
    }

    #[task(binds = TIM2, priority = 4)]
    fn timer_compare(_: timer_compare::Context) {}

    #[task(binds = TIM3_CH1, priority = 3)]
    fn crank_edge(_: crank_edge::Context) {}

    #[task(priority = 2)]
    fn sensor_and_math_task(_: sensor_and_math_task::Context) {}

    #[task(priority = 1)]
    fn protocol_rx_task(_: protocol_rx_task::Context) {}
}
