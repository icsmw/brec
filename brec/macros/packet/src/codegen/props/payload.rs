use crate::*;
use proc_macro2::TokenStream;
use quote::quote;

impl Crc for Payload {
    fn gen(&self) -> Result<TokenStream, E> {
        let payload_name = self.name();
        Ok(if self.attrs.is_bincode() {
            quote! {
                impl brec::PayloadCrc for #payload_name {}
            }
        } else {
            quote! {}
        })
    }
}

impl Size for Payload {
    fn gen(&self) -> TokenStream {
        let payload_name = self.name();
        if self.attrs.is_bincode() {
            quote! {
                impl brec::PayloadSize for #payload_name {
                    fn size(&self) -> std::io::Result<u64> {
                        bincode::serialized_size(self)
                            .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e.to_string()))
                    }
                }
            }
        } else {
            quote! {}
        }
    }
}
