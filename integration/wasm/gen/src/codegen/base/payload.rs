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
            fn to_wasm_object(&self) -> Result<wasm_bindgen::JsValue, brec::Error> {
                <#name as brec::WasmConvert>::to_wasm_value(self)
            }

            fn from_wasm_object(value: wasm_bindgen::JsValue) -> Result<Self, brec::Error> {
                <#name as brec::WasmConvert>::from_wasm_value(value)
            }

            pub fn decode_wasm(
                bytes: &[u8],
                ctx: &mut crate::PayloadContext<'_>,
            ) -> Result<wasm_bindgen::JsValue, brec::Error> {
                let mut cursor = std::io::Cursor::new(bytes);
                let header = <brec::PayloadHeader as brec::ReadFrom>::read(&mut cursor)?;
                let payload = <#name as brec::ReadPayloadFrom<#name>>::read(
                    &mut cursor,
                    &header,
                    ctx,
                )?;
                payload.to_wasm_object()
            }

            pub fn encode_wasm(
                value: wasm_bindgen::JsValue,
                out: &mut Vec<u8>,
                ctx: &mut crate::PayloadContext<'_>,
            ) -> Result<(), brec::Error> {
                let mut payload = #name::from_wasm_object(value)?;
                brec::WritePayloadWithHeaderTo::write_all(&mut payload, out, ctx)?;
                Ok(())
            }
        }

        impl brec::WasmObject for #name {
            fn to_wasm_object(&self) -> Result<wasm_bindgen::JsValue, brec::Error> {
                #name::to_wasm_object(self)
            }

            fn from_wasm_object(value: wasm_bindgen::JsValue) -> Result<Self, brec::Error> {
                #name::from_wasm_object(value)
            }
        }
    })
}
