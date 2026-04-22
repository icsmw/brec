use brec_macros_parser::*;

use proc_macro2::TokenStream;
use quote::quote;
use syn::LitStr;

pub fn generate_impl(payloads: &[&Payload], cfg: &Config) -> Result<TokenStream, E> {
    let mut to_wrapped = Vec::new();
    let mut from_wrapped = Vec::new();
    for payload in payloads.iter().filter(|pl| !pl.attrs.is_ctx()) {
        let fullname = payload.fullname()?;
        let fullpath = payload.fullpath()?;
        let key = LitStr::new(&fullname.to_string(), proc_macro2::Span::call_site());
        if payload.attrs.is_bincode() {
            to_wrapped.push(quote! {
                Payload::#fullname(payload) => {
                    let value = brec::JavaObject::to_java_object(payload, env)?;
                    brec::java_feature::map_put(env, &obj, #key, &value).map_err(|err| {
                        brec::Error::Java(brec::JavaError::InvalidAggregatorShape(err.to_string()))
                    })?;
                }
            });
            from_wrapped.push(quote! {
                #key => {
                    let inner = brec::java_feature::map_get(env, &obj, #key).map_err(|err| {
                        brec::Error::Java(brec::JavaError::InvalidAggregatorShape(err.to_string()))
                    })?;
                    let payload = <#fullpath as brec::JavaObject>::from_java_object(env, inner)?;
                    return Ok(Payload::#fullname(payload));
                }
            });
        } else {
            to_wrapped.push(quote! {
                Payload::#fullname(_) => {
                    return Err(brec::Error::Java(brec::JavaError::InvalidAggregatorShape(
                        format!("payload variant {} requires #[payload(bincode)] for java MVP", #key),
                    )));
                }
            });
            from_wrapped.push(quote! {
                #key => {
                    return Err(brec::Error::Java(brec::JavaError::InvalidAggregatorShape(
                        format!("payload variant {} requires #[payload(bincode)] for java MVP", #key),
                    )));
                }
            });
        }
    }

    let (defaults_to, defaults_from) = if cfg.is_no_default_payloads() {
        (quote! {}, quote! {})
    } else {
        (
            quote! {
                Payload::Bytes(payload) => {
                    let value = <Vec<u8> as brec::JavaConvert>::to_java_value(payload, env)?;
                    brec::java_feature::map_put(env, &obj, "Bytes", &value).map_err(|err| {
                        brec::Error::Java(brec::JavaError::InvalidAggregatorShape(err.to_string()))
                    })?;
                }
                Payload::String(payload) => {
                    let value = <String as brec::JavaConvert>::to_java_value(payload, env)?;
                    brec::java_feature::map_put(env, &obj, "String", &value).map_err(|err| {
                        brec::Error::Java(brec::JavaError::InvalidAggregatorShape(err.to_string()))
                    })?;
                }
            },
            quote! {
                "Bytes" => {
                    let inner = brec::java_feature::map_get(env, &obj, "Bytes").map_err(|err| {
                        brec::Error::Java(brec::JavaError::InvalidAggregatorShape(err.to_string()))
                    })?;
                    let payload = <Vec<u8> as brec::JavaConvert>::from_java_value(env, inner)?;
                    return Ok(Payload::Bytes(payload));
                }
                "String" => {
                    let inner = brec::java_feature::map_get(env, &obj, "String").map_err(|err| {
                        brec::Error::Java(brec::JavaError::InvalidAggregatorShape(err.to_string()))
                    })?;
                    let payload = <String as brec::JavaConvert>::from_java_value(env, inner)?;
                    return Ok(Payload::String(payload));
                }
            },
        )
    };

    Ok(quote! {
        impl Payload {
            fn to_java_object<'local>(&self, env: &mut jni::JNIEnv<'local>) -> Result<jni::objects::JObject<'local>, brec::Error> {
                let obj = brec::java_feature::new_hash_map(env).map_err(|err| {
                    brec::Error::Java(brec::JavaError::InvalidAggregatorShape(err.to_string()))
                })?;
                match self {
                    #(#to_wrapped)*
                    #defaults_to
                }
                Ok(obj)
            }

            fn from_java_object<'local>(env: &mut jni::JNIEnv<'local>, value: jni::objects::JObject<'local>) -> Result<Self, brec::Error> {
                let obj = value;
                let (keys_len, key_opt) = brec::java_feature::map_keys_len_and_first(env, &obj).map_err(|err| {
                    brec::Error::Java(brec::JavaError::InvalidAggregatorShape(err.to_string()))
                })?;
                if keys_len != 1 {
                    return Err(brec::Error::Java(brec::JavaError::InvalidAggregatorShape(
                        format!("expected object with exactly one field, got {}", keys_len),
                    )));
                }
                let key = key_opt.ok_or_else(|| {
                    brec::Error::Java(brec::JavaError::InvalidAggregatorShape(
                        "expected object key to be a string".to_owned(),
                    ))
                })?;
                match key.as_str() {
                    #(#from_wrapped)*
                    #defaults_from
                    _ => Err(brec::Error::Java(brec::JavaError::InvalidAggregatorShape(
                        format!("unknown payload key: {key}"),
                    ))),
                }
            }

            pub fn decode_java<'local>(
                env: &mut jni::JNIEnv<'local>,
                bytes: &[u8],
                ctx: &mut crate::PayloadContext<'_>,
            ) -> Result<jni::objects::JObject<'local>, brec::Error> {
                let mut cursor = std::io::Cursor::new(bytes);
                let header = <brec::PayloadHeader as brec::ReadFrom>::read(&mut cursor)?;
                let payload = <Payload as brec::ExtractPayloadFrom<Payload>>::read(&mut cursor, &header, ctx)?;
                payload.to_java_object(env)
            }

            pub fn encode_java<'local>(
                env: &mut jni::JNIEnv<'local>,
                value: jni::objects::JObject<'local>,
                out: &mut Vec<u8>,
                ctx: &mut crate::PayloadContext<'_>,
            ) -> Result<(), brec::Error> {
                let mut payload = Payload::from_java_object(env, value)?;
                brec::WriteMutTo::write_all(&mut payload, out, ctx)?;
                Ok(())
            }
        }

        impl brec::JavaObject for Payload {
            fn to_java_object<'local>(&self, env: &mut jni::JNIEnv<'local>) -> Result<jni::objects::JObject<'local>, brec::Error> {
                Payload::to_java_object(self, env)
            }

            fn from_java_object<'local>(env: &mut jni::JNIEnv<'local>, value: jni::objects::JObject<'local>) -> Result<Self, brec::Error> {
                Payload::from_java_object(env, value)
            }
        }
    })
}
