
pub trait Watchdog {
    fn init(&mut self, timeout_ms: u32);
    fn feed(&mut self);
}
