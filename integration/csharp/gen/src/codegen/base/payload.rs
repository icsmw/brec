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
            fn to_csharp_object(&self) -> Result<brec::CSharpValue, brec::Error> {
                <#name as brec::CSharpConvert>::to_csharp_value(self)
            }

            fn from_csharp_object(value: brec::CSharpValue) -> Result<Self, brec::Error> {
                <#name as brec::CSharpConvert>::from_csharp_value(value)
            }

            pub fn decode_csharp(
                bytes: &[u8],
                ctx: &mut crate::PayloadContext<'_>,
            ) -> Result<brec::CSharpValue, brec::Error> {
                let mut cursor = std::io::Cursor::new(bytes);
                let header = <brec::PayloadHeader as brec::ReadFrom>::read(&mut cursor)?;
                let payload = <#name as brec::ReadPayloadFrom<#name>>::read(
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
                let mut payload = #name::from_csharp_object(value)?;
                brec::WritePayloadWithHeaderTo::write_all(&mut payload, out, ctx)?;
                Ok(())
            }
        }

        impl brec::CSharpObject for #name {
            fn to_csharp_object(&self) -> Result<brec::CSharpValue, brec::Error> {
                #name::to_csharp_object(self)
            }

            fn from_csharp_object(value: brec::CSharpValue) -> Result<Self, brec::Error> {
                #name::from_csharp_object(value)
            }
        }
    })
}
