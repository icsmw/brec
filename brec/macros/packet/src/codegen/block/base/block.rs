use crate::*;
use proc_macro2::TokenStream;
use quote::{format_ident, quote, ToTokens};
use syn::{token, Visibility};

impl Base for Block {
    fn gen(&self) -> Result<TokenStream, E> {
        let referred_name = self.referred_name();
        let block_name = self.name();
        let struct_fields = self
            .fields
            .iter()
            .map(|f| {
                let visibility = if f.public {
                    Visibility::Public(token::Pub::default()).into_token_stream()
                } else {
                    TokenStream::new()
                };
                let inner = if matches!(f.ty, Ty::blob(..)) {
                    f.referenced_ty()
                } else {
                    f.direct_ty()
                };
                quote! {
                    #visibility #inner
                }
            })
            .collect::<Vec<TokenStream>>();
        let derefed = self
            .fields
            .iter()
            .filter(|f| !f.injected)
            .map(|f| {
                let field = format_ident!("{}", f.name);
                let field_path = if matches!(f.ty, Ty::blob(..)) {
                    quote! {
                        *block.#field
                    }
                } else {
                    quote! {
                        block.#field
                    }
                };
                quote! {
                    #field: #field_path,
                }
            })
            .collect::<Vec<TokenStream>>();
        let const_sig = self.const_sig_name();
        let sig = self.sig();
        let sig_len = self.sig_len();
        Ok(quote! {

            #[repr(C)]
            #[derive(Debug)]
            struct #referred_name <'a>
                where Self: Sized
            {
                #(#struct_fields)*
            }

            impl<'a> From<#referred_name <'a>> for #block_name {
                fn from(block: #referred_name <'a>) -> Self {
                    #block_name {
                        #(#derefed)*
                    }
                }
            }

            const #const_sig: [u8; #sig_len] = #sig;

            impl brec::SignatureU32 for #referred_name <'_> {

                fn sig() -> &'static [u8; #sig_len] {
                    &#const_sig
                }

            }

        })
    }
}

impl Gen for Block {
    fn gen(&self) -> Result<TokenStream, E> {
        let base = Base::gen(self)?;
        let read = Read::gen(self)?;
        let read_slice = ReadFromSlice::gen(self)?;
        let try_read = TryRead::gen(self)?;
        let try_read_buffered = TryReadBuffered::gen(self)?;
        let crc = Crc::gen(self)?;
        let size = Size::gen(self);
        let write = Write::gen(self)?;
        Ok(quote! {
            #base
            #crc
            #size
            #read
            #read_slice
            #try_read
            #try_read_buffered
            #write
        })
    }
}
