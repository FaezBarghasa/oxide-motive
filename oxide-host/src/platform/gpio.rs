
use gpiod::{Chip, Lines, Options};
use anyhow::Result;

pub struct UniversalGpioManager {
    chip: Chip,
}

impl UniversalGpioManager {
    pub fn new(chip_path: &str) -> Result<Self> {
        let chip = Chip::new(chip_path)?;
        Ok(Self { chip })
    }

    pub fn set_output(&self, line_offset: u32, value: bool) -> Result<()> {
        let opts = Options::output([line_offset]);
        let lines = self.chip.request_lines(opts)?;
        lines.set_values(value as u8)?;
        Ok(())
    }

    pub fn read_input(&self, line_offset: u32) -> Result<bool> {
        let opts = Options::input([line_offset]);
        let lines = self.chip.request_lines(opts)?;
        let value = lines.get_values::<u8>()?;
        Ok(value[0] != 0)
    }
}
