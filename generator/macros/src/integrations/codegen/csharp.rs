use crate::*;
use proc_macro2::TokenStream;
use quote::quote;
use syn::{Data, Fields, Ident, Variant};

fn get_named_field_ident(field: &syn::Field) -> Result<&Ident, E> {
    field.ident.as_ref().ok_or_else(|| {
        E::NotSupportedBy("CSharp codegen requires named field identifier".to_owned())
    })
}

fn get_first_unnamed_ty(fields: &syn::FieldsUnnamed) -> Result<&syn::Type, E> {
    fields
        .unnamed
        .first()
        .map(|f| &f.ty)
        .ok_or_else(|| E::NotSupportedBy("CSharp codegen expected one unnamed field".to_owned()))
}

fn gen_struct_to_csharp(fields: &Fields) -> Result<TokenStream, E> {
    match fields {
        Fields::Named(fields) => {
            let setters = fields
                .named
                .iter()
                .map(|field| {
                    let ident = get_named_field_ident(field)?;
                    let name = ident.to_string();
                    let ty = &field.ty;
                    Ok(quote! {
                        {
                            let value = <#ty as brec::CSharpConvert>::to_csharp_value(&self.#ident)?;
                            brec::csharp_feature::map_put(&mut obj, #name, value)?;
                        }
                    })
                })
                .collect::<Result<Vec<_>, E>>()?;
            Ok(quote! {
                let mut obj = brec::csharp_feature::new_object();
                #(#setters)*
                Ok(brec::CSharpValue::Object(obj))
            })
        }
        Fields::Unnamed(fields) => {
            let setters = fields
                .unnamed
                .iter()
                .enumerate()
                .map(|(idx, field)| {
                    let idx_member = syn::Index::from(idx);
                    let ty = &field.ty;
                    Ok(quote! {
                        {
                            let value = <#ty as brec::CSharpConvert>::to_csharp_value(&self.#idx_member)?;
                            brec::csharp_feature::list_add(&mut arr, value)
                                .map_err(|err| brec::CSharpError::invalid_field_name(#idx.to_string(), err))?;
                        }
                    })
                })
                .collect::<Result<Vec<_>, E>>()?;
            let len = fields.unnamed.len();
            Ok(quote! {
                let mut arr = brec::csharp_feature::new_array(#len);
                #(#setters)*
                Ok(brec::CSharpValue::Array(arr))
            })
        }
        Fields::Unit => Ok(quote! {
            let obj = brec::csharp_feature::new_object();
            Ok(brec::CSharpValue::Object(obj))
        }),
    }
}

fn gen_struct_from_csharp(name: &Ident, fields: &Fields) -> Result<TokenStream, E> {
    match fields {
        Fields::Named(fields) => {
            let getters = fields
                .named
                .iter()
                .map(|field| {
                    let ident = get_named_field_ident(field)?;
                    let field_name = ident.to_string();
                    let ty = &field.ty;
                    Ok(quote! {
                        let raw = brec::csharp_feature::map_take(&mut obj, #field_name)
                            .map_err(|err| brec::CSharpError::invalid_field_name(#field_name, err))?;
                        let #ident: #ty = <#ty as brec::CSharpConvert>::from_csharp_value(raw)?;
                    })
                })
                .collect::<Result<Vec<_>, E>>()?;
            let ctor_fields = fields
                .named
                .iter()
                .map(|field| {
                    let ident = get_named_field_ident(field)?;
                    Ok(quote! { #ident, })
                })
                .collect::<Result<Vec<_>, E>>()?;
            Ok(quote! {
                let mut obj: brec::CSharpObjectMap = brec::csharp_feature::from_value_name("object", value)?;
                #(#getters)*
                Ok(#name { #(#ctor_fields)* })
            })
        }
        Fields::Unnamed(fields) => {
            let len = fields.unnamed.len();
            let getters = fields
                .unnamed
                .iter()
                .enumerate()
                .map(|(idx, field)| {
                    let ident =
                        syn::Ident::new(&format!("v{}", idx), proc_macro2::Span::call_site());
                    let ty = &field.ty;
                    Ok(quote! {
                        let raw = arr_iter.next().ok_or_else(|| {
                            brec::CSharpError::invalid_field_name(
                                #idx.to_string(),
                                format!("missing element at index {}", #idx),
                            )
                        })?;
                        let #ident: #ty = <#ty as brec::CSharpConvert>::from_csharp_value(raw)?;
                    })
                })
                .collect::<Result<Vec<_>, E>>()?;
            let ctor_fields = (0..fields.unnamed.len())
                .map(|idx| syn::Ident::new(&format!("v{}", idx), proc_macro2::Span::call_site()))
                .collect::<Vec<_>>();
            Ok(quote! {
                let arr: Vec<brec::CSharpValue> = brec::csharp_feature::from_value_name("array", value)?;
                if arr.len() != #len {
                    return Err(brec::Error::CSharp(brec::CSharpError::InvalidAggregatorShape(
                        format!("expected array with {} elements, got {}", #len, arr.len()),
                    )));
                }
                let mut arr_iter = arr.into_iter();
                #(#getters)*
                Ok(#name(#(#ctor_fields),*))
            })
        }
        Fields::Unit => Ok(quote! {
            let _obj: brec::CSharpObjectMap = brec::csharp_feature::from_value_name("object", value)?;
            Ok(#name)
        }),
    }
}

fn gen_variant_to_csharp(enum_name: &Ident, variant: &Variant) -> Result<TokenStream, E> {
    let vname = &variant.ident;
    let key = vname.to_string();
    match &variant.fields {
        Fields::Unit => Ok(quote! {
            #enum_name::#vname => {
                brec::csharp_feature::map_put(&mut obj, #key, brec::CSharpValue::Null)
                    .map_err(|err| brec::CSharpError::invalid_field_name(#key, err))?;
            }
        }),
        Fields::Unnamed(fields) => {
            if fields.unnamed.len() == 1 {
                Ok(quote! {
                    #enum_name::#vname(inner) => {
                        let value = brec::CSharpConvert::to_csharp_value(inner)?;
                        brec::csharp_feature::map_put(&mut obj, #key, value)
                            .map_err(|err| brec::CSharpError::invalid_field_name(#key, err))?;
                    }
                })
            } else {
                let bindings = (0..fields.unnamed.len())
                    .map(|idx| {
                        syn::Ident::new(&format!("v{}", idx), proc_macro2::Span::call_site())
                    })
                    .collect::<Vec<_>>();
                let set_elems = bindings
                    .iter()
                    .map(|ident| {
                        quote! {
                            let value = brec::CSharpConvert::to_csharp_value(#ident)?;
                            brec::csharp_feature::list_add(&mut arr, value)
                                .map_err(|err| brec::CSharpError::invalid_field_name(#key, err))?;
                        }
                    })
                    .collect::<Vec<_>>();
                let len = bindings.len();
                Ok(quote! {
                    #enum_name::#vname(#(#bindings),*) => {
                        let mut arr = brec::csharp_feature::new_array(#len);
                        #(#set_elems)*
                        brec::csharp_feature::map_put(&mut obj, #key, brec::CSharpValue::Array(arr))
                            .map_err(|err| brec::CSharpError::invalid_field_name(#key, err))?;
                    }
                })
            }
        }
        Fields::Named(fields) => {
            let setters = fields
                .named
                .iter()
                .map(|field| {
                    let ident = get_named_field_ident(field)?;
                    let fname = ident.to_string();
                    Ok(quote! {
                        let value = brec::CSharpConvert::to_csharp_value(#ident)?;
                        brec::csharp_feature::map_put(&mut inner, #fname, value)
                            .map_err(|err| brec::CSharpError::invalid_field_name(#key, err))?;
                    })
                })
                .collect::<Result<Vec<_>, E>>()?;
            let bindings = fields
                .named
                .iter()
                .map(get_named_field_ident)
                .collect::<Result<Vec<_>, E>>()?;
            Ok(quote! {
                #enum_name::#vname { #(#bindings),* } => {
                    let mut inner = brec::csharp_feature::new_object();
                    #(#setters)*
                    brec::csharp_feature::map_put(&mut obj, #key, brec::CSharpValue::Object(inner))
                        .map_err(|err| brec::CSharpError::invalid_field_name(#key, err))?;
                }
            })
        }
    }
}

fn gen_variant_from_csharp(variant: &Variant) -> Result<TokenStream, E> {
    let vname = &variant.ident;
    let key = vname.to_string();
    match &variant.fields {
        Fields::Unit => Ok(quote! { #key => Ok(Self::#vname), }),
        Fields::Unnamed(fields) => {
            if fields.unnamed.len() == 1 {
                let ty = get_first_unnamed_ty(fields)?;
                Ok(quote! {
                    #key => {
                        let value: #ty = <#ty as brec::CSharpConvert>::from_csharp_value(inner)?;
                        Ok(Self::#vname(value))
                    }
                })
            } else {
                let len = fields.unnamed.len();
                let reads = fields
                    .unnamed
                    .iter()
                    .enumerate()
                    .map(|(idx, field)| {
                        let ident =
                            syn::Ident::new(&format!("v{}", idx), proc_macro2::Span::call_site());
                        let ty = &field.ty;
                        quote! {
                            let raw = arr_iter.next().ok_or_else(|| {
                                brec::CSharpError::invalid_field_name(
                                    #key,
                                    format!("missing tuple element at index {}", #idx),
                                )
                            })?;
                            let #ident: #ty = <#ty as brec::CSharpConvert>::from_csharp_value(raw)?;
                        }
                    })
                    .collect::<Vec<_>>();
                let vals = (0..fields.unnamed.len())
                    .map(|idx| {
                        syn::Ident::new(&format!("v{}", idx), proc_macro2::Span::call_site())
                    })
                    .collect::<Vec<_>>();
                Ok(quote! {
                    #key => {
                        let arr: Vec<brec::CSharpValue> = brec::csharp_feature::from_value_name(#key, inner)?;
                        if arr.len() != #len {
                            return Err(brec::CSharpError::invalid_field_name(
                                #key,
                                format!("expected tuple array with {} elements, got {}", #len, arr.len()),
                            ));
                        }
                        let mut arr_iter = arr.into_iter();
                        #(#reads)*
                        Ok(Self::#vname(#(#vals),*))
                    }
                })
            }
        }
        Fields::Named(fields) => {
            let reads = fields
                .named
                .iter()
                .map(|field| {
                    let ident = get_named_field_ident(field)?;
                    let fname = ident.to_string();
                    let ty = &field.ty;
                    Ok(quote! {
                        let raw = brec::csharp_feature::map_take(&mut obj, #fname)
                            .map_err(|err| brec::CSharpError::invalid_field_name(#key, err))?;
                        let #ident: #ty = <#ty as brec::CSharpConvert>::from_csharp_value(raw)?;
                    })
                })
                .collect::<Result<Vec<_>, E>>()?;
            let ctor = fields
                .named
                .iter()
                .map(|field| {
                    let ident = get_named_field_ident(field)?;
                    Ok(quote! { #ident, })
                })
                .collect::<Result<Vec<_>, E>>()?;
            Ok(quote! {
                #key => {
                    let mut obj: brec::CSharpObjectMap = brec::csharp_feature::from_value_name(#key, inner)?;
                    #(#reads)*
                    Ok(Self::#vname { #(#ctor)* })
                }
            })
        }
    }
}

pub(crate) fn generate_impl(name: &Ident, data: &Data) -> Result<TokenStream, E> {
    match data {
        Data::Struct(data) => {
            let to_csharp = gen_struct_to_csharp(&data.fields)?;
            let from_csharp = gen_struct_from_csharp(name, &data.fields)?;
            Ok(quote! {
                impl brec::CSharpConvert for #name {
                    fn to_csharp_value(&self) -> Result<brec::CSharpValue, brec::Error> {
                        #to_csharp
                    }

                    fn from_csharp_value(value: brec::CSharpValue) -> Result<Self, brec::Error> {
                        #from_csharp
                    }
                }
            })
        }
        Data::Enum(data) => {
            let to_arms = data
                .variants
                .iter()
                .map(|variant| gen_variant_to_csharp(name, variant))
                .collect::<Result<Vec<_>, _>>()?;
            let from_arms = data
                .variants
                .iter()
                .map(gen_variant_from_csharp)
                .collect::<Result<Vec<_>, _>>()?;
            Ok(quote! {
                impl brec::CSharpConvert for #name {
                    fn to_csharp_value(&self) -> Result<brec::CSharpValue, brec::Error> {
                        let mut obj = brec::csharp_feature::new_object();
                        match self {
                            #(#to_arms)*
                        }
                        Ok(brec::CSharpValue::Object(obj))
                    }

                    fn from_csharp_value(value: brec::CSharpValue) -> Result<Self, brec::Error> {
                        let obj: brec::CSharpObjectMap = brec::csharp_feature::from_value_name("object", value)?;
                        let keys_len = obj.len();
                        if keys_len != 1 {
                            return Err(brec::Error::CSharp(brec::CSharpError::InvalidAggregatorShape(
                                format!("expected object with exactly 1 field, got {}", keys_len),
                            )));
                        }
                        let (key, inner) = obj.into_iter().next().ok_or_else(|| {
                            brec::Error::CSharp(brec::CSharpError::InvalidAggregatorShape(
                                "expected object key to be a string".to_owned(),
                            ))
                        })?;
                        match key.as_str() {
                            #(#from_arms)*
                            _ => Err(brec::Error::CSharp(brec::CSharpError::InvalidAggregatorShape(
                                format!("unknown enum variant key {}", key),
                            ))),
                        }
                    }
                }
            })
        }
        Data::Union(_) => Err(E::NotSupportedBy(
            "CSharp derive does not support union types".to_owned(),
        )),
    }
}
