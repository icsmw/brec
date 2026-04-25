mod block;
mod payload;

use crate::*;

use proc_macro2::TokenStream;

pub trait Size {
    fn generate(&self) -> TokenStream;
}

pub trait Crc {
    fn generate(&self) -> Result<TokenStream, E>;
}
