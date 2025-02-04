use crate::*;
use proc_macro2::TokenStream;
use quote::{format_ident, quote};

impl Reflected for Block {
    fn generate(&self) -> TokenStream {
        let referred_name = self.referred_name();
        let struct_fields = self
            .fields
            .iter()
            .map(|f| f.referenced_ty())
            .collect::<Vec<TokenStream>>();
        let derefed = self
            .fields
            .iter()
            .filter(|f| !f.injected)
            .map(|f| {
                let field = format_ident!("{}", f.name);
                let field_path = if matches!(f.ty, Ty::Slice(..)) {
                    quote! {
                        *packet.#field
                    }
                } else {
                    quote! {
                        packet.#field
                    }
                };
                quote! {
                    #field: *#field_path,
                }
            })
            .collect::<Vec<TokenStream>>();
        let block_name = self.name();
        let const_sig = self.const_sig_name();
        let mut fields = Vec::new();
        let mut fnames = Vec::new();
        let mut offset = 0usize;
        let sig: TokenStream = self.sig();
        let src: syn::Ident = format_ident!("data");
        for field in self.fields.iter() {
            fields.push(field.r#unsafe(&src, offset));
            fnames.push(format_ident!("{}", field.name));
            offset += field.ty.size();
        }
        quote! {

            #[repr(C)]
            #[derive(Debug)]
            struct #referred_name <'a> {
                #(#struct_fields)*
            }

            impl<'a> From<#referred_name <'a>> for #block_name {
                fn from(packet: #referred_name <'a>) -> Self {
                    #block_name {
                        #(#derefed)*
                    }
                }
            }

            const #const_sig: [u8; 4] = #sig;

            impl<'a> brec::Block<'a, #referred_name <'a>> for #referred_name <'a> {

                fn sig() -> &'static [u8; 4] {
                    &#const_sig
                }

                fn read(#src: &'a [u8]) -> Result<Option<#referred_name<'a>>, brec::Error> {
                    use std::mem;

                    if #src.len() < 4 {
                        return Err(brec::Error::NotEnoughtSignatureData(#src.len(), 4));
                    }

                    if #src[..4] != #const_sig {
                        return Ok(None);
                    }

                    if #src.len() < mem::size_of::<#block_name>() {
                        return Err(brec::Error::NotEnoughtData(#src.len(), mem::size_of::<#block_name>()));
                    }

                    if #src.as_ptr() as usize % std::mem::align_of::<#block_name>() != 0 {
                        return Err(brec::Error::InvalidAlign(
                            #src.as_ptr() as usize,
                            mem::size_of::<#block_name>(),
                            #src.as_ptr() as usize % std::mem::align_of::<#block_name>()
                        ));
                    }

                    #(#fields)*

                    Ok(Some(#referred_name {
                        #(#fnames,)*
                    }))
                }
            }
        }
    }
}
