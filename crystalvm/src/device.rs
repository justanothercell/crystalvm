pub trait Device {
    fn write_byte(&mut self) -> u8;
    fn receive_byte(&mut self) -> u8;
}