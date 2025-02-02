use crate::*;
use proc_macro2::TokenStream;
use quote::{format_ident, quote};

impl StructuredBase for Block {
    fn gen(&self) -> Result<TokenStream, E> {
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

            impl<'a> From<#referred_name <'a>> for MyBlock {
                fn from(packet: #referred_name <'a>) -> Self {
                    MyBlock {
                        #(#derefed)*
                    }
                }
            }

            const #const_sig: [u8; #sig_len] = #sig;

            impl Signature for #referred_name <'_> {

                fn sig() -> &'static [u8; #sig_len] {
                    &#const_sig
                }

            }

        })
    }
}

impl StructuredRead for Block {
    fn gen(&self) -> Result<TokenStream, E> {
        let packet_name = self.name();
        let const_sig = self.const_sig_name();
        let mut fields = Vec::new();
        let mut fnames = Vec::new();
        let sig_len = self.sig_len();
        let src: syn::Ident = format_ident!("buf");
        for field in self.fields.iter().filter(|f| !f.injected) {
            fields.push(field.read_exact(&src)?);
            fnames.push(format_ident!("{}", field.name));
        }
        Ok(quote! {

            impl brec::Read for #packet_name {
                fn read<T: std::io::Read>(buf: &mut T, skip_sig: bool) -> Result<Self, brec::Error>
                where
                    Self: Sized {
                        if !skip_sig {
                            let mut sig = [0u8; #sig_len];
                            #src.read_exact(&mut sig)?;
                            if sig != #const_sig {
                                return Err(brec::Error::SignatureDismatch)
                            }
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

        })
    }
}

impl StructuredReadFromSlice for Block {
    fn gen(&self) -> Result<TokenStream, E> {
        let referred_name = self.referred_name();
        let packet_name = self.name();
        let const_sig = self.const_sig_name();
        let sig_len = self.sig_len();
        let mut fields = Vec::new();
        let mut fnames = Vec::new();
        let mut offset = 0usize;
        let src: syn::Ident = format_ident!("buf");
        for field in self.fields.iter() {
            if &field.name == "__sig" {
                fields.push(quote! {
                    let __sig = if skip_sig {
                        &#const_sig
                    } else {
                        <&[u8; 4usize]>::try_from(&#src[0usize..4usize])?
                    };
                });
            } else {
                fields.push(field.safe(&src, offset, offset + field.ty.size()));
            }
            fnames.push(format_ident!("{}", field.name));
            offset += field.ty.size();
        }
        Ok(quote! {

            impl<'a> brec::ReadFromSlice<'a> for #referred_name <'a> {

                fn read_from_slice(#src: &'a [u8], skip_sig: bool) -> Result<Self, brec::Error>
                where
                    Self: Sized,
                {
                    if !skip_sig {
                        if #src.len() < #sig_len {
                            return Err(brec::Error::NotEnoughtSignatureData(#src.len(), #sig_len));
                        }

                        if #src[..#sig_len] != #const_sig {
                            return Err(brec::Error::SignatureDismatch);
                        }
                    }
                    let required = if skip_sig {
                        #packet_name::size() - #sig_len
                    } else {
                        #packet_name::size()
                    } as usize;
                    if #src.len() < required {
                        return Err(brec::Error::NotEnoughtData(#src.len(), required));
                    }

                    #(#fields)*

                    Ok(#referred_name {
                        #(#fnames,)*
                    })
                }

            }
        })
    }
}

impl StructuredTryRead for Block {
    fn gen(&self) -> Result<TokenStream, E> {
        let packet_name = self.name();
        let const_sig = self.const_sig_name();
        let sig_len = self.sig_len();
        Ok(quote! {

            impl brec::TryRead for #packet_name {

                fn try_read<T: std::io::Read + std::io::Seek>(buf: &mut T) -> Result<ReadStatus<Self>, Error>
                where
                    Self: Sized,
                {
                    let mut sig_buf = [0u8; #sig_len];
                    let start_pos = buf.stream_position()?;
                    let len = buf.seek(std::io::SeekFrom::End(0))? - start_pos;

                    buf.seek(std::io::SeekFrom::Start(start_pos))?;
                    if len < #sig_len {
                        return Ok(ReadStatus::NotEnoughtDataToReadSig(#sig_len - len));
                    }
                    buf.read_exact(&mut sig_buf)?;
                    if sig_buf != #const_sig {
                        buf.seek(std::io::SeekFrom::Start(start_pos))?;
                        return Ok(ReadStatus::DismatchSignature);
                    }
                    if len < #packet_name::size() {
                        return Ok(ReadStatus::NotEnoughtData(#packet_name::size() - len));
                    }
                    Ok(ReadStatus::Success(#packet_name::read(buf, true)?))
                }
            }
        })
    }
}

impl StructuredTryReadBuffered for Block {
    fn gen(&self) -> Result<TokenStream, E> {
        let packet_name = self.name();
        let const_sig = self.const_sig_name();
        let sig_len = self.sig_len();
        Ok(quote! {

            impl brec::TryReadBuffered for #packet_name {

                fn try_read<T: std::io::Read>(buf: &mut T) -> Result<ReadStatus<Self>, Error>
                where
                    Self: Sized,
                {
                    use std::io::BufRead;

                    let mut reader = std::io::BufReader::new(buf);
                    let bytes = reader.fill_buf()?;

                    if bytes.len() < #sig_len {
                        return Ok(ReadStatus::NotEnoughtDataToReadSig(
                            (#sig_len - bytes.len()) as u64,
                        ));
                    }

                    if !bytes.starts_with(&#const_sig) {
                        return Ok(ReadStatus::DismatchSignature);
                    }

                    if (bytes.len() as u64) < #packet_name::size() {
                        return Ok(ReadStatus::NotEnoughtData(
                            #packet_name::size() - bytes.len() as u64,
                        ));
                    }
                    reader.consume(#sig_len);
                    let blk = #packet_name::read(&mut reader, true);
                    reader.consume(#packet_name::size() as usize - #sig_len);
                    Ok(ReadStatus::Success(blk?))
                }
                        }
        })
    }
}

impl StructuredWrite for Block {
    fn gen(&self) -> Result<TokenStream, E> {
        let packet_name = self.name();
        let mut write_pushes = Vec::new();
        let mut write_all_pushes = Vec::new();
        for field in self.fields.iter().filter(|f| !f.injected) {
            let as_bytes = field.to_bytes()?;
            write_pushes.push(quote! {
                + buf.write(#as_bytes)?
            });
            write_all_pushes.push(quote! {
                buf.write_all(#as_bytes)?;
            });
        }
        let const_sig = self.const_sig_name();
        Ok(quote! {

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

        })
    }
}

impl Structured for Block {
    fn gen(&self) -> Result<TokenStream, E> {
        let base = StructuredBase::gen(self)?;
        let read = StructuredRead::gen(self)?;
        let read_slice = StructuredReadFromSlice::gen(self)?;
        let try_read = StructuredTryRead::gen(self)?;
        let try_read_buffered = StructuredTryReadBuffered::gen(self)?;
        let crc = Crc::gen(self);
        let size = Size::gen(self);
        let write = StructuredWrite::gen(self)?;
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
