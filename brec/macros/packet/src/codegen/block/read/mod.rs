mod block;

use crate::*;
use proc_macro2::TokenStream;

pub trait Read {
    fn gen(&self) -> Result<TokenStream, E>;
}

pub trait ReadFromSlice {
    fn gen(&self) -> Result<TokenStream, E>;
}

pub trait TryRead {
    fn gen(&self) -> Result<TokenStream, E>;
}

pub trait TryReadBuffered {
    fn gen(&self) -> Result<TokenStream, E>;
}
