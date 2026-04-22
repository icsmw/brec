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
                    let value = brec::WasmObject::to_wasm_object(payload)?;
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
                    let payload = <#fullpath as brec::WasmObject>::from_wasm_object(inner)?;
                    return Ok(Payload::#fullname(payload));
                }
            });
        } else {
            to_wrapped.push(quote! {
                Payload::#fullname(_) => {
                    return Err(brec::Error::Wasm(brec::WasmError::InvalidAggregatorShape(
                        format!("payload variant {} requires #[payload(bincode)] for wasm MVP", #key),
                    )));
                }
            });
            from_wrapped.push(quote! {
                #key => {
                    return Err(brec::Error::Wasm(brec::WasmError::InvalidAggregatorShape(
                        format!("payload variant {} requires #[payload(bincode)] for wasm MVP", #key),
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
                    let value = <Vec<u8> as brec::WasmConvert>::to_wasm_value(payload)?;
                    js_sys::Reflect::set(&obj, &wasm_bindgen::JsValue::from_str("Bytes"), &value).map_err(|err| {
                        brec::Error::Wasm(brec::WasmError::InvalidAggregatorShape(format!("{err:?}")))
                    })?;
                }
                Payload::String(payload) => {
                    let value = <String as brec::WasmConvert>::to_wasm_value(payload)?;
                    js_sys::Reflect::set(&obj, &wasm_bindgen::JsValue::from_str("String"), &value).map_err(|err| {
                        brec::Error::Wasm(brec::WasmError::InvalidAggregatorShape(format!("{err:?}")))
                    })?;
                }
            },
            quote! {
                "Bytes" => {
                    let inner = js_sys::Reflect::get(&obj, &wasm_bindgen::JsValue::from_str("Bytes")).map_err(|err| {
                        brec::Error::Wasm(brec::WasmError::InvalidAggregatorShape(format!("{err:?}")))
                    })?;
                    let payload = <Vec<u8> as brec::WasmConvert>::from_wasm_value(inner)?;
                    return Ok(Payload::Bytes(payload));
                }
                "String" => {
                    let inner = js_sys::Reflect::get(&obj, &wasm_bindgen::JsValue::from_str("String")).map_err(|err| {
                        brec::Error::Wasm(brec::WasmError::InvalidAggregatorShape(format!("{err:?}")))
                    })?;
                    let payload = <String as brec::WasmConvert>::from_wasm_value(inner)?;
                    return Ok(Payload::String(payload));
                }
            },
        )
    };

    Ok(quote! {
        impl Payload {
            fn to_wasm_object(&self) -> Result<wasm_bindgen::JsValue, brec::Error> {
                let obj = js_sys::Object::new();
                match self {
                    #(#to_wrapped)*
                    #defaults_to
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
                    #defaults_from
                    _ => Err(brec::Error::Wasm(brec::WasmError::InvalidAggregatorShape(
                        format!("unknown payload key: {key}"),
                    ))),
                }
            }

            pub fn decode_wasm(
                bytes: &[u8],
                ctx: &mut crate::PayloadContext<'_>,
            ) -> Result<wasm_bindgen::JsValue, brec::Error> {
                let mut cursor = std::io::Cursor::new(bytes);
                let header = <brec::PayloadHeader as brec::ReadFrom>::read(&mut cursor)?;
                let payload = <Payload as brec::ExtractPayloadFrom<Payload>>::read(&mut cursor, &header, ctx)?;
                payload.to_wasm_object()
            }

            pub fn encode_wasm(
                value: wasm_bindgen::JsValue,
                out: &mut Vec<u8>,
                ctx: &mut crate::PayloadContext<'_>,
            ) -> Result<(), brec::Error> {
                let mut payload = Payload::from_wasm_object(value)?;
                brec::WriteMutTo::write_all(&mut payload, out, ctx)?;
                Ok(())
            }
        }

        impl brec::WasmObject for Payload {
            fn to_wasm_object(&self) -> Result<wasm_bindgen::JsValue, brec::Error> {
                Payload::to_wasm_object(self)
            }

            fn from_wasm_object(value: wasm_bindgen::JsValue) -> Result<Self, brec::Error> {
                Payload::from_wasm_object(value)
            }
        }
    })
}
