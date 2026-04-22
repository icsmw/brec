use brec_gen_tys::*;
use proc_macro2::{Ident, TokenStream};
use quote::quote;

pub fn generate(name: &Ident, attrs: &PayloadAttrs) -> Result<TokenStream, E> {
    if attrs.is_ctx() {
        return Ok(quote! {});
    }
    Ok(quote! {
        impl #name {
            fn to_napi_object<'env>(&self, env: &'env napi::Env) -> Result<napi::Unknown<'env>, brec::Error> {
                <#name as brec::NapiConvert>::to_napi_value(self, env)
            }

            fn from_napi_object(env: &napi::Env, value: napi::Unknown<'_>) -> Result<Self, brec::Error> {
                <#name as brec::NapiConvert>::from_napi_value(env, value)
            }

            pub fn decode_napi<'env>(
                env: &'env napi::Env,
                bytes: napi::bindgen_prelude::Buffer,
                ctx: &mut crate::PayloadContext<'_>,
            ) -> Result<napi::Unknown<'env>, brec::Error> {
                let mut cursor = std::io::Cursor::new(bytes.as_ref());
                let header = <brec::PayloadHeader as brec::ReadFrom>::read(&mut cursor)?;
                let payload = <#name as brec::ReadPayloadFrom<#name>>::read(
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
                let mut payload = #name::from_napi_object(env, value)?;
                brec::WritePayloadWithHeaderTo::write_all(&mut payload, out, ctx)?;
                Ok(())
            }
        }

        impl brec::NapiObject for #name {
            fn to_napi_object<'env>(&self, env: &'env napi::Env) -> Result<napi::Unknown<'env>, brec::Error> {
                #name::to_napi_object(self, env)
            }

            fn from_napi_object(env: &napi::Env, value: napi::Unknown<'_>) -> Result<Self, brec::Error> {
                #name::from_napi_object(env, value)
            }
        }
    })
}
