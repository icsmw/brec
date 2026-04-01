use crate::*;
use proc_macro2::TokenStream;
use quote::quote;

impl Payload {
    pub(crate) fn generate_napi(&self) -> Result<TokenStream, E> {
        if self.attrs.is_ctx() {
            return Ok(quote! {});
        }
        let payload_name = self.name();
        Ok(quote! {
            impl #payload_name {
                fn to_napi_object<'env>(&self, env: &'env napi::Env) -> Result<napi::Unknown<'env>, brec::Error> {
                    <#payload_name as brec::NapiConvert>::to_napi_value(self, env)
                }

                fn from_napi_object(env: &napi::Env, value: napi::Unknown<'_>) -> Result<Self, brec::Error> {
                    <#payload_name as brec::NapiConvert>::from_napi_value(env, value)
                }

                pub fn decode_napi<'env>(
                    env: &'env napi::Env,
                    bytes: napi::bindgen_prelude::Buffer,
                    ctx: &mut crate::PayloadContext<'_>,
                ) -> Result<napi::Unknown<'env>, brec::Error> {
                    let mut cursor = std::io::Cursor::new(bytes.as_ref());
                    let header = <brec::PayloadHeader as brec::ReadFrom>::read(&mut cursor)?;
                    let payload = <#payload_name as brec::ReadPayloadFrom<#payload_name>>::read(
                        &mut cursor,
                        &header,
                        ctx,
                    )?;
                    payload.to_napi_object(env)
                }

                pub fn encode_napi(
                    env: &napi::Env,
                    value: napi::Unknown<'_>,
                    out: &mut Vec<u8>,
                    ctx: &mut crate::PayloadContext<'_>,
                ) -> Result<(), brec::Error> {
                    let mut payload = #payload_name::from_napi_object(env, value)?;
                    brec::WritePayloadWithHeaderTo::write_all(&mut payload, out, ctx)?;
                    Ok(())
                }
            }

            impl brec::NapiObject for #payload_name {
                fn to_napi_object<'env>(&self, env: &'env napi::Env) -> Result<napi::Unknown<'env>, brec::Error> {
                    #payload_name::to_napi_object(self, env)
                }

                fn from_napi_object(env: &napi::Env, value: napi::Unknown<'_>) -> Result<Self, brec::Error> {
                    #payload_name::from_napi_object(env, value)
                }
            }
        })
    }
}
