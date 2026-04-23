use brec_macros_parser::*;

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
                let value = brec::csharp_feat::CSharpObject::to_csharp_object(block)?;
                brec::csharp_feat::map_put(&mut obj, #key, value).map_err(|err| {
                    brec::csharp_feat::CSharpError::InvalidAggregatorShape(err.to_string())
                })?;
            }
        });
        from_wrapped.push(quote! {
            #key => {
                let block = <#fullpath as brec::csharp_feat::CSharpObject>::from_csharp_object(inner)?;
                return Ok(Block::#fullname(block));
            }
        });
    }

    Ok(quote! {
        impl Block {
            fn to_csharp_object(&self) -> Result<brec::csharp_feat::CSharpValue, brec::csharp_feat::CSharpError> {
                let mut obj = brec::csharp_feat::new_object();
                match self {
                    #(#to_wrapped)*
                }
                Ok(brec::csharp_feat::CSharpValue::Object(obj))
            }

            fn from_csharp_object(value: brec::csharp_feat::CSharpValue) -> Result<Self, brec::csharp_feat::CSharpError> {
                let obj: brec::csharp_feat::CSharpObjectMap =
                    brec::csharp_feat::from_value_name("object", value)
                        .map_err(|err| brec::csharp_feat::CSharpError::InvalidAggregatorShape(err.to_string()))?;
                let keys_len = obj.len();
                if keys_len != 1 {
                    return Err(brec::csharp_feat::CSharpError::InvalidAggregatorShape(
                        format!("expected object with exactly one field, got {}", keys_len),
                    ));
                }
                let (key, inner) = obj.into_iter().next().ok_or_else(|| {
                    brec::csharp_feat::CSharpError::InvalidAggregatorShape(
                        "expected object key to be a string".to_owned(),
                    )
                })?;
                match key.as_str() {
                    #(#from_wrapped)*
                    _ => Err(brec::csharp_feat::CSharpError::InvalidAggregatorShape(
                        format!("unknown block key: {key}"),
                    )),
                }
            }

            pub fn decode_csharp(bytes: &[u8]) -> Result<brec::csharp_feat::CSharpValue, brec::Error> {
                let mut src = bytes;
                let block = <Block as brec::ReadBlockFrom>::read(&mut src, false)?;
                Ok(block.to_csharp_object()?)
            }

            pub fn encode_csharp(value: brec::csharp_feat::CSharpValue, out: &mut Vec<u8>) -> Result<(), brec::Error> {
                let block = Block::from_csharp_object(value)?;
                brec::WriteTo::write_all(&block, out)?;
                Ok(())
            }
        }

        impl brec::csharp_feat::CSharpObject for Block {
            fn to_csharp_object(&self) -> Result<brec::csharp_feat::CSharpValue, brec::csharp_feat::CSharpError> {
                Block::to_csharp_object(self)
            }

            fn from_csharp_object(value: brec::csharp_feat::CSharpValue) -> Result<Self, brec::csharp_feat::CSharpError> {
                Block::from_csharp_object(value)
            }
        }
    })
}
