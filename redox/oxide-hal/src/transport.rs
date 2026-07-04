use embedded_io_async::{Read, Write};

pub trait Transport: Read + Write {}
