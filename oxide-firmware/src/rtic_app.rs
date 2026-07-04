
#[cfg_attr(feature = "stm32h7", rtic::app(device = stm32h7xx_hal::pac, dispatchers = [SPI1, SPI2]))]
#[cfg_attr(feature = "nrf91", rtic::app(device = nrf9160_hal::pac, dispatchers = [SPIM0_SPIS0_TWIM0_TWIS0_UART0, SPIM1_SPIS1_TWIM1_TWIS1_UART1]))]
#[cfg_attr(feature = "lpc55", rtic::app(device = lpc55_hal::pac, dispatchers = [USB0, USB1]))]
mod app {
    use cortex_m::asm;
    use oxide_hal::EngineTimer;

    #[cfg(feature = "stm32h7")]
    use stm32h7xx_hal::{pac, prelude::*};
    #[cfg(feature = "stm32h7")]
    use crate::hal_impl::stm32::Stm32EngineTimer as EngineTimerImpl;
    #[cfg(feature = "stm32h7")]
    const ENGINE_TIMER_IRQ: pac::Interrupt = pac::Interrupt::TIM2;

    #[cfg(feature = "nrf91")]
    use nrf9160_hal::{pac, prelude::*};
    #[cfg(feature = "nrf91")]
    use crate::hal_impl::nordic::NordicEngineTimer as EngineTimerImpl;
    #[cfg(feature = "nrf91")]
    const ENGINE_TIMER_IRQ: pac::Interrupt = pac::Interrupt::TIMER0;

    #[cfg(feature = "lpc55")]
    use lpc55_hal::{pac, prelude::*};
    #[cfg(feature = "lpc55")]
    use crate::hal_impl::nxp::NxpEngineTimer as EngineTimerImpl;
    #[cfg(feature = "lpc55")]
    const ENGINE_TIMER_IRQ: pac::Interrupt = pac::Interrupt::CTIMER0;

    #[shared]
    struct Shared {}

    #[local]
    struct Local {
        timer: EngineTimerImpl,
    }

    #[init]
    fn init(cx: init::Context) -> (Shared, Local, init::Monotonics) {
        let mut core = cx.core;
        let device = cx.device;

        #[cfg(feature = "stm32h7")]
        let (timer, ..) = {
            let pwr = device.PWR.constrain();
            let vos = pwr.freeze();
            let rcc = device.RCC.constrain();
            let ccdr = rcc.sys_ck(100.MHz()).freeze(vos, &device.SYSCFG);
            (Stm32EngineTimer::new(device.TIM2, &ccdr.clocks), ccdr)
        };

        #[cfg(not(feature = "stm32h7"))]
        let timer = EngineTimerImpl;


        (Shared {}, Local { timer }, init::Monotonics())
    }

    #[idle]
    fn idle(_: idle::Context) -> ! {
        loop {
            asm::wfi();
        }
    }

    #[task(binds = ENGINE_TIMER_IRQ, local = [timer])]
    fn timer_compare(mut cx: timer_compare::Context) {
        cx.local.timer.clear_interrupt(0);
    }
}
