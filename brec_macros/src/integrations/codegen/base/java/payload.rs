use crate::*;
use proc_macro2::TokenStream;
use quote::quote;

impl Payload {
    pub(crate) fn generate_java(&self) -> Result<TokenStream, E> {
        if self.attrs.is_ctx() {
            return Ok(quote! {});
        }
        let payload_name = self.name();
        Ok(quote! {
            impl #payload_name {
                fn to_java_object<'local>(&self, env: &mut jni::JNIEnv<'local>) -> Result<jni::objects::JObject<'local>, brec::Error> {
                    <#payload_name as brec::JavaConvert>::to_java_value(self, env)
                }

                fn from_java_object<'local>(env: &mut jni::JNIEnv<'local>, value: jni::objects::JObject<'local>) -> Result<Self, brec::Error> {
                    <#payload_name as brec::JavaConvert>::from_java_value(env, value)
                }

                pub fn decode_java<'local>(
                    env: &mut jni::JNIEnv<'local>,
                    bytes: &[u8],
                    ctx: &mut crate::PayloadContext<'_>,
                ) -> Result<jni::objects::JObject<'local>, brec::Error> {
                    let mut cursor = std::io::Cursor::new(bytes);
                    let header = <brec::PayloadHeader as brec::ReadFrom>::read(&mut cursor)?;
                    let payload = <#payload_name as brec::ReadPayloadFrom<#payload_name>>::read(
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
                    let mut payload = #payload_name::from_java_object(env, value)?;
                    brec::WritePayloadWithHeaderTo::write_all(&mut payload, out, ctx)?;
                    Ok(())
                }
            }

            impl brec::JavaObject for #payload_name {
                fn to_java_object<'local>(&self, env: &mut jni::JNIEnv<'local>) -> Result<jni::objects::JObject<'local>, brec::Error> {
                    #payload_name::to_java_object(self, env)
                }

                fn from_java_object<'local>(env: &mut jni::JNIEnv<'local>, value: jni::objects::JObject<'local>) -> Result<Self, brec::Error> {
                    #payload_name::from_java_object(env, value)
                }
            }
        })
    }
}
