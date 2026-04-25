mod block;
mod payload;

use crate::*;
use proc_macro2::TokenStream;

pub trait Read {
    fn generate(&self) -> Result<TokenStream, E>;
}

pub trait ReadFromSlice {
    fn generate(&self) -> Result<TokenStream, E>;
}

pub trait TryRead {
    fn generate(&self) -> Result<TokenStream, E>;
}

pub trait TryReadBuffered {
    fn generate(&self) -> Result<TokenStream, E>;
}
