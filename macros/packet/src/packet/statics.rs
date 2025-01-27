use crate::*;
use proc_macro2::TokenStream;

impl StaticPacket for Packet {
    fn r#static(&self) -> TokenStream {
        let name = self.packet_name();
        let fields = self
            .fields
            .iter()
            .map(|f| f.r#static())
            .collect::<Vec<TokenStream>>();
        quote! {
            #[repr(C)]
            #[derive(Debug)]
            struct #name {
                sig: [u8; 4],
                #(#fields)*
                crc: u32,
                next: [u8; 4],
            }
        }
    }
}
