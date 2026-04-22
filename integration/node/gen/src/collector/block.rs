use brec_gen_tys::*;
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
                let value = brec::NapiObject::to_napi_object(block, env)?;
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
                let block = <#fullpath as brec::NapiObject>::from_napi_object(&env, inner)?;
                return Ok(Block::#fullname(block));
            }
        });
    }

    Ok(quote! {
        impl Block {
            fn to_napi_object<'env>(&self, env: &'env napi::Env) -> Result<napi::Unknown<'env>, brec::Error> {
                use napi::bindgen_prelude::{JsObjectValue, JsValue, Object};
                let mut obj: Object<'env> = Object::new(env).map_err(|err| {
                    brec::Error::Napi(brec::NapiError::InvalidAggregatorShape(err.to_string()))
                })?;
                match self {
                    #(#to_wrapped)*
                }
                Ok(obj.to_unknown())
            }

            fn from_napi_object(value: napi::Unknown<'_>) -> Result<Self, brec::Error> {
                use napi::bindgen_prelude::{JsObjectValue, JsValue};
                let env = napi::Env::from_raw(value.value().env);
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
                    _ => Err(brec::Error::Napi(brec::NapiError::InvalidAggregatorShape(
                        format!("unknown block key: {key}"),
                    ))),
                }
            }

            pub fn decode_napi<'env>(
                env: &'env napi::Env,
                bytes: napi::bindgen_prelude::Buffer,
            ) -> Result<napi::Unknown<'env>, brec::Error> {
                let mut src = bytes.as_ref();
                let block = <Block as brec::ReadBlockFrom>::read(&mut src, false)?;
                block.to_napi_object(env)
            }

            pub fn encode_napi(value: napi::Unknown<'_>, out: &mut Vec<u8>) -> Result<(), brec::Error> {
                let block = Block::from_napi_object(value)?;
                brec::WriteTo::write_all(&block, out)?;
                Ok(())
            }
        }

        impl brec::NapiObject for Block {
            fn to_napi_object<'env>(&self, env: &'env napi::Env) -> Result<napi::Unknown<'env>, brec::Error> {
                Block::to_napi_object(self, env)
            }

            fn from_napi_object(env: &napi::Env, value: napi::Unknown<'_>) -> Result<Self, brec::Error> {
                let _ = env;
                Block::from_napi_object(value)
            }
        }
    })
}
