use crate::*;
use proc_macro2::TokenStream;

impl ReferredPacket for Packet {
    fn referred(&self) -> TokenStream {
        let name = self.referred_name();
        let fields = self
            .fields
            .iter()
            .map(|f| f.referred())
            .collect::<Vec<TokenStream>>();
        let derefed = self
            .fields
            .iter()
            .filter(|f| !f.injected)
            .map(|f| {
                let field = format_ident!("{}", f.name);
                let field_path = quote! {
                    packet.#field
                };
                quote! {
                    #field: *#field_path,
                }
            })
            .collect::<Vec<TokenStream>>();
        quote! {
            #[repr(C)]
            #[derive(Debug)]
            struct #name <'a> {
                #(#fields)*
            }

            impl<'a> From<#name <'a>> for MyPacket {
                fn from(packet: #name <'a>) -> Self {
                    MyPacket {
                        #(#derefed)*
                    }
                }
            }
        }
    }
}
