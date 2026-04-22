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
        let schema_impl = quote! {
            impl brec::PayloadSchema for #payload_name {
                type Context<'a> = crate::PayloadContext<'a>;
            }
        };
        let bincode_impl = if self.attrs.is_bincode() && !self.attrs.is_crypt() {
            quote! {
                impl brec::PayloadEncode for #payload_name {
                    fn encode(&self, _ctx: &mut Self::Context<'_>) -> std::io::Result<Vec<u8>> {
                        brec::bincode::serde::encode_to_vec(self, brec::bincode::config::standard())
                            .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e.to_string()))
                    }
                }

                impl brec::PayloadEncodeReferred for #payload_name {
                    fn encode(&self, _ctx: &mut Self::Context<'_>) -> std::io::Result<Option<&[u8]>> {
                        Ok(None)
                    }
                }

                impl brec::PayloadDecode<#payload_name> for #payload_name {
                    fn decode(buf: &[u8], _ctx: &mut Self::Context<'_>) -> std::io::Result<#payload_name> {
                        brec::bincode::serde::decode_from_slice(buf, brec::bincode::config::standard())
                            .map(|(value, _)| value)
                            .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e.to_string()))
                    }
                }
            }
        } else {
            quote! {}
        };
        let crypt_and_bincode_impl = if self.attrs.is_crypt() && self.attrs.is_bincode() {
            quote! {
                impl brec::PayloadEncode for #payload_name {
                    fn encode(&self, ctx: &mut Self::Context<'_>) -> std::io::Result<Vec<u8>> {
                        let payload_body = brec::bincode::serde::encode_to_vec(self, brec::bincode::config::standard())
                            .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e.to_string()))?;
                        let encrypt_options = match ctx {
                            crate::PayloadContext::Encrypt(opt) => opt,
                            _ => {
                                return Err(std::io::Error::new(
                                    std::io::ErrorKind::InvalidInput,
                                    format!(
                                        "payload {} with #[payload(crypt, bincode)] expects PayloadContext::Encrypt",
                                        stringify!(#payload_name),
                                    ),
                                ));
                            }
                        };
                        brec::BricCryptCodec::encrypt(&payload_body, encrypt_options).map_err(std::io::Error::from)
                    }
                }

                impl brec::PayloadEncodeReferred for #payload_name {
                    fn encode(&self, _ctx: &mut Self::Context<'_>) -> std::io::Result<Option<&[u8]>> {
                        Ok(None)
                    }
                }

                impl brec::PayloadDecode<#payload_name> for #payload_name {
                    fn decode(buf: &[u8], ctx: &mut Self::Context<'_>) -> std::io::Result<#payload_name> {
                        let decrypt_options = match ctx {
                            crate::PayloadContext::Decrypt(opt) => opt,
                            _ => {
                                return Err(std::io::Error::new(
                                    std::io::ErrorKind::InvalidInput,
                                    format!(
                                        "payload {} with #[payload(crypt, bincode)] expects PayloadContext::Decrypt",
                                        stringify!(#payload_name),
                                    ),
                                ));
                            }
                        };
                        let payload_body = brec::BricCryptCodec::decrypt(buf, decrypt_options)
                            .map_err(std::io::Error::from)?;
                        brec::bincode::serde::decode_from_slice(&payload_body, brec::bincode::config::standard())
                            .map(|(value, _)| value)
                            .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e.to_string()))
                    }
                }
            }
        } else {
            quote! {}
        };
        let napi_impl = if cfg!(feature = "napi") {
            integrations::codegen::base::napi::payload::generate_napi(&self.name(), &self.attrs)?
        } else {
            quote! {}
        };
        let wasm_impl = if cfg!(feature = "wasm") {
            integrations::codegen::base::wasm::payload::generate_wasm(&self.name(), &self.attrs)?
        } else {
            quote! {}
        };
        let java_impl = if cfg!(feature = "java") {
            integrations::codegen::base::java::payload::generate_java(&self.name(), &self.attrs)?
        } else {
            quote! {}
        };
        let csharp_impl = if cfg!(feature = "csharp") {
            integrations::codegen::base::csharp::payload::generate_csharp(
                &self.name(),
                &self.attrs,
            )?
        } else {
            quote! {}
        };
        Ok(quote! {
            #sig_impl
            #hooks_impl
            #schema_impl
            #bincode_impl
            #crypt_and_bincode_impl
            #napi_impl
            #wasm_impl
            #java_impl
            #csharp_impl
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
