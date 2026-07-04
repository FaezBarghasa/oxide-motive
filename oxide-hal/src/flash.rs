//! Abstract trait for external flash memory devices.

/// A generic interface for external flash memory chips (like QSPI or SPI flash).
pub trait ExternalFlash {
    /// The error type for flash operations.
    type Error;

    /// Initializes the flash memory device.
    /// This typically involves configuring the peripheral, setting up DMA,
    /// and sending initial commands to the flash chip.
    fn init(&mut self) -> Result<(), Self::Error>;

    /// Reads a block of data from the specified address.
    ///
    /// # Arguments
    /// * `addr` - The starting address to read from.
    /// * `buf` - The buffer to store the read data into.
    fn read(&mut self, addr: u32, buf: &mut [u8]) -> Result<(), Self::Error>;

    /// Writes a single page of data to the specified address.
    /// Note: The target sector must typically be erased before writing.
    ///
    /// # Arguments
    /// * `addr` - The starting address of the page to write to.
    /// * `data` - The data to write. The length should not exceed the page size.
    fn write_page(&mut self, addr: u32, data: &[u8]) -> Result<(), Self::Error>;

    /// Erases a sector of the flash memory.
    ///
    /// # Arguments
    /// * `addr` - The address of the sector to erase.
    fn erase_sector(&mut self, addr: u32) -> Result<(), Self::Error>;

    /// Reads the device's manufacturer and device ID.
    /// This is useful for verifying that the correct flash chip is connected.
    fn read_device_id(&mut self) -> Result<u32, Self::Error>;
}
