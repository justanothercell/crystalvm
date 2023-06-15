pub trait Device {
    fn dinfo(&self) -> [u8; 32];
}