mod block;
mod napi;
mod payload;

use crate::*;
use proc_macro2::TokenStream;

pub trait Gen {
    fn generate(&self) -> Result<TokenStream, E>;
}

pub trait Base {
    fn generate(&self) -> Result<TokenStream, E>;
}
