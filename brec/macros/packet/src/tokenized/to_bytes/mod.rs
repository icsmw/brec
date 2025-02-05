mod field;

use crate::*;
use proc_macro2::TokenStream;

pub trait ToBytes {
    fn to_bytes(&self) -> Result<TokenStream, E>;
}
