use crate::*;
use proc_macro2::TokenStream;

impl Reflected for Packet {
    fn reflected(&self) -> TokenStream {
        let name = self.packet_name();
        quote! {
            struct UserPacketAReferred<'a> {
                magic: &'a [u8; 4],
                a: &'a u8,
                b: &'a u64,
                c: &'a [u8; 1024],
                crc: &'a u32,
                next: &'a [u8; 4],
            }
        }
    }
}
