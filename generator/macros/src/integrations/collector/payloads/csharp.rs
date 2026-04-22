use crate::*;

use proc_macro2::TokenStream;
use quote::quote;
use syn::LitStr;

pub(crate) fn generate_impl(payloads: &[&Payload], cfg: &Config) -> Result<TokenStream, E> {
    let mut to_wrapped = Vec::new();
    let mut from_wrapped = Vec::new();
    for payload in payloads.iter().filter(|pl| !pl.attrs.is_ctx()) {
        let fullname = payload.fullname()?;
        let fullpath = payload.fullpath()?;
        let key = LitStr::new(&fullname.to_string(), proc_macro2::Span::call_site());
        if payload.attrs.is_bincode() {
            to_wrapped.push(quote! {
                Payload::#fullname(payload) => {
                    let value = brec::CSharpObject::to_csharp_object(payload)?;
                    brec::csharp_feature::map_put(&mut obj, #key, value).map_err(|err| {
                        brec::Error::CSharp(brec::CSharpError::InvalidAggregatorShape(err.to_string()))
                    })?;
                }
            });
            from_wrapped.push(quote! {
                #key => {
                    let payload = <#fullpath as brec::CSharpObject>::from_csharp_object(inner)?;
                    return Ok(Payload::#fullname(payload));
                }
            });
        } else {
            to_wrapped.push(quote! {
                Payload::#fullname(_) => {
                    return Err(brec::Error::CSharp(brec::CSharpError::InvalidAggregatorShape(
                        format!("payload variant {} requires #[payload(bincode)] for csharp MVP", #key),
                    )));
                }
            });
            from_wrapped.push(quote! {
                #key => {
                    return Err(brec::Error::CSharp(brec::CSharpError::InvalidAggregatorShape(
                        format!("payload variant {} requires #[payload(bincode)] for csharp MVP", #key),
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
                    let value = <Vec<u8> as brec::CSharpConvert>::to_csharp_value(payload)?;
                    brec::csharp_feature::map_put(&mut obj, "Bytes", value).map_err(|err| {
                        brec::Error::CSharp(brec::CSharpError::InvalidAggregatorShape(err.to_string()))
                    })?;
                }
                Payload::String(payload) => {
                    let value = <String as brec::CSharpConvert>::to_csharp_value(payload)?;
                    brec::csharp_feature::map_put(&mut obj, "String", value).map_err(|err| {
                        brec::Error::CSharp(brec::CSharpError::InvalidAggregatorShape(err.to_string()))
                    })?;
                }
            },
            quote! {
                "Bytes" => {
                    let payload = <Vec<u8> as brec::CSharpConvert>::from_csharp_value(inner)?;
                    return Ok(Payload::Bytes(payload));
                }
                "String" => {
                    let payload = <String as brec::CSharpConvert>::from_csharp_value(inner)?;
                    return Ok(Payload::String(payload));
                }
            },
        )
    };

    Ok(quote! {
        impl Payload {
            fn to_csharp_object(&self) -> Result<brec::CSharpValue, brec::Error> {
                let mut obj = brec::csharp_feature::new_object();
                match self {
                    #(#to_wrapped)*
                    #defaults_to
                }
                Ok(brec::CSharpValue::Object(obj))
            }

            fn from_csharp_object(value: brec::CSharpValue) -> Result<Self, brec::Error> {
                let obj: brec::CSharpObjectMap =
                    brec::csharp_feature::from_value_name("object", value)
                        .map_err(|err| brec::Error::CSharp(brec::CSharpError::InvalidAggregatorShape(err.to_string())))?;
                let keys_len = obj.len();
                if keys_len != 1 {
                    return Err(brec::Error::CSharp(brec::CSharpError::InvalidAggregatorShape(
                        format!("expected object with exactly one field, got {}", keys_len),
                    )));
                }
                let (key, inner) = obj.into_iter().next().ok_or_else(|| {
                    brec::Error::CSharp(brec::CSharpError::InvalidAggregatorShape(
                        "expected object key to be a string".to_owned(),
                    ))
                })?;
                match key.as_str() {
                    #(#from_wrapped)*
                    #defaults_from
                    _ => Err(brec::Error::CSharp(brec::CSharpError::InvalidAggregatorShape(
                        format!("unknown payload key: {key}"),
                    ))),
                }
            }

            pub fn decode_csharp(
                bytes: &[u8],
                ctx: &mut crate::PayloadContext<'_>,
            ) -> Result<brec::CSharpValue, brec::Error> {
                let mut cursor = std::io::Cursor::new(bytes);
                let header = <brec::PayloadHeader as brec::ReadFrom>::read(&mut cursor)?;
                let payload = <Payload as brec::ExtractPayloadFrom<Payload>>::read(&mut cursor, &header, ctx)?;
                payload.to_csharp_object()
            }

            pub fn encode_csharp(
                value: brec::CSharpValue,
                out: &mut Vec<u8>,
                ctx: &mut crate::PayloadContext<'_>,
            ) -> Result<(), brec::Error> {
                let mut payload = Payload::from_csharp_object(value)?;
                brec::WriteMutTo::write_all(&mut payload, out, ctx)?;
                Ok(())
            }
        }

        impl brec::CSharpObject for Payload {
            fn to_csharp_object(&self) -> Result<brec::CSharpValue, brec::Error> {
                Payload::to_csharp_object(self)
            }

            fn from_csharp_object(value: brec::CSharpValue) -> Result<Self, brec::Error> {
                Payload::from_csharp_object(value)
            }
        }
    })
}
