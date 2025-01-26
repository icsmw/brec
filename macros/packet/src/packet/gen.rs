use crate::*;
use proc_macro2::TokenStream;

impl ReferredPacket for Packet {
    fn referred(&self) -> TokenStream {
        let name = format_ident!("{}Referred", self.packet_name());
        let fields = self
            .fields
            .iter()
            .map(|f| f.referred())
            .collect::<Vec<TokenStream>>();
        quote! {
            struct #name <'a> {
                sig: &'a [u8; 4],
                #(#fields)*
                crc: &'a u32,
                next: &'a [u8; 4],
            }
        }
    }
}
