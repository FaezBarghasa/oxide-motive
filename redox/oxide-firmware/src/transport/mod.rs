use embassy_stm32::usart::{UartRx, UartTx};
use embassy_stm32::mode::Async;
use embassy_sync::{
    blocking_mutex::raw::ThreadModeRawMutex,
    channel::Channel,
};
use oxide_protocol::{HostToMcu, McuToHost};
use postcard;

pub struct UartTransport<'d> {
    _uart: embassy_stm32::usart::Uart<'d, embassy_stm32::peripherals::USART1, Async>,
}

pub static HOST_TO_MCU_CHANNEL: Channel<ThreadModeRawMutex, HostToMcu, 16> = Channel::new();
pub static MCU_TO_HOST_CHANNEL: Channel<ThreadModeRawMutex, McuToHost, 16> = Channel::new();

#[embassy_executor::task]
pub async fn transport_rx_task(mut rx: UartRx<'static, embassy_stm32::peripherals::USART1, Async>) {
    let mut buf = [0u8; 256];
    let mut idx = 0;
    loop {
        let mut byte = [0u8; 1];
        if rx.read(&mut byte).await.is_ok() {
            buf[idx] = byte[0];
            idx += 1;
            if byte[0] == 0 {
                if let Ok(msg) = postcard::from_bytes_cobs::<HostToMcu>(&mut buf[..idx]) {
                    let _ = HOST_TO_MCU_CHANNEL.send(msg).await;
                }
                idx = 0;
            }
            if idx >= buf.len() { idx = 0; }
        }
    }
}

#[embassy_executor::task]
pub async fn transport_tx_task(mut tx: UartTx<'static, embassy_stm32::peripherals::USART1, Async>) {
    let mut buf = [0u8; 256];
    loop {
        let msg = MCU_TO_HOST_CHANNEL.receive().await;
        if let Ok(encoded) = postcard::to_slice_cobs(&msg, &mut buf) {
            let _ = tx.write(encoded).await;
        }
    }
}
