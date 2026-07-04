use embassy_stm32::usart::Uart;
use embassy_usb::class::cdc_acm::CdcAcmClass;
use embassy_usb::driver::Driver;
use oxide_hal::transport::Transport;

impl<'d, T, P, R> Transport for Uart<'d, T, P, R> {}

impl<'d, D: Driver<'d>> Transport for CdcAcmClass<'d, D> {}

#[cfg(test)]
mod tests {
    use super::*;
    use heapless::spsc::Queue;
    use embedded_io_async::{Read, Write};

    #[tokio::test]
    async fn test_ring_buffer() {
        let mut q: Queue<u8, 1024> = Queue::new();
        let (mut p, mut c) = q.split();

        // Simulate writing at max speed
        for i in 0..1024 {
            p.enqueue(i as u8).unwrap();
        }
        assert!(p.enqueue(0).is_err()); // Queue is full

        // Simulate reading at max speed
        for i in 0..1024 {
            assert_eq!(c.dequeue(), Some(i as u8));
        }
        assert!(c.dequeue().is_none()); // Queue is empty
    }
}
