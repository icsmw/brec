use crate::*;
use proc_macro2::TokenStream;
use quote::quote;

impl Write for Payload {
    fn gen(&self) -> Result<TokenStream, E> {
        let payload_name = self.name();
        Ok(quote! {
            impl brec::WritePayloadWithHeaderTo for #payload_name {}
        })
    }
}

impl WriteVectored for Payload {
    fn gen(&self) -> Result<TokenStream, E> {
        let payload_name = self.name();
        Ok(quote! {
            impl brec::WriteVectoredPayloadWithHeaderTo for #payload_name {}
        })
    }
}
