use std::io::{Write, Read};

pub struct MemRead<'a> {
    slice: &'a[u8],
    index: usize
}

impl<'a> Iterator for MemRead<'a> {
    type Item = u8;

    fn next(&mut self) -> Option<Self::Item> {
        if self.index < self.slice.len() {
            self.index += 1;
            Some(self.slice[self.index - 1])
        } else { None }
    }
}

impl<'a> Read for MemRead<'a> {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        for (i, b) in buf.iter_mut().enumerate() {
            if self.index < self.slice.len() {
                self.index += 1;
                *b = self.slice[self.index - 1];
            } else { return Ok(i) }
            self.index += 1;
        }
        Ok(buf.len())
    }
}

// Write bytes to memory.
// `write` never errors and returns `Ok(0)` 
pub struct MemWrite<'a> {
    slice: &'a mut [u8],
    index: usize
}

impl<'a> Write for MemWrite<'a> {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        for (i, b) in buf.iter().enumerate() {
            if self.index < self.slice.len() {
                self.index += 1;
                self.slice[self.index - 1] = *b;
            } else { return Ok(i) }
            self.index += 1;
        }
        Ok(buf.len())
    }

    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}


pub trait Device {
    // read from a slice of memory
    fn read(&mut self, data: MemRead);
    // write to a slice of memory
    fn write(&mut self, data: MemWrite);
}

struct Console {

}

impl Device for Console {
    fn read(&mut self, data: MemRead) {
        std::io::stdout().write_all(&data.collect::<Vec<_>>()[..]).unwrap();
    }

    fn write(&mut self, _data: MemWrite) {
        
    }
}