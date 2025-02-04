use crate::*;
use proc_macro2::TokenStream;
use quote::quote;

impl Crc for Block {
    fn gen(&self) -> TokenStream {
        let packet_name = self.name();
        let mut hash_pushes = Vec::new();
        for field in self.fields.iter().filter(|f| !f.injected) {
            let as_bytes = field.to_bytes().unwrap();
            let el = if let Ty::Slice(.., inner_ty) = &field.ty {
                if matches!(**inner_ty, Ty::u8) {
                    quote! {
                        hasher.update(#as_bytes);
                    }
                } else {
                    quote! {
                        let bytes = #as_bytes;
                        hasher.update(&bytes);
                    }
                }
            } else {
                quote! {
                    hasher.update(#as_bytes);
                }
            };
            hash_pushes.push(el);
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

impl Size for Block {
    fn gen(&self) -> TokenStream {
        let packet_name = self.name();
        let mut size = 0u64;
        for field in self.fields.iter() {
            size += field.size() as u64;
        }
        quote! {

            impl brec::Size for #packet_name {

                fn size() -> u64 {
                    #size
                }

            }

        }
    }
}
