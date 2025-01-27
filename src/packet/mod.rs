pub use crate::*;

pub trait Packet<'a, T> {
    fn sig() -> &'static [u8; 4];
    fn read(data: &'a [u8]) -> Result<Option<T>, Error>;
}
