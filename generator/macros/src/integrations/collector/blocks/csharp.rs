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
                let value = brec::CSharpObject::to_csharp_object(block)?;
                brec::csharp_feature::map_put(&mut obj, #key, value).map_err(|err| {
                    brec::Error::CSharp(brec::CSharpError::InvalidAggregatorShape(err.to_string()))
                })?;
            }
        });
        from_wrapped.push(quote! {
            #key => {
                let block = <#fullpath as brec::CSharpObject>::from_csharp_object(inner)?;
                return Ok(Block::#fullname(block));
            }
        });
    }

    Ok(quote! {
        impl Block {
            fn to_csharp_object(&self) -> Result<brec::CSharpValue, brec::Error> {
                let mut obj = brec::csharp_feature::new_object();
                match self {
                    #(#to_wrapped)*
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
                    _ => Err(brec::Error::CSharp(brec::CSharpError::InvalidAggregatorShape(
                        format!("unknown block key: {key}"),
                    ))),
                }
            }

            pub fn decode_csharp(bytes: &[u8]) -> Result<brec::CSharpValue, brec::Error> {
                let mut src = bytes;
                let block = <Block as brec::ReadBlockFrom>::read(&mut src, false)?;
                block.to_csharp_object()
            }

            pub fn encode_csharp(value: brec::CSharpValue, out: &mut Vec<u8>) -> Result<(), brec::Error> {
                let block = Block::from_csharp_object(value)?;
                brec::WriteTo::write_all(&block, out)?;
                Ok(())
            }
        }

        impl brec::CSharpObject for Block {
            fn to_csharp_object(&self) -> Result<brec::CSharpValue, brec::Error> {
                Block::to_csharp_object(self)
            }

            fn from_csharp_object(value: brec::CSharpValue) -> Result<Self, brec::Error> {
                Block::from_csharp_object(value)
            }
        }
    })
}
