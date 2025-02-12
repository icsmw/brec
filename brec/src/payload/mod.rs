mod defaults;
mod header;

pub use header::*;

pub trait PayloadEncode {
    fn encode(&self) -> std::io::Result<Vec<u8>>;
}

pub trait PayloadEncodeReferred {
    fn encode(&self) -> std::io::Result<Option<&[u8]>>;
}

pub trait PayloadDecode<T> {
    fn decode(buf: &[u8]) -> std::io::Result<T>;
}
