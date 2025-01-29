mod packet;

use proc_macro2::TokenStream;

pub trait Reflected {
    fn generate(&self) -> TokenStream;
}
