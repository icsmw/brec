use brec_gen_tys::*;
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
                    let value = brec::NapiObject::to_napi_object(payload, env)?;
                    obj.set_named_property(#key, value).map_err(|err| {
                        brec::Error::Napi(brec::NapiError::InvalidAggregatorShape(err.to_string()))
                    })?;
                }
            });
            from_wrapped.push(quote! {
                #key => {
                    let inner: napi::Unknown<'_> = obj.get_named_property(#key).map_err(|err| {
                        brec::Error::Napi(brec::NapiError::InvalidAggregatorShape(err.to_string()))
                    })?;
                    let payload = <#fullpath as brec::NapiObject>::from_napi_object(env, inner)?;
                    return Ok(Payload::#fullname(payload));
                }
            });
        } else {
            to_wrapped.push(quote! {
                Payload::#fullname(_) => {
                    return Err(brec::Error::Napi(brec::NapiError::InvalidAggregatorShape(
                        format!("payload variant {} requires #[payload(bincode)] for napi MVP", #key),
                    )));
                }
            });
            from_wrapped.push(quote! {
                #key => {
                    return Err(brec::Error::Napi(brec::NapiError::InvalidAggregatorShape(
                        format!("payload variant {} requires #[payload(bincode)] for napi MVP", #key),
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
                    let value = env.to_js_value(payload).map_err(|err| {
                        brec::Error::Napi(brec::NapiError::InvalidAggregatorShape(err.to_string()))
                    })?;
                    obj.set_named_property("Bytes", value).map_err(|err| {
                        brec::Error::Napi(brec::NapiError::InvalidAggregatorShape(err.to_string()))
                    })?;
                }
                Payload::String(payload) => {
                    let value = env.to_js_value(payload).map_err(|err| {
                        brec::Error::Napi(brec::NapiError::InvalidAggregatorShape(err.to_string()))
                    })?;
                    obj.set_named_property("String", value).map_err(|err| {
                        brec::Error::Napi(brec::NapiError::InvalidAggregatorShape(err.to_string()))
                    })?;
                }
            },
            quote! {
                "Bytes" => {
                    let inner: napi::Unknown<'_> = obj.get_named_property("Bytes").map_err(|err| {
                        brec::Error::Napi(brec::NapiError::InvalidAggregatorShape(err.to_string()))
                    })?;
                    let payload = env.from_js_value::<Vec<u8>, _>(inner).map_err(|err| {
                        brec::Error::Napi(brec::NapiError::InvalidAggregatorShape(err.to_string()))
                    })?;
                    return Ok(Payload::Bytes(payload));
                }
                "String" => {
                    let inner: napi::Unknown<'_> = obj.get_named_property("String").map_err(|err| {
                        brec::Error::Napi(brec::NapiError::InvalidAggregatorShape(err.to_string()))
                    })?;
                    let payload = env.from_js_value::<String, _>(inner).map_err(|err| {
                        brec::Error::Napi(brec::NapiError::InvalidAggregatorShape(err.to_string()))
                    })?;
                    return Ok(Payload::String(payload));
                }
            },
        )
    };

    Ok(quote! {
        impl Payload {
            fn to_napi_object<'env>(&self, env: &'env napi::Env) -> Result<napi::Unknown<'env>, brec::Error> {
                use napi::bindgen_prelude::{JsObjectValue, JsValue, Object};
                let mut obj: Object<'env> = Object::new(env).map_err(|err| {
                    brec::Error::Napi(brec::NapiError::InvalidAggregatorShape(err.to_string()))
                })?;
                match self {
                    #(#to_wrapped)*
                    #defaults_to
                }
                Ok(obj.to_unknown())
            }

            fn from_napi_object(env: &napi::Env, value: napi::Unknown<'_>) -> Result<Self, brec::Error> {
                use napi::bindgen_prelude::JsObjectValue;
                let obj: napi::bindgen_prelude::Object<'_> =
                    brec::napi_feature::from_unknown_name("object", value)
                        .map_err(|err| brec::Error::Napi(brec::NapiError::InvalidAggregatorShape(err.to_string())))?;
                let keys = obj.get_property_names().map_err(|err| {
                    brec::Error::Napi(brec::NapiError::InvalidAggregatorShape(err.to_string()))
                })?;
                let keys_len = keys.get_array_length().map_err(|err| {
                    brec::Error::Napi(brec::NapiError::InvalidAggregatorShape(err.to_string()))
                })?;
                if keys_len != 1 {
                    return Err(brec::Error::Napi(brec::NapiError::InvalidAggregatorShape(
                        format!("expected object with exactly one field, got {}", keys_len),
                    )));
                }
                let key: String = keys.get_element(0).map_err(|err| {
                    brec::Error::Napi(brec::NapiError::InvalidAggregatorShape(err.to_string()))
                })?;
                match key.as_str() {
                    #(#from_wrapped)*
                    #defaults_from
                    _ => Err(brec::Error::Napi(brec::NapiError::InvalidAggregatorShape(
                        format!("unknown payload key: {key}"),
                    ))),
                }
            }

            pub fn decode_napi<'env>(
                env: &'env napi::Env,
                bytes: napi::bindgen_prelude::Buffer,
                ctx: &mut crate::PayloadContext<'_>,
            ) -> Result<napi::Unknown<'env>, brec::Error> {
                let mut cursor = std::io::Cursor::new(bytes.as_ref());
                let header = <brec::PayloadHeader as brec::ReadFrom>::read(&mut cursor)?;
                let payload = <Payload as brec::ExtractPayloadFrom<Payload>>::read(&mut cursor, &header, ctx)?;
                payload.to_napi_object(env)
            }

            pub fn encode_napi(
                env: &napi::Env,
                value: napi::Unknown<'_>,
                out: &mut Vec<u8>,
                ctx: &mut crate::PayloadContext<'_>,
            ) -> Result<(), brec::Error> {
                let mut payload = Payload::from_napi_object(env, value)?;
                brec::WriteMutTo::write_all(&mut payload, out, ctx)?;
                Ok(())
            }
        }

        impl brec::NapiObject for Payload {
            fn to_napi_object<'env>(&self, env: &'env napi::Env) -> Result<napi::Unknown<'env>, brec::Error> {
                Payload::to_napi_object(self, env)
            }

            fn from_napi_object(env: &napi::Env, value: napi::Unknown<'_>) -> Result<Self, brec::Error> {
                Payload::from_napi_object(env, value)
            }
        }
    })
}
