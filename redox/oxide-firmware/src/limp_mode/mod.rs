use embassy_stm32::wdg::IndependentWatchdog;
use embassy_sync::{blocking_mutex::raw::ThreadModeRawMutex, channel::Channel};
use embassy_time::{Duration, Timer};

pub static HEARTBEAT_CHANNEL: Channel<ThreadModeRawMutex, (), 1> = Channel::new();

#[embassy_executor::task]
pub async fn watchdog_feed_task(mut watchdog: IndependentWatchdog<'static, embassy_stm32::peripherals::IWDG>) {
    loop {
        match embassy_time::with_timeout(Duration::from_millis(800), HEARTBEAT_CHANNEL.receive()).await {
            Ok(_) => watchdog.unleash(),
            Err(_) => {
                // Stop feeding the watchdog, causing a reset
                break;
            }
        }
    }
}

pub fn is_iwdg_reset(rcc: &embassy_stm32::pac::RCC) -> bool {
    rcc.bdcr().read().bdrstf()
}

pub fn get_limp_fuel_map(rpm: u32, map: u32) -> u32 {
    // Simple fallback map
    if rpm < 1000 {
        1000
    } else if rpm < 3000 {
        2000
    } else {
        3000
    }
}
