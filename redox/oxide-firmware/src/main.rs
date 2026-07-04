#![no_std]
#![no_main]

use embassy_executor::Spawner;
use embassy_stm32::Config;
use embassy_stm32::adc::Adc;
use embassy_stm32::usart::{Uart, Config as UartConfig};
use embassy_stm32::time::mhz;
use {defmt_rtt as _, panic_probe as _};

mod hardware_config;
mod trigger_decoder;
mod transport;
mod scheduler;
mod io;

#[embassy_executor::main]
async fn main(spawner: Spawner) {
    let mut config = Config::default();
    config.rcc.sys = embassy_stm32::rcc::Sysclk::PLL1_P;
    let p = embassy_stm32::init(config);

    let mut adc = Adc::new(p.ADC1);
    
    let mut uart_config = UartConfig::default();
    uart_config.baudrate = 115200;
    let uart = Uart::new(p.USART1, p.PA10, p.PA9, p.DMA1_CH1, p.DMA1_CH2, uart_config).unwrap();
    let (tx, rx) = uart.split();
    
    spawner.spawn(io::adc_task(adc)).unwrap();
    spawner.spawn(transport::transport_rx_task(rx)).unwrap();
    spawner.spawn(transport::transport_tx_task(tx)).unwrap();
    spawner.spawn(scheduler::scheduler_manager_task()).unwrap();
}
