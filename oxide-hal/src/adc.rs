
pub trait Adc {
    type Error;
    fn read_all(&mut self) -> Result<[u16; 16], Self::Error>;
    fn calibrate(&mut self) -> Result<(), Self::Error>;
}
