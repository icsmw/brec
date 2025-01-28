mod packet;

use proc_macro2::TokenStream;

pub trait StructuredMode {
    fn generate(&self) -> TokenStream;
}
