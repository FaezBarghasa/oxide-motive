#![no_std]

//! Hardware Abstraction Layer (HAL) for Oxide-Motive.
//!
//! This crate defines traits that abstract various hardware peripherals,
//! allowing the firmware to be written in a hardware-agnostic manner.
//! Specific MCU HAL implementations (e.g., `stm32h7xx-hal`, `s32k-hal`)
//! will implement these traits.

/// Error type for HAL operations.
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum HalError {
    /// Operation timed out.
    Timeout,
    /// Invalid argument provided.
    InvalidArgument,
    /// Peripheral is busy.
    Busy,
    /// Generic I/O error.
    Io,
    /// Feature not supported by the hardware.
    Unsupported,
}

/// Trait for Analog-to-Digital Converter (ADC) functionality.
pub trait Adc {
    /// The type representing a raw ADC reading.
    type RawValue: Into<u16>;
    /// The type representing a physical voltage reading.
    type Voltage: Into<f32>;

    /// Reads a single ADC channel.
    ///
    /// # Arguments
    /// * `channel` - The channel to read.
    ///
    /// # Returns
    /// The raw ADC value, or a `HalError` if the read fails.
    fn read_channel(&mut self, channel: u8) -> Result<Self::RawValue, HalError>;

    /// Reads all configured ADC channels into a buffer.
    ///
    /// # Arguments
    /// * `buffer` - A mutable slice to store the raw ADC values.
    ///
    /// # Returns
    /// The number of channels read, or a `HalError` if the read fails.
    fn read_all_channels(&mut self, buffer: &mut [Self::RawValue]) -> Result<usize, HalError>;

    /// Converts a raw ADC value to a physical voltage.
    ///
    /// # Arguments
    /// * `raw_value` - The raw ADC value.
    ///
    /// # Returns
    /// The voltage in Volts.
    fn raw_to_voltage(&self, raw_value: Self::RawValue) -> Self::Voltage;

    /// Calibrates the ADC.
    ///
    /// This might involve reading internal references or applying offset/gain corrections.
    fn calibrate(&mut self) -> Result<(), HalError>;
}

/// Trait for General Purpose Input/Output (GPIO) pins.
pub trait GpioPin {
    /// Sets the pin to a high logic level.
    fn set_high(&mut self);

    /// Sets the pin to a low logic level.
    fn set_low(&mut self);

    /// Toggles the pin's logic level.
    fn toggle(&mut self);

    /// Reads the current logic level of the pin.
    ///
    /// # Returns
    /// `true` if the pin is high, `false` if low.
    fn is_high(&self) -> bool;

    /// Configures the pin as an output.
    fn into_output(&mut self);

    /// Configures the pin as an input.
    fn into_input(&mut self);
}

/// Trait for Pulse Width Modulation (PWM) output.
pub trait Pwm {
    /// Sets the duty cycle for a specific PWM channel.
    ///
    /// # Arguments
    /// * `channel` - The PWM channel to configure.
    /// * `duty_cycle` - The duty cycle as a float between 0.0 and 1.0.
    fn set_duty_cycle(&mut self, channel: u8, duty_cycle: f32) -> Result<(), HalError>;

    /// Sets the pulse width for a specific PWM channel.
    ///
    /// # Arguments
    /// * `channel` - The PWM channel to configure.
    /// * `pulse_width_us` - The pulse width in microseconds.
    fn set_pulse_width_us(&mut self, channel: u8, pulse_width_us: u32) -> Result<(), HalError>;

    /// Sets the frequency of the PWM output.
    ///
    /// # Arguments
    /// * `frequency_hz` - The frequency in Hertz.
    fn set_frequency_hz(&mut self, frequency_hz: u32) -> Result<(), HalError>;
}

/// Trait for a high-resolution timer.
pub trait Timer {
    /// The type representing a timer tick.
    type Tick: Into<u32>;

    /// Returns the current value of the timer counter.
    fn counter(&self) -> Self::Tick;

    /// Sets the compare value for a timer channel.
    /// When the counter reaches this value, an interrupt can be triggered.
    ///
    /// # Arguments
    /// * `compare_value` - The value to compare against.
    fn set_compare(&mut self, compare_value: Self::Tick);

