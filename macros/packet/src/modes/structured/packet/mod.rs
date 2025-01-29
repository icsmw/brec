use crate::*;
use proc_macro2::TokenStream;
use quote::{format_ident, quote};

impl StructuredBase for Packet {
    fn gen(&self) -> TokenStream {
        let referred_name = self.referred_name();
        let struct_fields = self
            .fields
            .iter()
            .map(|f| {
                if matches!(f.ty, Ty::Slice(..) | Ty::Option(..)) {
                    f.referenced_ty()
                } else {
                    f.direct_ty()
                }
            })
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
                    #field: #field_path,
                }
            })
            .collect::<Vec<TokenStream>>();
        let const_sig = self.const_sig_name();
        let sig: TokenStream = self.sig();
        quote! {

            #[repr(C)]
            #[derive(Debug)]
            struct #referred_name <'a> {
                #(#struct_fields)*
            }

            impl<'a> From<#referred_name <'a>> for MyPacket {
                fn from(packet: #referred_name <'a>) -> Self {
                    MyPacket {
                        #(#derefed)*
                    }
                }
            }

            const #const_sig: [u8; 4] = #sig;

        }
    }
}

impl StructuredDeserializableImpl for Packet {
    fn gen(&self) -> TokenStream {
        let referred_name = self.referred_name();
        let packet_name = self.name();
        let const_sig = self.const_sig_name();
        let mut fields = Vec::new();
        let mut fnames = Vec::new();
        let mut offset = 0usize;
        let src: syn::Ident = format_ident!("data");
        for field in self.fields.iter() {
            fields.push(field.safe(&src, offset, offset + field.ty.size()));
            fnames.push(format_ident!("{}", field.name));
            offset += field.ty.size();
        }
        quote! {

            impl<'a> brec::Packet<'a, #referred_name <'a>> for #referred_name <'a> {

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

                    if #src.len() < mem::size_of::<#packet_name>() {
                        return Err(brec::Error::NotEnoughtData(#src.len(), mem::size_of::<#packet_name>()));
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

impl StructuredSerializableImpl for Packet {
    fn gen(&self) -> TokenStream {
        let packet_name = self.name();
        let mut write_pushes = Vec::new();
        let mut write_all_pushes = Vec::new();
        for field in self.fields.iter().filter(|f| !f.injected) {
            let as_bytes = field.to_bytes().unwrap();
            write_pushes.push(quote! {
                + buf.write(#as_bytes)?
            });
            write_all_pushes.push(quote! {
                buf.write_all(#as_bytes)?;
            });
        }
        let const_sig = self.const_sig_name();
        quote! {

            impl brec::Write for #packet_name {

                fn write<T: std::io::Write>(&self, buf: &mut T) -> std::io::Result<usize> {
                    Ok(buf.write(&#const_sig)?
                    #(#write_pushes)*
                    + buf.write(&self.crc())?)
                }

                fn write_all<T: std::io::Write>(&self, buf: &mut T) -> std::io::Result<()> {
                    buf.write_all(&#const_sig)?;
                    #(#write_all_pushes)*
                    buf.write_all(&self.crc())?;
                    Ok(())
                }

            }

        }
    }
}

// fn write(&self,  next: Option<[u8; 4]>  ) -> [u8] {
//     let mut buffer = [0u8; #offset + 4 + 4 + 4];
//     buffer[0..4].copy_from_slice(&#const_sig);
//     #(#buf_pushes)*
//     buffer[#offset..#offset + 4].copy_from_slice(&self.crc());
//     buffer[#offset + 4 ..#offset + 4 + 4].copy_from_slice(&next);
//     buffer
// }
impl Structured for Packet {
    fn gen(&self) -> TokenStream {
        let base = StructuredBase::gen(self);
        let de = StructuredDeserializableImpl::gen(self);
        let crc = Crc::gen(self);
        let size = Size::gen(self);
        let se = StructuredSerializableImpl::gen(self);
        quote! {
            #base
            #crc
            #size
            #de
            #se
        }
    }
}
