use crate::*;
use proc_macro2::TokenStream;
use quote::quote;

impl Crc for Block {
    fn gen(&self) -> Result<TokenStream, E> {
        let block_name = self.name();
        let referred_name = self.referred_name();
        if self.attrs.is_no_crc() {
            return Ok(quote! {

                impl brec::CrcU32 for #block_name {

                    fn crc(&self) -> [u8; 4] {
                        [0u8, 0u8, 0u8, 0u8, ]
                    }

                }

                impl brec::CrcU32 for #referred_name<'_> {

                    fn crc(&self) -> [u8; 4] {
                        [0u8, 0u8, 0u8, 0u8, ]
                    }

                }

            });
        }
        let mut hash_packet = Vec::new();
        let mut hash_referred = Vec::new();
        for field in self.fields.iter().filter(|f| !f.injected) {
            let value = field.to_bytes(true)?;
            let referred = field.to_bytes(false)?;
            hash_packet.push(quote! {
                hasher.update(#value);
            });
            hash_referred.push(quote! {
                hasher.update(#referred);
            });
        }
        Ok(quote! {

            impl brec::CrcU32 for #block_name {

                fn crc(&self) -> [u8; 4] {
                    let mut hasher = brec::crc32fast::Hasher::new();
                    #(#hash_packet)*
                    hasher.finalize().to_le_bytes()
                }

            }

            impl brec::CrcU32 for #referred_name<'_> {

                fn crc(&self) -> [u8; 4] {
                    let mut hasher = brec::crc32fast::Hasher::new();
                    #(#hash_referred)*
                    hasher.finalize().to_le_bytes()
                }

            }

        })
    }
}

impl Size for Block {
    fn gen(&self) -> TokenStream {
        let block_name = self.name();
        let mut size = 0u64;
        for field in self.fields.iter() {
            size += field.size() as u64;
        }
        quote! {

            impl brec::StaticSize for #block_name {

                fn ssize() -> u64 {
                    #size
                }

            }

        }
    }
}
