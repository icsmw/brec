use crate::*;
use proc_macro2::TokenStream;
use quote::quote;

impl Read for Payload {
    fn gen(&self) -> Result<TokenStream, E> {
        let payload_name = self.name();
        Ok(quote! {
            impl brec::ReadPayloadFrom<#payload_name> for #payload_name {}
        })
    }
}

impl TryRead for Payload {
    fn gen(&self) -> Result<TokenStream, E> {
        let payload_name = self.name();
        Ok(quote! {
            impl brec::TryReadPayloadFrom<#payload_name> for #payload_name {}
        })
    }
}

impl TryReadBuffered for Payload {
    fn gen(&self) -> Result<TokenStream, E> {
        let payload_name = self.name();
        Ok(quote! {
            impl brec::TryReadPayloadFromBuffered<#payload_name> for #payload_name {}
        })
    }
}
