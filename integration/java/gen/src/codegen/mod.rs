pub mod base;

use brec_macros_parser::*;
use proc_macro2::TokenStream;
use quote::quote;
use syn::{Data, Fields, Ident, Variant};

fn get_named_field_ident(field: &syn::Field) -> Result<&Ident, E> {
    field
        .ident
        .as_ref()
        .ok_or_else(|| E::NotSupportedBy("Java codegen requires named field identifier".to_owned()))
}

fn get_first_unnamed_ty(fields: &syn::FieldsUnnamed) -> Result<&syn::Type, E> {
    fields
        .unnamed
        .first()
        .map(|f| &f.ty)
        .ok_or_else(|| E::NotSupportedBy("Java codegen expected one unnamed field".to_owned()))
}

fn gen_struct_to_java(fields: &Fields) -> Result<TokenStream, E> {
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
                            let value = <#ty as brec::java_feat::JavaConvert>::to_java_value(&self.#ident, env)?;
                            brec::java_feat::map_put(env, &obj, #name, &value)?;
                        }
                    })
                })
                .collect::<Result<Vec<_>, E>>()?;
            Ok(quote! {
                let obj = brec::java_feat::new_hash_map(env)?;
                #(#setters)*
                Ok(obj)
            })
        }
        Fields::Unnamed(fields) => {
            let len = fields.unnamed.len() as i32;
            let setters = fields
                .unnamed
                .iter()
                .enumerate()
                .map(|(idx, field)| {
                    let idx_lit = idx as i32;
                    let idx_member = syn::Index::from(idx);
                    let ty = &field.ty;
                    Ok(quote! {
                        {
                            let value = <#ty as brec::java_feat::JavaConvert>::to_java_value(&self.#idx_member, env)?;
                            brec::java_feat::list_add(env, &arr, &value)
                                .map_err(|err| brec::java_feat::JavaError::invalid_field_name(#idx_lit.to_string(), err))?;
                        }
                    })
                })
                .collect::<Result<Vec<_>, E>>()?;
            Ok(quote! {
                let arr = brec::java_feat::new_array_list(env, #len)?;
                #(#setters)*
                Ok(arr)
            })
        }
        Fields::Unit => Ok(quote! {
            let obj = brec::java_feat::new_hash_map(env)?;
            Ok(obj)
        }),
    }
}

fn gen_struct_from_java(name: &Ident, fields: &Fields) -> Result<TokenStream, E> {
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
                        let raw = brec::java_feat::map_get(env, &obj, #field_name)
                            .map_err(|err| brec::java_feat::JavaError::invalid_field_name(#field_name, err))?;
                        let #ident: #ty = <#ty as brec::java_feat::JavaConvert>::from_java_value(env, raw)?;
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
                let obj = value;
                #(#getters)*
                Ok(#name { #(#ctor_fields)* })
            })
        }
        Fields::Unnamed(fields) => {
            let len = fields.unnamed.len() as i32;
            let getters = fields
                .unnamed
                .iter()
                .enumerate()
                .map(|(idx, field)| {
                    let idx_lit = idx as i32;
                    let ident = syn::Ident::new(&format!("v{}", idx), proc_macro2::Span::call_site());
                    let ty = &field.ty;
                    Ok(quote! {
                        let raw = brec::java_feat::list_get(env, &arr, #idx_lit)
                            .map_err(|err| brec::java_feat::JavaError::invalid_field_name(#idx_lit.to_string(), err))?;
                        let #ident: #ty = <#ty as brec::java_feat::JavaConvert>::from_java_value(env, raw)?;
                    })
                })
                .collect::<Result<Vec<_>, E>>()?;
            let ctor_fields = (0..fields.unnamed.len())
                .map(|idx| syn::Ident::new(&format!("v{}", idx), proc_macro2::Span::call_site()))
                .collect::<Vec<_>>();
            Ok(quote! {
                let arr = value;
                let arr_len = brec::java_feat::list_size(env, &arr)?;
                if arr_len != #len {
                    return Err(brec::java_feat::JavaError::InvalidAggregatorShape(
                        format!("expected array with {} elements, got {}", #len, arr_len),
                    ));
                }
                #(#getters)*
                Ok(#name(#(#ctor_fields),*))
            })
        }
        Fields::Unit => Ok(quote! {
            let _obj = value;
            Ok(#name)
        }),
    }
}

fn gen_variant_to_java(enum_name: &Ident, variant: &Variant) -> Result<TokenStream, E> {
    let vname = &variant.ident;
    let key = vname.to_string();
    match &variant.fields {
        Fields::Unit => Ok(quote! {
            #enum_name::#vname => {
                brec::java_feat::map_put(env, &obj, #key, &jni::objects::JObject::null())
                    .map_err(|err| brec::java_feat::JavaError::invalid_field_name(#key, err))?;
            }
        }),
        Fields::Unnamed(fields) => {
            if fields.unnamed.len() == 1 {
                Ok(quote! {
                    #enum_name::#vname(inner) => {
                        let value = brec::java_feat::JavaConvert::to_java_value(inner, env)?;
                        brec::java_feat::map_put(env, &obj, #key, &value)
                            .map_err(|err| brec::java_feat::JavaError::invalid_field_name(#key, err))?;
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
                            let value = brec::java_feat::JavaConvert::to_java_value(#ident, env)?;
                            brec::java_feat::list_add(env, &arr, &value)
                                .map_err(|err| brec::java_feat::JavaError::invalid_field_name(#key, err))?;
                        }
                    })
                    .collect::<Vec<_>>();
                let len = bindings.len() as i32;
                Ok(quote! {
                    #enum_name::#vname(#(#bindings),*) => {
                        let arr = brec::java_feat::new_array_list(env, #len)?;
                        #(#set_elems)*
                        brec::java_feat::map_put(env, &obj, #key, &arr)
                            .map_err(|err| brec::java_feat::JavaError::invalid_field_name(#key, err))?;
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
                        let value = brec::java_feat::JavaConvert::to_java_value(#ident, env)?;
                        brec::java_feat::map_put(env, &inner, #fname, &value)
                            .map_err(|err| brec::java_feat::JavaError::invalid_field_name(#key, err))?;
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
                    let inner = brec::java_feat::new_hash_map(env)?;
                    #(#setters)*
                    brec::java_feat::map_put(env, &obj, #key, &inner)
                        .map_err(|err| brec::java_feat::JavaError::invalid_field_name(#key, err))?;
                }
            })
        }
    }
}

fn gen_variant_from_java(variant: &Variant) -> Result<TokenStream, E> {
    let vname = &variant.ident;
    let key = vname.to_string();
    match &variant.fields {
        Fields::Unit => Ok(quote! { #key => Ok(Self::#vname), }),
        Fields::Unnamed(fields) => {
            if fields.unnamed.len() == 1 {
                let ty = get_first_unnamed_ty(fields)?;
                Ok(quote! {
                    #key => {
                        let value: #ty = <#ty as brec::java_feat::JavaConvert>::from_java_value(env, inner)?;
                        Ok(Self::#vname(value))
                    }
                })
            } else {
                let len = fields.unnamed.len() as i32;
                let reads = fields
                    .unnamed
                    .iter()
                    .enumerate()
                    .map(|(idx, field)| {
                        let idx_lit = idx as i32;
                        let ident = syn::Ident::new(&format!("v{}", idx), proc_macro2::Span::call_site());
                        let ty = &field.ty;
                        quote! {
                            let raw = brec::java_feat::list_get(env, &arr, #idx_lit)
                                .map_err(|err| brec::java_feat::JavaError::invalid_field_name(#key, err))?;
                            let #ident: #ty = <#ty as brec::java_feat::JavaConvert>::from_java_value(env, raw)?;
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
                        let arr = inner;
                        let arr_len = brec::java_feat::list_size(env, &arr)?;
                        if arr_len != #len {
                            return Err(brec::java_feat::JavaError::invalid_field_name(
                                #key,
                                format!("expected tuple array with {} elements, got {}", #len, arr_len),
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
                        let raw = brec::java_feat::map_get(env, &obj, #fname)
                            .map_err(|err| brec::java_feat::JavaError::invalid_field_name(#key, err))?;
                        let #ident: #ty = <#ty as brec::java_feat::JavaConvert>::from_java_value(env, raw)?;
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
                    let obj = inner;
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
            let to_java = gen_struct_to_java(&data.fields)?;
            let from_java = gen_struct_from_java(name, &data.fields)?;
            Ok(quote! {
                impl brec::java_feat::JavaConvert for #name {
                    fn to_java_value<'local>(&self, env: &mut jni::JNIEnv<'local>) -> Result<jni::objects::JObject<'local>, brec::java_feat::JavaError> {
                        #to_java
                    }

                    fn from_java_value<'local>(env: &mut jni::JNIEnv<'local>, value: jni::objects::JObject<'local>) -> Result<Self, brec::java_feat::JavaError> {
                        #from_java
                    }
                }
            })
        }
        Data::Enum(data) => {
            let to_arms = data
                .variants
                .iter()
                .map(|variant| gen_variant_to_java(name, variant))
                .collect::<Result<Vec<_>, _>>()?;
            let from_arms = data
                .variants
                .iter()
                .map(gen_variant_from_java)
                .collect::<Result<Vec<_>, _>>()?;
            Ok(quote! {
                impl brec::java_feat::JavaConvert for #name {
                    fn to_java_value<'local>(&self, env: &mut jni::JNIEnv<'local>) -> Result<jni::objects::JObject<'local>, brec::java_feat::JavaError> {
                        let obj = brec::java_feat::new_hash_map(env)?;
                        match self {
                            #(#to_arms)*
                        }
                        Ok(obj)
                    }

                    fn from_java_value<'local>(env: &mut jni::JNIEnv<'local>, value: jni::objects::JObject<'local>) -> Result<Self, brec::java_feat::JavaError> {
                        let obj = value;
                        let (keys_len, key_opt) = brec::java_feat::map_keys_len_and_first(env, &obj)?;
                        if keys_len != 1 {
                            return Err(brec::java_feat::JavaError::InvalidAggregatorShape(
                                format!("expected object with exactly 1 field, got {}", keys_len),
                            ));
                        }
                        let key = key_opt.ok_or_else(|| {
                            brec::java_feat::JavaError::InvalidAggregatorShape(
                                "expected object key to be a string".to_owned(),
                            )
                        })?;
                        let inner = brec::java_feat::map_get(env, &obj, &key)?;
                        match key.as_str() {
                            #(#from_arms)*
                            _ => Err(brec::java_feat::JavaError::InvalidAggregatorShape(
                                format!("unknown enum key: {key}"),
                            )),
                        }
                    }
                }
            })
        }
        Data::Union(_) => Err(E::NotSupportedBy(
            "Java derive does not support union types".to_owned(),
        )),
    }
}
