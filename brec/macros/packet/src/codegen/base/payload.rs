use crate::*;
use proc_macro2::TokenStream;
use quote::quote;

impl Base for Payload {
    fn gen(&self) -> Result<TokenStream, E> {
        let payload_name = self.name();
        let sig = self.sig()?;
        let sig_impl = if self.attrs.no_default_sig() {
            quote! {}
        } else {
            quote! {

                impl brec::Signature for #payload_name {
                    fn sig() -> brec::ByteBlock {
                        brec::ByteBlock::Len4(#sig)
                    }
                }

            }
        };
        Ok(if self.attrs.is_bincode() {
            quote! {
                #sig_impl

                impl brec::PayloadEncode for #payload_name {
                    fn encode(&self) -> std::io::Result<Vec<u8>> {
                        brec::bincode::serialize(self)
                            .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e.to_string()))
                    }
                }

                impl brec::PayloadEncodeReferred for #payload_name {
                    fn encode(&self) -> std::io::Result<Option<&[u8]>> {
                        Ok(None)
                    }
                }

                impl brec::PayloadDecode<#payload_name> for #payload_name {
                    fn decode(buf: &[u8]) -> std::io::Result<#payload_name> {
                        brec::bincode::deserialize(buf)
                            .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e.to_string()))
                    }
                }
            }
        } else {
            sig_impl
        })
    }
}

impl Gen for Payload {
    fn gen(&self) -> Result<TokenStream, E> {
        let base = Base::gen(self)?;
        let read = Read::gen(self)?;
        let try_read = TryRead::gen(self)?;
        let try_read_buffered = TryReadBuffered::gen(self)?;
        let crc = Crc::gen(self)?;
        let size = Size::gen(self);
        let write = Write::gen(self)?;
        let write_vec = WriteVectored::gen(self)?;
        Ok(quote! {
            #base
            #crc
            #size
            #read
            #try_read
            #try_read_buffered
            #write
            #write_vec
        })
    }
}
