use crate::*;
use proc_macro2::TokenStream;
use quote::quote;

impl Payload {
    pub(crate) fn generate_wasm(&self) -> Result<TokenStream, E> {
        if self.attrs.is_ctx() {
            return Ok(quote! {});
        }
        let payload_name = self.name();
        Ok(quote! {
            impl #payload_name {
                fn to_wasm_object(&self) -> Result<wasm_bindgen::JsValue, brec::Error> {
                    <#payload_name as brec::WasmConvert>::to_wasm_value(self)
                }

                fn from_wasm_object(value: wasm_bindgen::JsValue) -> Result<Self, brec::Error> {
                    <#payload_name as brec::WasmConvert>::from_wasm_value(value)
                }

                pub fn decode_wasm(
                    bytes: &[u8],
                    ctx: &mut crate::PayloadContext<'_>,
                ) -> Result<wasm_bindgen::JsValue, brec::Error> {
                    let mut cursor = std::io::Cursor::new(bytes);
                    let header = <brec::PayloadHeader as brec::ReadFrom>::read(&mut cursor)?;
                    let payload = <#payload_name as brec::ReadPayloadFrom<#payload_name>>::read(
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
                    let mut payload = #payload_name::from_wasm_object(value)?;
                    brec::WritePayloadWithHeaderTo::write_all(&mut payload, out, ctx)?;
                    Ok(())
                }
            }

            impl brec::WasmObject for #payload_name {
                fn to_wasm_object(&self) -> Result<wasm_bindgen::JsValue, brec::Error> {
                    #payload_name::to_wasm_object(self)
                }

                fn from_wasm_object(value: wasm_bindgen::JsValue) -> Result<Self, brec::Error> {
                    #payload_name::from_wasm_object(value)
                }
            }
        })
    }
}
