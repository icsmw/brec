mod defaults;
mod header;

pub use header::*;

pub trait PayloadHooks {
    fn before_encode(&mut self) -> std::io::Result<()> {
        Ok(())
    }
    fn after_decode(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}
pub trait PayloadEncode: PayloadHooks {
    fn encode(&self) -> std::io::Result<Vec<u8>>;
}

pub trait PayloadEncodeReferred {
    fn encode(&self) -> std::io::Result<Option<&[u8]>>;
}

pub trait PayloadDecode<T>: PayloadHooks {
    fn decode(buf: &[u8]) -> std::io::Result<T>;
}
