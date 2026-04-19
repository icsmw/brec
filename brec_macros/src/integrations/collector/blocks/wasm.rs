use crate::*;

use proc_macro2::TokenStream;
use quote::quote;
use syn::LitStr;

pub(crate) fn generate_impl(blocks: &[&Block]) -> Result<TokenStream, E> {
    let mut to_wrapped = Vec::new();
    let mut from_wrapped = Vec::new();
    for blk in blocks.iter() {
        let fullname = blk.fullname()?;
        let fullpath = blk.fullpath()?;
        let key = LitStr::new(&fullname.to_string(), proc_macro2::Span::call_site());
        to_wrapped.push(quote! {
            Block::#fullname(block) => {
                let value = brec::WasmObject::to_wasm_object(block)?;
                js_sys::Reflect::set(&obj, &wasm_bindgen::JsValue::from_str(#key), &value).map_err(|err| {
                    brec::Error::Wasm(brec::WasmError::InvalidAggregatorShape(format!("{err:?}")))
                })?;
            }
        });
        from_wrapped.push(quote! {
            #key => {
                let inner = js_sys::Reflect::get(&obj, &wasm_bindgen::JsValue::from_str(#key)).map_err(|err| {
                    brec::Error::Wasm(brec::WasmError::InvalidAggregatorShape(format!("{err:?}")))
                })?;
                let block = <#fullpath as brec::WasmObject>::from_wasm_object(inner)?;
                return Ok(Block::#fullname(block));
            }
        });
    }

    Ok(quote! {
        impl Block {
            fn to_wasm_object(&self) -> Result<wasm_bindgen::JsValue, brec::Error> {
                let obj = js_sys::Object::new();
                match self {
                    #(#to_wrapped)*
                }
                Ok(obj.into())
            }

            fn from_wasm_object(value: wasm_bindgen::JsValue) -> Result<Self, brec::Error> {
                let obj: js_sys::Object = brec::wasm_feature::from_value_name("object", value)
                    .map_err(|err| brec::Error::Wasm(brec::WasmError::InvalidAggregatorShape(err.to_string())))?;
                let keys = js_sys::Object::keys(&obj);
                let keys_len = keys.length();
                if keys_len != 1 {
                    return Err(brec::Error::Wasm(brec::WasmError::InvalidAggregatorShape(
                        format!("expected object with exactly one field, got {}", keys_len),
                    )));
                }
                let key = keys.get(0).as_string().ok_or_else(|| {
                    brec::Error::Wasm(brec::WasmError::InvalidAggregatorShape(
                        "expected object key to be a string".to_owned(),
                    ))
                })?;
                match key.as_str() {
                    #(#from_wrapped)*
                    _ => Err(brec::Error::Wasm(brec::WasmError::InvalidAggregatorShape(
                        format!("unknown block key: {key}"),
                    ))),
                }
            }

            pub fn decode_wasm(bytes: &[u8]) -> Result<wasm_bindgen::JsValue, brec::Error> {
                let mut src = bytes;
                let block = <Block as brec::ReadBlockFrom>::read(&mut src, false)?;
                block.to_wasm_object()
            }

            pub fn encode_wasm(value: wasm_bindgen::JsValue, out: &mut Vec<u8>) -> Result<(), brec::Error> {
                let block = Block::from_wasm_object(value)?;
                brec::WriteTo::write_all(&block, out)?;
                Ok(())
            }
        }

        impl brec::WasmObject for Block {
            fn to_wasm_object(&self) -> Result<wasm_bindgen::JsValue, brec::Error> {
                Block::to_wasm_object(self)
            }

            fn from_wasm_object(value: wasm_bindgen::JsValue) -> Result<Self, brec::Error> {
                Block::from_wasm_object(value)
            }
        }
    })
}
