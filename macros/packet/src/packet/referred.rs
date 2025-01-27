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
        let sig = self.sig();
        let const_sig = self.const_sig_name();
        quote! {
            #[repr(C)]
            #[derive(Debug)]
            struct #name <'a> {
                sig: &'a [u8; 4],
                #(#fields)*
                crc: &'a u32,
                next: &'a [u8; 4],
            }

            const #const_sig: [u8; 4] = #sig;

            impl<'a> brec::Packet for #name <'a> {
                fn sig() -> &'static [u8; 4] {
                    &#const_sig
                }
            }
        }
    }
}
