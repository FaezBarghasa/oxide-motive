//! Abstract traits for high-bandwidth, non-blocking communication transports.

/// A generic interface for a communication channel (e.g., UART, USB CDC).
/// Implementations should be non-blocking and suitable for use in interrupt contexts
/// or high-frequency polling loops.
pub trait Transport {
    /// The error type for transport operations.
    type Error;

    /// Sends data over the transport in a non-blocking manner.
    ///
    /// # Arguments
    /// * `data` - The byte slice to send.
    ///
    /// # Returns
    /// The number of bytes successfully written to the transport's buffer.
    /// This may be less than `data.len()` if the buffer is full.
    fn send_non_blocking(&mut self, data: &[u8]) -> Result<usize, Self::Error>;

    /// Receives data from the transport in a non-blocking manner.
    ///
    /// # Arguments
    /// * `buf` - The buffer to store the received data into.
    ///
    /// # Returns
    /// The number of bytes read from the transport's buffer.
    fn receive_non_blocking(&mut self, buf: &mut [u8]) -> Result<usize, Self::Error>;

    /// Flushes any buffered data, ensuring it is sent.
    /// For some transports (like DMA-based UART), this might involve waiting for
    /// a transfer to complete.
    fn flush(&mut self) -> Result<(), Self::Error>;
}
