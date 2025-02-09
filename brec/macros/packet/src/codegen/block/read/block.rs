use crate::*;
use proc_macro2::TokenStream;
use quote::{format_ident, quote};

impl Read for Block {
    fn gen(&self) -> Result<TokenStream, E> {
        let block_name = self.name();
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

            impl brec::ReadBlockFrom for #block_name {
                fn read<T: std::io::Read>(buf: &mut T, skip_sig: bool) -> Result<Self, brec::Error>
                where
                    Self: Sized {
                        use brec::prelude::*;
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

                        let block = #block_name {
                            #(#fnames,)*
                        };
                        if block.crc() != crc {
                            return Err(brec::Error::CrcDismatch)
                        }

                        Ok(block)
                }
            }

        })
    }
}

impl ReadFromSlice for Block {
    fn gen(&self) -> Result<TokenStream, E> {
        let referred_name = self.referred_name();
        let block_name = self.name();
        let const_sig = self.const_sig_name();
        let sig_len = self.sig_len();
        let mut fields = Vec::new();
        let mut fnames = Vec::new();
        let mut offset = 0usize;
        let src: syn::Ident = format_ident!("buf");
        for field in self.fields.iter() {
            if &field.name == "__sig" {
                let name = format_ident!("{}", FIELD_SIG);
                fields.push(quote! {
                    let #name = if skip_sig {
                        &#const_sig
                    } else {
                        <&[u8; #SIG_LEN]>::try_from(&#src[0usize..#SIG_LEN])?
                    };
                });
            } else if field.name == FIELD_CRC {
                let name = format_ident!("{}", FIELD_CRC);
                fields.push(quote! {
                    let #name = <&[u8; #CRC_LEN]>::try_from(&#src[#offset..#offset + #CRC_LEN])?;
                    let crc = #name;
                });
            } else {
                fields.push(field.safe(&src, offset, offset + field.ty.size()));
            }
            fnames.push(format_ident!("{}", field.name));
            offset += field.ty.size();
        }
        Ok(quote! {

            impl<'a> brec::ReadBlockFromSlice<'a> for #referred_name <'a> {

                fn read_from_slice(#src: &'a [u8], skip_sig: bool) -> Result<Self, brec::Error>
                where
                    Self: Sized,
                {
                    use brec::prelude::*;
                    if !skip_sig {
                        if #src.len() < #sig_len {
                            return Err(brec::Error::NotEnoughtSignatureData(#src.len(), #sig_len));
                        }

                        if #src[..#sig_len] != #const_sig {
                            return Err(brec::Error::SignatureDismatch);
                        }
                    }
                    let required = if skip_sig {
                        #block_name::ssize() - #sig_len
                    } else {
                        #block_name::ssize()
                    } as usize;
                    if #src.len() < required {
                        return Err(brec::Error::NotEnoughData(#src.len(), required));
                    }

                    #(#fields)*

                    let block = #referred_name {
                        #(#fnames,)*
                    };

                    if block.crc() != *crc {
                        return Err(brec::Error::CrcDismatch)
                    }

                    Ok(block)
                }

            }
        })
    }
}

impl TryRead for Block {
    fn gen(&self) -> Result<TokenStream, E> {
        let block_name = self.name();
        let const_sig = self.const_sig_name();
        let sig_len = self.sig_len();
        Ok(quote! {

            impl brec::TryReadFrom for #block_name {

                fn try_read<T: std::io::Read + std::io::Seek>(buf: &mut T) -> Result<brec::ReadStatus<Self>, brec::Error>
                where
                    Self: Sized,
                {
                    use brec::prelude::*;
                    let mut sig_buf = [0u8; #sig_len];
                    let start_pos = buf.stream_position()?;
                    let len = buf.seek(std::io::SeekFrom::End(0))? - start_pos;

                    buf.seek(std::io::SeekFrom::Start(start_pos))?;
                    if len < #sig_len {
                        return Ok(brec::ReadStatus::NotEnoughData(#sig_len - len));
                    }
                    buf.read_exact(&mut sig_buf)?;
                    if sig_buf != #const_sig {
                        buf.seek(std::io::SeekFrom::Start(start_pos))?;
                        return Err(brec::Error::SignatureDismatch);
                    }
                    if len < #block_name::ssize() {
                        return Ok(brec::ReadStatus::NotEnoughData(#block_name::ssize() - len));
                    }
                    Ok(brec::ReadStatus::Success(#block_name::read(buf, true)?))
                }
            }
        })
    }
}

impl TryReadBuffered for Block {
    fn gen(&self) -> Result<TokenStream, E> {
        let block_name = self.name();
        let const_sig = self.const_sig_name();
        let sig_len = self.sig_len();
        Ok(quote! {

            impl brec::TryReadFromBuffered for #block_name {

                fn try_read<T: std::io::Read>(buf: &mut T) -> Result<brec::ReadStatus<Self>, brec::Error>
                where
                    Self: Sized,
                {
                    use std::io::BufRead;
                    use brec::prelude::*;

                    let mut reader = std::io::BufReader::new(buf);
                    let bytes = reader.fill_buf()?;

                    if bytes.len() < #sig_len {
                        return Ok(brec::ReadStatus::NotEnoughData(
                            (#sig_len - bytes.len()) as u64,
                        ));
                    }

                    if !bytes.starts_with(&#const_sig) {
                        return Err(brec::Error::SignatureDismatch);
                    }

                    if (bytes.len() as u64) < #block_name::ssize() {
                        return Ok(brec::ReadStatus::NotEnoughData(
                            #block_name::ssize() - bytes.len() as u64,
                        ));
                    }
                    reader.consume(#sig_len);
                    let blk = #block_name::read(&mut reader, true);
                    reader.consume(#block_name::ssize() as usize - #sig_len);
                    Ok(brec::ReadStatus::Success(blk?))
                }
            }
        })
    }
}
