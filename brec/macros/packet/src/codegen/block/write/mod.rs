mod block;

use crate::*;
use proc_macro2::TokenStream;

pub trait Write {
    fn gen(&self) -> Result<TokenStream, E>;
}

pub trait WriteOwned {
    fn gen(&self) -> Result<TokenStream, E>;
}
