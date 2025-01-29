use crate::*;
use proc_macro2::TokenStream;
use quote::quote;

impl Crc for Packet {
    fn gen(&self) -> TokenStream {
        let packet_name = self.name();
        let mut hash_pushes = Vec::new();
        for field in self.fields.iter().filter(|f| !f.injected) {
            let as_bytes = field.to_bytes().unwrap();
            hash_pushes.push(quote! {
                hasher.update(#as_bytes);
            });
        }
        quote! {

            impl brec::Crc for #packet_name {

                fn crc(&self) -> [u8; 4] {
                    let mut hasher = brec::crc32fast::Hasher::new();
                    #(#hash_pushes)*
                    hasher.finalize().to_le_bytes()
                }

            }

        }
    }
}

impl Size for Packet {
    fn gen(&self) -> TokenStream {
        let packet_name = self.name();
        let mut size = 0usize;
        for field in self.fields.iter() {
            size += field.size();
        }
        quote! {

            impl brec::Size for #packet_name {

                fn size(&self) -> usize {
                    #size
                }

            }

        }
    }
}
