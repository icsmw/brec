use crate::*;
use proc_macro2::TokenStream;
use quote::quote;

impl Payload {
    pub(crate) fn generate_csharp(&self) -> Result<TokenStream, E> {
        if self.attrs.is_ctx() {
            return Ok(quote! {});
        }
        let payload_name = self.name();
        Ok(quote! {
            impl #payload_name {
                fn to_csharp_object(&self) -> Result<brec::CSharpValue, brec::Error> {
                    <#payload_name as brec::CSharpConvert>::to_csharp_value(self)
                }

                fn from_csharp_object(value: brec::CSharpValue) -> Result<Self, brec::Error> {
                    <#payload_name as brec::CSharpConvert>::from_csharp_value(value)
                }

                pub fn decode_csharp(
                    bytes: &[u8],
                    ctx: &mut crate::PayloadContext<'_>,
                ) -> Result<brec::CSharpValue, brec::Error> {
                    let mut cursor = std::io::Cursor::new(bytes);
                    let header = <brec::PayloadHeader as brec::ReadFrom>::read(&mut cursor)?;
                    let payload = <#payload_name as brec::ReadPayloadFrom<#payload_name>>::read(
                        &mut cursor,
                        &header,
                        ctx,
                    )?;
                    payload.to_csharp_object()
                }

                pub fn encode_csharp(
                    value: brec::CSharpValue,
                    out: &mut Vec<u8>,
                    ctx: &mut crate::PayloadContext<'_>,
                ) -> Result<(), brec::Error> {
                    let mut payload = #payload_name::from_csharp_object(value)?;
                    brec::WritePayloadWithHeaderTo::write_all(&mut payload, out, ctx)?;
                    Ok(())
                }
            }

            impl brec::CSharpObject for #payload_name {
                fn to_csharp_object(&self) -> Result<brec::CSharpValue, brec::Error> {
                    #payload_name::to_csharp_object(self)
                }

                fn from_csharp_object(value: brec::CSharpValue) -> Result<Self, brec::Error> {
                    #payload_name::from_csharp_object(value)
                }
            }
        })
    }
}
