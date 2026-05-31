use brec_macros_parser::*;
use proc_macro2::TokenStream;
use quote::quote;
use syn::Ident;

pub fn generate(name: &Ident, _attrs: &PayloadAttrs) -> Result<TokenStream, E> {
    Ok(quote! {
        impl #name {
            fn to_java_object<'local>(&self, env: &mut jni::Env<'local>) -> Result<jni::objects::JObject<'local>, brec::java_feat::JavaError> {
                <#name as brec::java_feat::JavaConvert>::to_java_value(self, env)
            }

            fn from_java_object<'local>(env: &mut jni::Env<'local>, value: jni::objects::JObject<'local>) -> Result<Self, brec::java_feat::JavaError> {
                <#name as brec::java_feat::JavaConvert>::from_java_value(env, value)
            }

            pub fn decode_java<'local>(
                env: &mut jni::Env<'local>,
                bytes: &[u8],
                ctx: &mut crate::ProtocolContext<'_>,
            ) -> Result<jni::objects::JObject<'local>, brec::Error> {
                let mut cursor = std::io::Cursor::new(bytes);
                let header = <brec::PayloadHeader as brec::ReadFrom>::read::<_, crate::Payload>(&mut cursor)?;
                let payload = <#name as brec::ReadPayloadFrom<#name>>::read(
                    &mut cursor,
                    &header,
                    ctx,
                )?;
                Ok(payload.to_java_object(env)?)
            }

            pub fn encode_java<'local>(
                env: &mut jni::Env<'local>,
                value: jni::objects::JObject<'local>,
                out: &mut Vec<u8>,
                ctx: &mut crate::ProtocolContext<'_>,
            ) -> Result<(), brec::Error> {
                let mut payload = #name::from_java_object(env, value)?;
                brec::WritePayloadWithHeaderTo::write_all(&mut payload, out, ctx)?;
                Ok(())
            }
        }

        impl brec::java_feat::JavaObject for #name {
            fn to_java_object<'local>(&self, env: &mut jni::Env<'local>) -> Result<jni::objects::JObject<'local>, brec::java_feat::JavaError> {
                #name::to_java_object(self, env)
            }

            fn from_java_object<'local>(env: &mut jni::Env<'local>, value: jni::objects::JObject<'local>) -> Result<Self, brec::java_feat::JavaError> {
                #name::from_java_object(env, value)
            }
        }
    })
}
