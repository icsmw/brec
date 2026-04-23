pub mod base;

use brec_macros_parser::*;
use proc_macro2::TokenStream;
use quote::quote;
use syn::{Data, Fields, Ident, Variant};

fn get_named_field_ident(field: &syn::Field) -> Result<&Ident, E> {
    field
        .ident
        .as_ref()
        .ok_or_else(|| E::NotSupportedBy("Napi codegen requires named field identifier".to_owned()))
}

fn get_first_unnamed_ty(fields: &syn::FieldsUnnamed) -> Result<&syn::Type, E> {
    fields
        .unnamed
        .first()
        .map(|f| &f.ty)
        .ok_or_else(|| E::NotSupportedBy("Napi codegen expected one unnamed field".to_owned()))
}

fn gen_struct_to_napi(fields: &Fields) -> Result<TokenStream, E> {
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
                            let value = <#ty as brec::napi_feat::NapiConvert>::to_napi_value(&self.#ident, env)?;
                            obj.set_named_property(#name, value)
                                .map_err(|err| brec::napi_feat::NapiError::invalid_field_name(#name, err))?;
                        }
                    })
                })
                .collect::<Result<Vec<_>, E>>()?;
            Ok(quote! {
                use napi::bindgen_prelude::{JsObjectValue, JsValue, Object};
                let mut obj: Object<'env> = Object::new(env)
                    .map_err(|err| brec::napi_feat::NapiError::InvalidObject(err.to_string()))?;
                #(#setters)*
                Ok(obj.to_unknown())
            })
        }
        Fields::Unnamed(fields) => {
            let len = fields.unnamed.len();
            let setters = fields
                .unnamed
                .iter()
                .enumerate()
                .map(|(idx, field)| {
                    let idx_lit = idx as u32;
                    let idx_member = syn::Index::from(idx);
                    let ty = &field.ty;
                    Ok(quote! {
                        {
                            let value = <#ty as brec::napi_feat::NapiConvert>::to_napi_value(&self.#idx_member, env)?;
                            arr.set(#idx_lit, value)
                                .map_err(|err| brec::napi_feat::NapiError::invalid_field_name(#idx_lit.to_string(), err))?;
                        }
                    })
            })
                .collect::<Result<Vec<_>, E>>()?;
            Ok(quote! {
                use napi::bindgen_prelude::JsValue;
                let mut arr = env.create_array(#len as u32)
                    .map_err(|err| brec::napi_feat::NapiError::InvalidObject(err.to_string()))?;
                #(#setters)*
                Ok(arr.to_unknown())
            })
        }
        Fields::Unit => Ok(quote! {
            use napi::bindgen_prelude::{JsObjectValue, JsValue, Object};
            let obj: Object<'env> = Object::new(env)
                .map_err(|err| brec::napi_feat::NapiError::InvalidObject(err.to_string()))?;
            Ok(obj.to_unknown())
        }),
    }
}

fn gen_struct_from_napi(name: &Ident, fields: &Fields) -> Result<TokenStream, E> {
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
                        let raw: napi::Unknown<'_> = obj.get_named_property(#field_name)
                            .map_err(|err| brec::napi_feat::NapiError::invalid_field_name(#field_name, err))?;
                        let #ident: #ty = <#ty as brec::napi_feat::NapiConvert>::from_napi_value(env, raw)?;
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
                use napi::bindgen_prelude::JsObjectValue;
                let obj: napi::bindgen_prelude::Object<'_> =
                    brec::napi_feat::from_unknown_name("object", value)?;
                #(#getters)*
                Ok(#name { #(#ctor_fields)* })
            })
        }
        Fields::Unnamed(fields) => {
            let len = fields.unnamed.len() as u32;
            let getters = fields
                .unnamed
                .iter()
                .enumerate()
                .map(|(idx, field)| {
                    let idx_lit = idx as u32;
                    let ident = syn::Ident::new(&format!("v{}", idx), proc_macro2::Span::call_site());
                    let ty = &field.ty;
                    Ok(quote! {
                        let raw: napi::Unknown<'_> = arr.get(#idx_lit)
                            .map_err(|err| brec::napi_feat::NapiError::invalid_field_name(#idx_lit.to_string(), err))?
                            .ok_or_else(|| brec::napi_feat::NapiError::invalid_field_name(#idx_lit.to_string(), "missing element"))?;
                        let #ident: #ty = <#ty as brec::napi_feat::NapiConvert>::from_napi_value(env, raw)?;
                    })
                })
                .collect::<Result<Vec<_>, E>>()?;
            let ctor_fields = (0..fields.unnamed.len())
                .map(|idx| syn::Ident::new(&format!("v{}", idx), proc_macro2::Span::call_site()))
                .collect::<Vec<_>>();
            Ok(quote! {
                let arr: napi::bindgen_prelude::Array<'_> =
                    brec::napi_feat::from_unknown_name("array", value)?;
                if arr.len() != #len {
                    return Err(brec::napi_feat::NapiError::InvalidAggregatorShape(
                        format!("expected array with {} elements, got {}", #len, arr.len()),
                    ));
                }
                #(#getters)*
                Ok(#name(#(#ctor_fields),*))
            })
        }
        Fields::Unit => Ok(quote! {
            let _obj: napi::bindgen_prelude::Object<'_> =
                brec::napi_feat::from_unknown_name("object", value)?;
            Ok(#name)
        }),
    }
}

fn gen_variant_to_napi(enum_name: &Ident, variant: &Variant) -> Result<TokenStream, E> {
    let vname = &variant.ident;
    let key = vname.to_string();
    match &variant.fields {
        Fields::Unit => Ok(quote! {
            #enum_name::#vname => {
                obj.set_named_property(#key, Option::<napi::Unknown<'env>>::None)
                    .map_err(|err| brec::napi_feat::NapiError::invalid_field_name(#key, err))?;
            }
        }),
        Fields::Unnamed(fields) => {
            if fields.unnamed.len() == 1 {
                Ok(quote! {
                    #enum_name::#vname(inner) => {
                        let value = brec::napi_feat::NapiConvert::to_napi_value(inner, env)?;
                        obj.set_named_property(#key, value)
                            .map_err(|err| brec::napi_feat::NapiError::invalid_field_name(#key, err))?;
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
                    .enumerate()
                    .map(|(idx, ident)| {
                        let idx_lit = idx as u32;
                        quote! {
                            let value = brec::napi_feat::NapiConvert::to_napi_value(#ident, env)?;
                            arr.set(#idx_lit, value).map_err(|err| {
                                brec::napi_feat::NapiError::invalid_field_name(#key, err)
                            })?;
                        }
                    })
                    .collect::<Vec<_>>();
                let len = bindings.len() as u32;
                Ok(quote! {
                    #enum_name::#vname(#(#bindings),*) => {
                        let mut arr = env.create_array(#len).map_err(|err| {
                            brec::napi_feat::NapiError::invalid_field_name(#key, err)
                        })?;
                        #(#set_elems)*
                        obj.set_named_property(#key, arr).map_err(|err| {
                            brec::napi_feat::NapiError::invalid_field_name(#key, err)
                        })?;
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
                        let value = brec::napi_feat::NapiConvert::to_napi_value(#ident, env)?;
                        inner.set_named_property(#fname, value).map_err(|err| {
                            brec::napi_feat::NapiError::invalid_field_name(#key, err)
                        })?;
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
                    let mut inner = napi::bindgen_prelude::Object::new(env).map_err(|err| {
                        brec::napi_feat::NapiError::invalid_field_name(#key, err)
                    })?;
                    #(#setters)*
                    obj.set_named_property(#key, inner).map_err(|err| {
                        brec::napi_feat::NapiError::invalid_field_name(#key, err)
                    })?;
                }
            })
        }
    }
}

fn gen_variant_from_napi(variant: &Variant) -> Result<TokenStream, E> {
    let vname = &variant.ident;
    let key = vname.to_string();
    match &variant.fields {
        Fields::Unit => Ok(quote! { #key => Ok(Self::#vname), }),
        Fields::Unnamed(fields) => {
            if fields.unnamed.len() == 1 {
                let ty = get_first_unnamed_ty(fields)?;
                Ok(quote! {
                    #key => {
                        let value: #ty = <#ty as brec::napi_feat::NapiConvert>::from_napi_value(env, inner)?;
                        Ok(Self::#vname(value))
                    }
                })
            } else {
                let len = fields.unnamed.len() as u32;
                let reads = fields
                    .unnamed
                    .iter()
                    .enumerate()
                    .map(|(idx, field)| {
                        let idx_lit = idx as u32;
                        let ident = syn::Ident::new(&format!("v{}", idx), proc_macro2::Span::call_site());
                        let ty = &field.ty;
                        quote! {
                            let raw: napi::Unknown<'_> = arr.get(#idx_lit)
                                .map_err(|err| brec::napi_feat::NapiError::invalid_field_name(#key, err))?
                                .ok_or_else(|| brec::napi_feat::NapiError::invalid_field_name(#key, "missing tuple element"))?;
                            let #ident: #ty = <#ty as brec::napi_feat::NapiConvert>::from_napi_value(env, raw)?;
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
                        let arr: napi::bindgen_prelude::Array<'_> =
                            brec::napi_feat::from_unknown_name(#key, inner)?;
                        if arr.len() != #len {
                            return Err(brec::napi_feat::NapiError::invalid_field_name(
                                #key,
                                format!("expected tuple array with {} elements, got {}", #len, arr.len()),
                            ));
                        }
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
                        let raw: napi::Unknown<'_> = obj.get_named_property(#fname)
                            .map_err(|err| brec::napi_feat::NapiError::invalid_field_name(#key, err))?;
                        let #ident: #ty = <#ty as brec::napi_feat::NapiConvert>::from_napi_value(env, raw)?;
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
                    let obj: napi::bindgen_prelude::Object<'_> =
                        brec::napi_feat::from_unknown_name(#key, inner)?;
                    #(#reads)*
                    Ok(Self::#vname { #(#ctor)* })
                }
            })
        }
    }
}

pub fn generate_impl(name: &Ident, data: &Data) -> Result<TokenStream, E> {
    match data {
        Data::Struct(data) => {
            let to_napi = gen_struct_to_napi(&data.fields)?;
            let from_napi = gen_struct_from_napi(name, &data.fields)?;
            Ok(quote! {
                impl brec::napi_feat::NapiConvert for #name {
                    fn to_napi_value<'env>(&self, env: &'env napi::Env) -> Result<napi::Unknown<'env>, brec::napi_feat::NapiError> {
                        #to_napi
                    }

                    fn from_napi_value(env: &napi::Env, value: napi::Unknown<'_>) -> Result<Self, brec::napi_feat::NapiError> {
                        #from_napi
                    }
                }
            })
        }
        Data::Enum(data) => {
            let to_arms = data
                .variants
                .iter()
                .map(|variant| gen_variant_to_napi(name, variant))
                .collect::<Result<Vec<_>, _>>()?;
            let from_arms = data
                .variants
                .iter()
                .map(gen_variant_from_napi)
                .collect::<Result<Vec<_>, _>>()?;
            Ok(quote! {
                impl brec::napi_feat::NapiConvert for #name {
                    fn to_napi_value<'env>(&self, env: &'env napi::Env) -> Result<napi::Unknown<'env>, brec::napi_feat::NapiError> {
                        use napi::bindgen_prelude::{JsObjectValue, JsValue, Object};
                        let mut obj: Object<'env> = Object::new(env)
                            .map_err(|err| brec::napi_feat::NapiError::InvalidObject(err.to_string()))?;
                        match self {
                            #(#to_arms)*
                        }
                        Ok(obj.to_unknown())
                    }

                    fn from_napi_value(env: &napi::Env, value: napi::Unknown<'_>) -> Result<Self, brec::napi_feat::NapiError> {
                        use napi::bindgen_prelude::JsObjectValue;
                        let obj: napi::bindgen_prelude::Object<'_> =
                            brec::napi_feat::from_unknown_name("object", value)?;
                        let keys = obj.get_property_names()
                            .map_err(|err| brec::napi_feat::NapiError::InvalidObject(err.to_string()))?;
                        let keys_len = keys.get_array_length()
                            .map_err(|err| brec::napi_feat::NapiError::InvalidObject(err.to_string()))?;
                        if keys_len != 1 {
                            return Err(brec::napi_feat::NapiError::InvalidAggregatorShape(
                                format!("expected object with exactly 1 field, got {}", keys_len),
                            ));
                        }
                        let key: String = keys.get_element(0)
                            .map_err(|err| brec::napi_feat::NapiError::InvalidObject(err.to_string()))?
                            ;
                        let inner: napi::Unknown<'_> = obj.get_named_property(key.as_str())
                            .map_err(|err| brec::napi_feat::NapiError::invalid_field_name(key.clone(), err))?;
                        match key.as_str() {
                            #(#from_arms)*
                            _ => Err(brec::napi_feat::NapiError::InvalidAggregatorShape(
                                format!("unknown enum variant key {}", key),
                            )),
                        }
                    }
                }
            })
        }
        Data::Union(_) => Err(E::NotSupportedBy("Napi for union".to_owned())),
    }
}
