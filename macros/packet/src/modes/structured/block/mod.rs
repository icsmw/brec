use crate::*;
use proc_macro2::TokenStream;
use quote::{format_ident, quote};

impl StructuredBase for Block {
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
            struct #referred_name <'a>
                where Self: Sized
            {
                #(#struct_fields)*
            }

            impl<'a> From<#referred_name <'a>> for MyBlock {
                fn from(packet: #referred_name <'a>) -> Self {
                    MyBlock {
                        #(#derefed)*
                    }
                }
            }

            const #const_sig: [u8; 4] = #sig;

            impl<'a> #referred_name <'a> {

                pub fn sig() -> &'static [u8; 4] {
                    &#const_sig
                }

            }

        }
    }
}

impl StructuredRead for Block {
    fn gen(&self) -> TokenStream {
        let packet_name = self.name();
        let const_sig = self.const_sig_name();
        let mut fields = Vec::new();
        let mut fnames = Vec::new();
        let src: syn::Ident = format_ident!("buf");
        for field in self.fields.iter().filter(|f| !f.injected) {
            fields.push(field.read_exact(&src).unwrap());
            fnames.push(format_ident!("{}", field.name));
        }
        quote! {

            impl brec::Read for #packet_name {
                fn read<T: std::io::Read>(buf: &mut T) -> Result<Self, brec::Error>
                where
                    Self: Sized {
                        let mut sig = [0u8; 4];
                        #src.read_exact(&mut sig)?;
                        if sig != #const_sig {
                            return Err(brec::Error::SignatureDismatch)
                        }

                        #(#fields)*

                        let mut crc = [0u8; 4];
                        #src.read_exact(&mut crc)?;

                        let packet = #packet_name {
                            #(#fnames,)*
                        };

                        if packet.crc() != crc {
                            return Err(brec::Error::CrcDismatch)
                        }

                        Ok(packet)
                }
            }

        }
    }
}

impl StructuredReadFromSlice for Block {
    fn gen(&self) -> TokenStream {
        let referred_name = self.referred_name();
        let packet_name = self.name();
        let const_sig = self.const_sig_name();
        let mut fields = Vec::new();
        let mut fnames = Vec::new();
        let mut offset = 0usize;
        let src: syn::Ident = format_ident!("buf");
        for field in self.fields.iter() {
            fields.push(field.safe(&src, offset, offset + field.ty.size()));
            fnames.push(format_ident!("{}", field.name));
            offset += field.ty.size();
        }
        quote! {

            impl<'a> brec::ReadFromSlice<'a> for #referred_name <'a> {

                fn read_from_slice(#src: &'a [u8]) -> Result<Self, brec::Error>
                where
                    Self: Sized,
                {
                    if #src.len() < 4 {
                        return Err(brec::Error::NotEnoughtSignatureData(#src.len(), 4));
                    }

                    if #src[..4] != #const_sig {
                        return Err(brec::Error::SignatureDismatch);
                    }

                    if #src.len() < std::mem::size_of::<#packet_name>() {
                        return Err(brec::Error::NotEnoughtData(#src.len(), std::mem::size_of::<#packet_name>()));
                    }

                    #(#fields)*

                    Ok(#referred_name {
                        #(#fnames,)*
                    })
                }

            }
        }
    }
}

impl StructuredWrite for Block {
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

impl Structured for Block {
    fn gen(&self) -> TokenStream {
        let base = StructuredBase::gen(self);
        let read = StructuredRead::gen(self);
        let read_slice = StructuredReadFromSlice::gen(self);
        let crc = Crc::gen(self);
        let size = Size::gen(self);
        let write = StructuredWrite::gen(self);
        quote! {
            #base
            #crc
            #size
            #read
            #read_slice
            #write
        }
    }
}
