mod block;

pub(crate) use block::*;

use crate::*;
use proc_macro2::TokenStream;

pub trait Gen {
    fn gen(&self) -> Result<TokenStream, E>;
}

pub trait Base {
    fn gen(&self) -> Result<TokenStream, E>;
}
