use std::{io::{Write, Read, Stdout}, process::Stdio};

use getch::Getch;

/// Operations on invalid device id return immediately, setting FLAG_BIT_E.
/// A device should never panic, instead just return 0.
pub trait Device {
    /// read from device. may block until data arrives
    fn read(&mut self) -> u32 {
        (self.read8() as u32) << 24 | (self.read8() as u32) << 16 | (self.read8() as u32) << 8 | (self.read8() as u32)
    }
    /// write to device
    fn write(&mut self, data: u32) {
        self.write8((data >> 24) as u8);
        self.write8((data >> 16) as u8);
        self.write8((data >> 8) as u8);
        self.write8(data as u8);
    }
    /// read from device. may block until data arrives
    fn read8(&mut self) -> u8;
    /// write to device
    fn write8(&mut self, data: u8);
    fn flush_read(&mut self);
    fn flush_write(&mut self);
}

struct Console {
    input: Getch,
    output: Stdout
}

impl Console {
    pub fn new() -> Self {
        Self { 
            input: Getch::new(), 
            output: std::io::stdout() 
        }
    }
}

impl Device for Console {
    fn read8(&mut self) -> u8 {
        self.input.getch().unwrap_or(0)
    }

    fn write8(&mut self, data: u8) {
        let _ = self.output.write_all(&[data]);
    }

    fn flush_read(&mut self) { }

    fn flush_write(&mut self) { let _ = self.output.flush();}
}