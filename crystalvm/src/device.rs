pub trait Device {
    fn write_byte(&mut self) -> Option<u8>;
    fn receive_byte(&mut self, b: u8);
}