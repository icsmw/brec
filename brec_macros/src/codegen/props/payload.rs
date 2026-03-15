use crate::*;
use proc_macro2::TokenStream;
use quote::quote;

impl Crc for Payload {
    fn generate(&self) -> Result<TokenStream, E> {
        let payload_name = self.name();
        Ok(
            if self.attrs.is_bincode() && !self.attrs.is_no_auto_crc() && !self.attrs.is_no_crc() {
                quote! {
                    impl brec::PayloadCrc for #payload_name {}
                }
            } else if self.attrs.is_bincode() && self.attrs.is_no_crc() {
                quote! {
                    impl brec::PayloadCrc for #payload_name {
                        fn crc(
                            &self,
                            _ctx: &mut Self::Context<'_>,
                        ) -> std::io::Result<brec::ByteBlock> {
                            Ok(brec::ByteBlock::Len4([0,0,0,0]))
                        }
                        fn crc_size() -> usize {
                            4
                        }
                    }
                }
            } else {
                quote! {}
            },
        )
    }
}

impl Size for Payload {
    fn generate(&self) -> TokenStream {
        let payload_name = self.name();
        if self.attrs.is_bincode() {
            quote! {
                impl brec::PayloadSize for #payload_name {}
            }
        } else {
            quote! {}
        }
    }
}
