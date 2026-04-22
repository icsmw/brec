use brec_macros_parser::*;
use proc_macro2::TokenStream;
use quote::quote;
use syn::Ident;

pub fn generate(name: &Ident, attrs: &PayloadAttrs) -> Result<TokenStream, E> {
    if attrs.is_ctx() {
        return Ok(quote! {});
    }
    Ok(quote! {
        impl #name {
            fn to_java_object<'local>(&self, env: &mut jni::JNIEnv<'local>) -> Result<jni::objects::JObject<'local>, brec::Error> {
                <#name as brec::JavaConvert>::to_java_value(self, env)
            }

            fn from_java_object<'local>(env: &mut jni::JNIEnv<'local>, value: jni::objects::JObject<'local>) -> Result<Self, brec::Error> {
                <#name as brec::JavaConvert>::from_java_value(env, value)
            }

            pub fn decode_java<'local>(
                env: &mut jni::JNIEnv<'local>,
                bytes: &[u8],
                ctx: &mut crate::PayloadContext<'_>,
            ) -> Result<jni::objects::JObject<'local>, brec::Error> {
                let mut cursor = std::io::Cursor::new(bytes);
                let header = <brec::PayloadHeader as brec::ReadFrom>::read(&mut cursor)?;
                let payload = <#name as brec::ReadPayloadFrom<#name>>::read(
                    &mut cursor,
                    &header,
                    ctx,
                )?;
                payload.to_java_object(env)
            }

            pub fn encode_java<'local>(
                env: &mut jni::JNIEnv<'local>,
                value: jni::objects::JObject<'local>,
                out: &mut Vec<u8>,
                ctx: &mut crate::PayloadContext<'_>,
            ) -> Result<(), brec::Error> {
                let mut payload = #name::from_java_object(env, value)?;
                brec::WritePayloadWithHeaderTo::write_all(&mut payload, out, ctx)?;
                Ok(())
            }
        }

        impl brec::JavaObject for #name {
            fn to_java_object<'local>(&self, env: &mut jni::JNIEnv<'local>) -> Result<jni::objects::JObject<'local>, brec::Error> {
                #name::to_java_object(self, env)
            }

            fn from_java_object<'local>(env: &mut jni::JNIEnv<'local>, value: jni::objects::JObject<'local>) -> Result<Self, brec::Error> {
                #name::from_java_object(env, value)
            }
        }
    })
}
