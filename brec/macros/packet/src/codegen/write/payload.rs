use crate::*;
use proc_macro2::TokenStream;
use quote::quote;

impl Write for Payload {
    fn gen(&self) -> Result<TokenStream, E> {
        let payload_name = self.name();
        Ok(quote! {
            impl brec::WritePayloadTo for #payload_name {}
        })
    }
}

impl WriteVectored for Payload {
    fn gen(&self) -> Result<TokenStream, E> {
        let payload_name = self.name();
        Ok(quote! {
            impl brec::WriteVectoredPayloadTo for #payload_name {}
        })
    }
}
