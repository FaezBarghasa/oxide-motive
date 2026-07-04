
use core::fmt::Debug;

/// An error type for timer operations.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TimerError {
    /// The requested channel is not available or invalid.
    InvalidChannel,
    /// An underlying hardware error occurred.
    HardwareError,
}

/// A trait for a hardware timer capable of precise scheduling, essential for engine control.
///
/// This timer provides microsecond-resolution timing and compare-match interrupts,
/// which are fundamental for scheduling events like ignition and injection.
pub trait EngineTimer {
    /// The error type returned by timer methods.
    type Error: Debug;

    /// Returns the current value of the timer's counter in microseconds.
    ///
    /// This provides a high-resolution timestamp for scheduling and logging.
    fn counter_us(&self) -> u32;

    /// Sets a compare match interrupt on a specific channel.
    ///
    /// When the timer's counter reaches `ticks_us`, an interrupt will be triggered
    /// on the specified `channel`. This is the primary mechanism for scheduling
    /// future actions.
    ///
    /// # Arguments
    /// * `channel` - The hardware timer channel to configure.
    /// * `ticks_us` - The time in microseconds for the compare match event.
    fn set_compare_us(&mut self, channel: u8, ticks_us: u32) -> Result<(), Self::Error>;

    /// Enables the compare match interrupt for a specific channel.
    ///
    /// # Arguments
    /// * `channel` - The hardware timer channel to enable interrupts for.
    fn enable_compare_interrupt(&mut self, channel: u8);

    /// Clears the interrupt flag for a specific channel.
    ///
    - This must be called within the interrupt handler to prevent re-triggering.
    ///
    /// # Arguments
    /// * `channel` - The hardware timer channel whose interrupt flag should be cleared.
    fn clear_interrupt_flag(&mut self, channel: u8);

    /// Returns the number of timer ticks that correspond to one microsecond.
    ///
    /// This is useful for converting between raw timer ticks and microseconds, and for
    /// understanding the timer's resolution.
    fn ticks_per_us(&self) -> u32;
}
