
// This file will define the RTIC application and abstract the interrupt bindings.

// --- Conditional Interrupt Mapping ---

#[cfg(feature = "stm32h7")]
use stm32h7xx_hal::pac::interrupt::{TIM2 as ENGINE_TIMER_IRQ, FDCAN1_IT0 as CAN_IRQ};

#[cfg(feature = "nrf91")]
use nrf9160_hal::pac::interrupt::{TIMER0 as ENGINE_TIMER_IRQ, SPIM0_SPIS0_TWIM0_TWIS0_UARTE0 as CAN_IRQ}; // Placeholder for CAN

#[cfg(feature = "lpc55")]
use lpc55_hal::pac::interrupt::{CTIMER0 as ENGINE_TIMER_IRQ, CAN0 as CAN_IRQ};

#[cfg(feature = "renesas_ra8")]
use renesas_ra::pac::interrupt::{GPT0 as ENGINE_TIMER_IRQ, SCI0_RXI as CAN_IRQ}; // Placeholder for CAN


// --- RTIC Application ---

#[rtic::app(
    // The device PAC will be selected based on the feature flag in the main lib.rs or app.rs
    // For example: #[cfg_attr(feature = "stm32h7", app(device = stm32h7xx_hal::pac, ...))]
    // This part needs to be handled where the app is actually declared.
    // For now, we'll just define the tasks.
)]
mod app {
    use super::*;
    use crate::scheduler::MainScheduler;
    use oxide_hal::EngineTimer;

    #[shared]
    struct Shared {
        // Shared resources
    }

    #[local]
    struct Local {
        scheduler: MainScheduler,
    }

    #[init]
    fn init(ctx: init::Context) -> (Shared, Local) {
        // Initialization code
        (Shared {}, Local { scheduler: MainScheduler::new() })
    }

    #[task(binds = ENGINE_TIMER_IRQ, priority = 4, local = [scheduler])]
    fn timer_compare(mut ctx: timer_compare::Context) {
        // Engine timer interrupt handler
        // This task is now portable across MCUs.
        ctx.local.scheduler.on_timer_tick();
    }

    #[task(binds = CAN_IRQ, priority = 3)]
    fn can_receive(ctx: can_receive::Context) {
        // CAN bus receive interrupt handler
    }
}
