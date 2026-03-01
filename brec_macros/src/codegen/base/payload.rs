use crate::*;
use proc_macro2::TokenStream;
use quote::quote;

impl Base for Payload {
    fn generate(&self) -> Result<TokenStream, E> {
        let payload_name = self.name();
        let sig = self.sig()?;
        let sig_impl = if self.attrs.no_default_sig() {
            quote! {}
        } else {
            quote! {

                impl brec::PayloadSignature for #payload_name {
                    fn sig(&self) -> brec::ByteBlock {
                        brec::ByteBlock::Len4(#sig)
                    }
                }
                impl brec::StaticPayloadSignature for #payload_name {
                    fn ssig() -> brec::ByteBlock {
                        brec::ByteBlock::Len4(#sig)
                    }
                }

            }
        };

        let hooks_impl = if self.attrs.hooks() {
            quote! {}
        } else {
            quote! {
                impl brec::PayloadHooks for #payload_name { }
            }
        };
        Ok(if self.attrs.is_bincode() {
            quote! {
                #sig_impl

                #hooks_impl

                impl brec::PayloadEncode for #payload_name {
                    fn encode(&self) -> std::io::Result<Vec<u8>> {
                        brec::bincode::serde::encode_to_vec(self, brec::bincode::config::standard())
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
                        brec::bincode::serde::decode_from_slice(buf, brec::bincode::config::standard())
                            .map(|(value, _)| value)
                            .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e.to_string()))
                    }
                }
            }
        } else {
            quote! {
                #sig_impl
                #hooks_impl

            }
        })
    }
}

impl Gen for Payload {
    fn generate(&self) -> Result<TokenStream, E> {
        let base = Base::generate(self)?;
        let read = Read::generate(self)?;
        let try_read = TryRead::generate(self)?;
        let try_read_buffered = TryReadBuffered::generate(self)?;
        let crc = Crc::generate(self)?;
        let size = Size::generate(self);
        let write = Write::generate(self)?;
        let write_vec = WriteVectored::generate(self)?;
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
