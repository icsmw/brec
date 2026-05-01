use brec_macros_parser::*;
use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use syn::{Ident, LitStr};

fn to_wasm_field_set(field: &BlockField) -> Result<TokenStream, E> {
    let rust_field = format_ident!("{}", field.name);
    let js_field = LitStr::new(&field.name, proc_macro2::Span::call_site());
    let value = match &field.ty {
        BlockTy::LinkedToU8(_) => quote! {{
            let value: u8 = (&self.#rust_field).into();
            <u8 as brec::wasm_feat::WasmConvert>::to_wasm_value(&value)?
        }},
        _ => {
            let ty = field.ty.direct();
            quote! { <#ty as brec::wasm_feat::WasmConvert>::to_wasm_value(&self.#rust_field)? }
        }
    };
    Ok(quote! {
        js_sys::Reflect::set(&obj, &wasm_bindgen::JsValue::from_str(#js_field), &#value)
            .map_err(|err| brec::wasm_feat::WasmError::invalid_field_name(#js_field, format!("{err:?}")))?;
    })
}

fn from_wasm_field_get(field: &BlockField) -> Result<TokenStream, E> {
    let rust_field = format_ident!("{}", field.name);
    let js_field = LitStr::new(&field.name, proc_macro2::Span::call_site());
    let ty = field.ty.direct();
    Ok(match &field.ty {
        BlockTy::LinkedToU8(enum_name) => quote! {
            let raw = js_sys::Reflect::get(&obj, &wasm_bindgen::JsValue::from_str(#js_field))
                .map_err(|err| brec::wasm_feat::WasmError::invalid_field_name(#js_field, format!("{err:?}")))?;
            let raw: u8 = <u8 as brec::wasm_feat::WasmConvert>::from_wasm_value(raw)?;
            let #rust_field = #ty::try_from(raw)
                .map_err(|err| brec::wasm_feat::WasmError::invalid_field_name(#enum_name, err))?;
        },
        _ => quote! {
            let raw = js_sys::Reflect::get(&obj, &wasm_bindgen::JsValue::from_str(#js_field))
                .map_err(|err| brec::wasm_feat::WasmError::invalid_field_name(#js_field, format!("{err:?}")))?;
            let #rust_field: #ty = <#ty as brec::wasm_feat::WasmConvert>::from_wasm_value(raw)?;
        },
    })
}

pub fn generate(block_name: &Ident, fields: &[BlockField]) -> Result<TokenStream, E> {
    let to_wasm = fields
        .iter()
        .filter(|field| !field.injected)
        .map(to_wasm_field_set)
        .collect::<Result<Vec<_>, _>>()?;
    let from_wasm = fields
        .iter()
        .filter(|field| !field.injected)
        .map(from_wasm_field_get)
        .collect::<Result<Vec<_>, _>>()?;
    let ctor_fields = fields
        .iter()
        .filter(|field| !field.injected)
        .map(|field| {
            let rust_field = format_ident!("{}", field.name);
            quote! { #rust_field, }
        })
        .collect::<Vec<_>>();

    Ok(quote! {
        impl #block_name {
            fn to_wasm_object(&self) -> Result<wasm_bindgen::JsValue, brec::wasm_feat::WasmError> {
                let obj = js_sys::Object::new();
                #(#to_wasm)*
                Ok(obj.into())
            }

            fn from_wasm_object(value: wasm_bindgen::JsValue) -> Result<Self, brec::wasm_feat::WasmError> {
                let obj: js_sys::Object = brec::wasm_feat::from_value_name("object", value)
                    .map_err(|err| brec::wasm_feat::WasmError::InvalidObject(err.to_string()))?;
                #(#from_wasm)*
                Ok(Self {
                    #(#ctor_fields)*
                })
            }

            pub fn decode_wasm(bytes: &[u8]) -> Result<wasm_bindgen::JsValue, brec::Error> {
                let mut src = bytes;
                let block = <#block_name as brec::ReadBlockFrom>::read(&mut src, false)?;
                Ok(block.to_wasm_object()?)
            }

            pub fn encode_wasm(value: wasm_bindgen::JsValue, out: &mut Vec<u8>) -> Result<(), brec::Error> {
                let block = #block_name::from_wasm_object(value)?;
                brec::WriteTo::write_all(&block, out)?;
                Ok(())
            }
        }

        impl brec::wasm_feat::WasmObject for #block_name {
            fn to_wasm_object(&self) -> Result<wasm_bindgen::JsValue, brec::wasm_feat::WasmError> {
                #block_name::to_wasm_object(self)
            }

            fn from_wasm_object(value: wasm_bindgen::JsValue) -> Result<Self, brec::wasm_feat::WasmError> {
                #block_name::from_wasm_object(value)
            }
        }
    })
}
