mod block;

use crate::*;

use proc_macro2::TokenStream;

pub trait Size {
    fn gen(&self) -> TokenStream;
}

pub trait Crc {
    fn gen(&self) -> Result<TokenStream, E>;
}
