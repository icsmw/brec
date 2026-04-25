mod field;

use crate::*;
use proc_macro2::TokenStream;

pub trait ToBytes {
    fn to_bytes(&self, blob_by_ref: bool) -> Result<TokenStream, E>;
}
