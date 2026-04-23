use brec_macros_parser::*;

use proc_macro2::TokenStream;
use quote::quote;
use syn::LitStr;

pub fn generate_impl(blocks: &[&Block]) -> Result<TokenStream, E> {
    let mut to_wrapped = Vec::new();
    let mut from_wrapped = Vec::new();
    for blk in blocks.iter() {
        let fullname = blk.fullname()?;
        let fullpath = blk.fullpath()?;
        let key = LitStr::new(&fullname.to_string(), proc_macro2::Span::call_site());
        to_wrapped.push(quote! {
            Block::#fullname(block) => {
                let value = brec::java_feat::JavaObject::to_java_object(block, env)?;
                brec::java_feat::map_put(env, &obj, #key, &value).map_err(|err| {
                    brec::java_feat::JavaError::InvalidAggregatorShape(err.to_string())
                })?;
            }
        });
        from_wrapped.push(quote! {
            #key => {
                let inner = brec::java_feat::map_get(env, &obj, #key).map_err(|err| {
                    brec::java_feat::JavaError::InvalidAggregatorShape(err.to_string())
                })?;
                let block = <#fullpath as brec::java_feat::JavaObject>::from_java_object(env, inner)?;
                return Ok(Block::#fullname(block));
            }
        });
    }

    Ok(quote! {
        impl Block {
            fn to_java_object<'local>(&self, env: &mut jni::JNIEnv<'local>) -> Result<jni::objects::JObject<'local>, brec::java_feat::JavaError> {
                let obj = brec::java_feat::new_hash_map(env).map_err(|err| {
                    brec::java_feat::JavaError::InvalidAggregatorShape(err.to_string())
                })?;
                match self {
                    #(#to_wrapped)*
                }
                Ok(obj)
            }

            fn from_java_object<'local>(env: &mut jni::JNIEnv<'local>, value: jni::objects::JObject<'local>) -> Result<Self, brec::java_feat::JavaError> {
                let obj = value;
                let (keys_len, key_opt) = brec::java_feat::map_keys_len_and_first(env, &obj).map_err(|err| {
                    brec::java_feat::JavaError::InvalidAggregatorShape(err.to_string())
                })?;
                if keys_len != 1 {
                    return Err(brec::java_feat::JavaError::InvalidAggregatorShape(
                        format!("expected object with exactly one field, got {}", keys_len),
                    ));
                }
                let key = key_opt.ok_or_else(|| {
                    brec::java_feat::JavaError::InvalidAggregatorShape(
                        "expected object key to be a string".to_owned(),
                    )
                })?;
                match key.as_str() {
                    #(#from_wrapped)*
                    _ => Err(brec::java_feat::JavaError::InvalidAggregatorShape(
                        format!("unknown block key: {key}"),
                    )),
                }
            }

            pub fn decode_java<'local>(
                env: &mut jni::JNIEnv<'local>,
                bytes: &[u8],
            ) -> Result<jni::objects::JObject<'local>, brec::Error> {
                let mut src = bytes;
                let block = <Block as brec::ReadBlockFrom>::read(&mut src, false)?;
                Ok(block.to_java_object(env)?)
            }

            pub fn encode_java<'local>(
                env: &mut jni::JNIEnv<'local>,
                value: jni::objects::JObject<'local>,
                out: &mut Vec<u8>,
            ) -> Result<(), brec::Error> {
                let block = Block::from_java_object(env, value)?;
                brec::WriteTo::write_all(&block, out)?;
                Ok(())
            }
        }

        impl brec::java_feat::JavaObject for Block {
            fn to_java_object<'local>(&self, env: &mut jni::JNIEnv<'local>) -> Result<jni::objects::JObject<'local>, brec::java_feat::JavaError> {
                Block::to_java_object(self, env)
            }

            fn from_java_object<'local>(env: &mut jni::JNIEnv<'local>, value: jni::objects::JObject<'local>) -> Result<Self, brec::java_feat::JavaError> {
                Block::from_java_object(env, value)
            }
        }
    })
}
