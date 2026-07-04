//! DMA-driven, non-blocking UART transport implementation.

use stm32h7xx_hal::{
    pac,
    prelude::*,
    dma::{self, Dma, DmaStream, Transfer, MemoryToPeripheral, PeripheralToMemory, FifoThreshold},
    serial::{self, Serial, Tx, Rx},
};
use crate::hal::Transport;
use heapless::spsc::{Queue, Producer, Consumer};

pub enum SerialError {
    Dma(dma::Error),
    Serial(serial::Error),
    BufferFull,
    BufferEmpty,
}

pub struct DmaSerialTransport {
    tx_dma: DmaStream<pac::DMA1, dma::stream::Stream0>,
    rx_dma: DmaStream<pac::DMA1, dma::stream::Stream1>,
    tx_usart: Tx<pac::USART1>,
    rx_usart: Rx<pac::USART1>,
    tx_producer: Producer<'static, u8, 1024>,
    rx_consumer: Consumer<'static, u8, 1024>,
}

impl DmaSerialTransport {
    pub fn new(
        usart: Serial<pac::USART1>,
        dma: Dma<pac::DMA1>,
        tx_p: Producer<'static, u8, 1024>,
        rx_c: Consumer<'static, u8, 1024>,
    ) -> Self {
        let (tx_usart, rx_usart) = usart.split();
        let dma_streams = dma.streams;

        Self {
            tx_dma: dma_streams.stream0,
            rx_dma: dma_streams.stream1,
            tx_usart,
            rx_usart,
            tx_producer: tx_p,
            rx_consumer: rx_c,
        }
    }

    // This would be called in an interrupt to handle DMA completion
    pub fn handle_tx_dma_interrupt(&mut self) {
        // Refill DMA buffer from the tx queue
    }

    pub fn handle_rx_dma_interrupt(&mut self) {
        // Move data from DMA buffer to the rx queue
    }
}

impl Transport for DmaSerialTransport {
    type Error = SerialError;

    fn send_non_blocking(&mut self, data: &[u8]) -> Result<usize, Self::Error> {
        let mut bytes_written = 0;
        for &byte in data {
            if self.tx_producer.enqueue(byte).is_err() {
                return Err(SerialError::BufferFull);
            }
            bytes_written += 1;
        }
        // In a real implementation, we would trigger the DMA transfer here if not already running.
        Ok(bytes_written)
    }

    fn receive_non_blocking(&mut self, buf: &mut [u8]) -> Result<usize, Self::Error> {
        let mut bytes_read = 0;
        for byte in buf.iter_mut() {
            if let Some(b) = self.rx_consumer.dequeue() {
                *byte = b;
                bytes_read += 1;
            } else {
                break;
            }
        }
        Ok(bytes_read)
    }

    fn flush(&mut self) -> Result<(), Self::Error> {
        // This would wait for the DMA transfer to complete.
        // For this mock, we assume it completes instantly.
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use heapless::spsc::Queue;

    #[test]
    fn test_send_and_receive() {
        static mut TX_Q: Queue<u8, 1024> = Queue::new();
        static mut RX_Q: Queue<u8, 1024> = Queue::new();
        let (tx_p, mut tx_c) = unsafe { TX_Q.split() };
        let (mut rx_p, rx_c) = unsafe { RX_Q.split() };

        // Mock the DMA by manually moving data between queues
        let mock_dma_transfer = |tx_consumer: &mut Consumer<u8, 1024>, rx_producer: &mut Producer<u8, 1024>| {
            while let Some(byte) = tx_consumer.dequeue() {
                rx_producer.enqueue(byte).unwrap();
            }
        };

        let mut transport = DmaSerialTransport {
            // Dummy peripherals
            tx_dma: unsafe { core::mem::zeroed() },
            rx_dma: unsafe { core::mem::zeroed() },
            tx_usart: unsafe { core::mem::zeroed() },
            rx_usart: unsafe { core::mem::zeroed() },
            tx_producer: tx_p,
            rx_consumer: rx_c,
        };

        let send_data = b"hello world";
        transport.send_non_blocking(send_data).unwrap();

        // Simulate the transfer
        mock_dma_transfer(&mut tx_c, &mut rx_p);

        let mut recv_buf = [0u8; 32];
        let bytes_read = transport.receive_non_blocking(&mut recv_buf).unwrap();

        assert_eq!(bytes_read, send_data.len());
        assert_eq!(&recv_buf[..bytes_read], send_data);
    }
}
