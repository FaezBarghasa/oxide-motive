#![no_std]
#![no_main]

use embassy_executor::Spawner;
use embassy_stm32::Config;
use {defmt_rtt as _, panic_probe as _};

mod hardware_config;

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let mut config = Config::default();
    // Configure clocks here
    let _p = embassy_stm32::init(config);

    // TODO: Implement HAL traits and spawn tasks
}
