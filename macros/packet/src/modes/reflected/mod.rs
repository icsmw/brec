mod packet;

use proc_macro2::TokenStream;

pub trait ReflectedMode {
    fn generate(&self) -> TokenStream;
}
