use embassy_stm32::usart::Uart;
use embassy_sync::{
    blocking_mutex::raw::ThreadModeRawMutex,
    channel::Channel,
};
use oxide_protocol::{HostToMcu, McuToHost};

pub struct UartTransport<'d> {
    _uart: Uart<'d, embassy_stm32::peripherals::USART1>,
}

pub static HOST_TO_MCU_CHANNEL: Channel<ThreadModeRawMutex, HostToMcu, 16> = Channel::new();
pub static MCU_TO_HOST_CHANNEL: Channel<ThreadModeRawMutex, McuToHost, 16> = Channel::new();

#[embassy_executor::task]
pub async fn transport_rx_task() {
    // TODO: Implement DMA ring buffer reading and COBS decoding
}

#[embassy_executor::task]
pub async fn transport_tx_task() {
    // TODO: Implement COBS encoding and DMA writing
}
