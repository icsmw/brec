pub trait Signature {
    fn sig() -> &'static [u8; 4];
}

pub trait Crc {
    fn crc(&self) -> [u8; 4];
}