    /// Enables the compare interrupt for a timer channel.
    fn enable_compare_interrupt(&mut self);

    /// Disables the compare interrupt for a timer channel.
    fn disable_compare_interrupt(&mut self);

    /// Clears the compare interrupt flag.
    fn clear_interrupt(&mut self);

    /// Returns the captured value from an input capture channel.
    fn capture_value(&self) -> Self::Tick;

    /// Enables input capture for a channel.
    fn enable_input_capture(&mut self, channel: u8) -> Result<(), HalError>;

    /// Disables input capture for a channel.
    fn disable_input_capture(&mut self, channel: u8) -> Result<(), HalError>;
}

/// Trait for Universal Asynchronous Receiver-Transmitter (UART) communication.
pub trait Uart {
    /// Writes a slice of bytes to the UART.
    ///
    /// # Arguments
    /// * `bytes` - The slice of bytes to write.
    ///
    /// # Returns
    /// The number of bytes written, or a `HalError` if the write fails.
    fn write(&mut self, bytes: &[u8]) -> Result<usize, HalError>;

    /// Reads bytes from the UART into a buffer.
    ///
    /// # Arguments
    /// * `buffer` - A mutable slice to store the read bytes.
    ///
    /// # Returns
    /// The number of bytes read, or a `HalError` if the read fails.
    fn read(&mut self, buffer: &mut [u8]) -> Result<usize, HalError>;

    /// Checks if there are any bytes available to read.
    fn has_data_to_read(&self) -> bool;

    /// Checks if the UART transmit buffer is empty.
    fn is_tx_empty(&self) -> bool;
}

/// Trait for Controller Area Network (CAN) communication.
pub trait Can {
    /// Sends a CAN message.
    ///
    /// # Arguments
    /// * `id` - The CAN message ID.
    /// * `data` - The payload of the CAN message.
    ///
    /// # Returns
    /// `Ok(())` if the message was sent, or a `HalError` if it failed.
    fn transmit(&mut self, id: u32, data: &[u8]) -> Result<(), HalError>;

    /// Receives a CAN message.
    ///
    /// # Arguments
    /// * `id_buffer` - A mutable reference to store the received message ID.
    /// * `data_buffer` - A mutable slice to store the received message payload.
    ///
    /// # Returns
    /// The number of bytes received, or `None` if no message is available,
    /// or a `HalError` if an error occurred during reception.
    fn receive(&mut self, id_buffer: &mut u32, data_buffer: &mut [u8]) -> Result<usize, HalError>;

    /// Checks if there are any pending CAN messages to receive.
    fn has_pending_messages(&self) -> bool;
}

/// Trait for Independent Watchdog (IWDG) functionality.
pub trait Watchdog {
    /// Feeds the watchdog to prevent a reset.
    fn feed(&mut self);

    /// Checks if the last reset was caused by the watchdog.
    fn was_reset_by_watchdog(&self) -> bool;

    /// Clears the watchdog reset flag.
    fn clear_reset_flag(&mut self);
}

/// Trait for Non-Volatile Memory (NVM) access, e.g., Flash or EEPROM.
pub trait Nvm {
    /// Reads data from a specific address in NVM.
    ///
    /// # Arguments
    /// * `address` - The starting address to read from.
    /// * `buffer` - A mutable slice to store the read data.
    ///
    /// # Returns
    /// `Ok(())` if the read was successful, or a `HalError` if it failed.
    fn read(&mut self, address: u32, buffer: &mut [u8]) -> Result<(), HalError>;

    /// Writes data to a specific address in NVM.
    ///
    /// # Arguments
    /// * `address` - The starting address to write to.
    /// * `data` - The slice of data to write.
    ///
    /// # Returns
    /// `Ok(())` if the write was successful, or a `HalError` if it failed.
    fn write(&mut self, address: u32, data: &[u8]) -> Result<(), HalError>;

    /// Erases a sector of the NVM.
    ///
    /// # Arguments
    /// * `sector_address` - The starting address of the sector to erase.
    fn erase_sector(&mut self, sector_address: u32) -> Result<(), HalError>;
}
