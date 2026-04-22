mod block;
mod payload;

use crate::*;
use proc_macro2::TokenStream;

pub trait Write {
    fn generate(&self) -> Result<TokenStream, E>;
}

pub trait WriteVectored {
    fn generate(&self) -> Result<TokenStream, E>;
}
