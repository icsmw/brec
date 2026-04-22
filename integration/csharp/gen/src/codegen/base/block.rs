use brec_macros_parser::*;
use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use syn::{Ident, LitStr};

fn to_csharp_field_set(field: &Field) -> Result<TokenStream, E> {
    let rust_field = format_ident!("{}", field.name);
    let field_name = LitStr::new(&field.name, proc_macro2::Span::call_site());
    let value = match &field.ty {
        Ty::LinkedToU8(_) => quote! {{
            let value: u8 = (&self.#rust_field).into();
            <u8 as brec::CSharpConvert>::to_csharp_value(&value)?
        }},
        _ => {
            let ty = field.ty.direct();
            quote! { <#ty as brec::CSharpConvert>::to_csharp_value(&self.#rust_field)? }
        }
    };
    Ok(quote! {
        let field_value = #value;
        brec::csharp_feature::map_put(&mut obj, #field_name, field_value)
            .map_err(|err| brec::CSharpError::invalid_field_name(#field_name, err))?;
    })
}

fn from_csharp_field_get(field: &Field) -> Result<TokenStream, E> {
    let rust_field = format_ident!("{}", field.name);
    let field_name = LitStr::new(&field.name, proc_macro2::Span::call_site());
    let ty = field.ty.direct();
    Ok(match &field.ty {
        Ty::LinkedToU8(enum_name) => quote! {
            let raw = brec::csharp_feature::map_take(&mut obj, #field_name)
                .map_err(|err| brec::CSharpError::invalid_field_name(#field_name, err))?;
            let raw: u8 = <u8 as brec::CSharpConvert>::from_csharp_value(raw)?;
            let #rust_field = #ty::try_from(raw)
                .map_err(|err| brec::Error::FailedConverting(#enum_name.to_owned(), err))?;
        },
        _ => quote! {
            let raw = brec::csharp_feature::map_take(&mut obj, #field_name)
                .map_err(|err| brec::CSharpError::invalid_field_name(#field_name, err))?;
            let #rust_field: #ty = <#ty as brec::CSharpConvert>::from_csharp_value(raw)?;
        },
    })
}

pub fn generate(block_name: &Ident, fields: &[Field]) -> Result<TokenStream, E> {
    let to_csharp = fields
        .iter()
        .filter(|field| !field.injected)
        .map(to_csharp_field_set)
        .collect::<Result<Vec<_>, _>>()?;
    let from_csharp = fields
        .iter()
        .filter(|field| !field.injected)
        .map(from_csharp_field_get)
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
            fn to_csharp_object(&self) -> Result<brec::CSharpValue, brec::Error> {
                let mut obj = brec::csharp_feature::new_object();
                #(#to_csharp)*
                Ok(brec::CSharpValue::Object(obj))
            }

            fn from_csharp_object(value: brec::CSharpValue) -> Result<Self, brec::Error> {
                let mut obj: brec::csharp_feature::CSharpObjectMap =
                    brec::csharp_feature::from_value_name("object", value)
                        .map_err(|err| brec::Error::CSharp(brec::CSharpError::InvalidObject(err.to_string())))?;
                #(#from_csharp)*
                Ok(Self {
                    #(#ctor_fields)*
                })
            }

            pub fn decode_csharp(bytes: &[u8]) -> Result<brec::CSharpValue, brec::Error> {
                let mut src = bytes;
                let block = <#block_name as brec::ReadBlockFrom>::read(&mut src, false)?;
                block.to_csharp_object()
            }

            pub fn encode_csharp(value: brec::CSharpValue, out: &mut Vec<u8>) -> Result<(), brec::Error> {
                let block = #block_name::from_csharp_object(value)?;
                brec::WriteTo::write_all(&block, out)?;
                Ok(())
            }
        }

        impl brec::CSharpObject for #block_name {
            fn to_csharp_object(&self) -> Result<brec::CSharpValue, brec::Error> {
                #block_name::to_csharp_object(self)
            }

            fn from_csharp_object(value: brec::CSharpValue) -> Result<Self, brec::Error> {
                #block_name::from_csharp_object(value)
            }
        }
    })
}
