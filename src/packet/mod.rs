pub use crate::*;

pub trait Packet<T> {
    fn sig() -> &'static [u8; 4];
    fn read(data: &[u8]) -> Result<Option<T>, Error>;
}
