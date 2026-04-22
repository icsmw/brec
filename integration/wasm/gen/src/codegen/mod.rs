pub mod base;

use brec_macros_parser::*;
use proc_macro2::TokenStream;
use quote::quote;
use syn::{Data, Fields, Ident, Variant};

fn get_named_field_ident(field: &syn::Field) -> Result<&Ident, E> {
    field
        .ident
        .as_ref()
        .ok_or_else(|| E::NotSupportedBy("Wasm codegen requires named field identifier".to_owned()))
}

fn get_first_unnamed_ty(fields: &syn::FieldsUnnamed) -> Result<&syn::Type, E> {
    fields
        .unnamed
        .first()
        .map(|f| &f.ty)
        .ok_or_else(|| E::NotSupportedBy("Wasm codegen expected one unnamed field".to_owned()))
}

fn gen_struct_to_wasm(fields: &Fields) -> Result<TokenStream, E> {
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
                            let value = <#ty as brec::WasmConvert>::to_wasm_value(&self.#ident)?;
                            js_sys::Reflect::set(&obj, &wasm_bindgen::JsValue::from_str(#name), &value)
                                .map_err(|err| brec::WasmError::invalid_field_name(#name, format!("{err:?}")))?;
                        }
                    })
                })
                .collect::<Result<Vec<_>, E>>()?;
            Ok(quote! {
                let obj = js_sys::Object::new();
                #(#setters)*
                Ok(obj.into())
            })
        }
        Fields::Unnamed(fields) => {
            let len = fields.unnamed.len() as u32;
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
                            let value = <#ty as brec::WasmConvert>::to_wasm_value(&self.#idx_member)?;
                            arr.set(#idx_lit, value);
                        }
                    })
                })
                .collect::<Result<Vec<_>, E>>()?;
            Ok(quote! {
                let arr = js_sys::Array::new_with_length(#len);
                #(#setters)*
                Ok(arr.into())
            })
        }
        Fields::Unit => Ok(quote! {
            Ok(js_sys::Object::new().into())
        }),
    }
}

fn gen_struct_from_wasm(name: &Ident, fields: &Fields) -> Result<TokenStream, E> {
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
                        let raw = js_sys::Reflect::get(&obj, &wasm_bindgen::JsValue::from_str(#field_name))
                            .map_err(|err| brec::WasmError::invalid_field_name(#field_name, format!("{err:?}")))?;
                        let #ident: #ty = <#ty as brec::WasmConvert>::from_wasm_value(raw)?;
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
                let obj: js_sys::Object = brec::wasm_feature::from_value_name("object", value)?;
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
                    let ident =
                        syn::Ident::new(&format!("v{}", idx), proc_macro2::Span::call_site());
                    let ty = &field.ty;
                    Ok(quote! {
                        let raw = arr.get(#idx_lit);
                        let #ident: #ty = <#ty as brec::WasmConvert>::from_wasm_value(raw)?;
                    })
                })
                .collect::<Result<Vec<_>, E>>()?;
            let ctor_fields = (0..fields.unnamed.len())
                .map(|idx| syn::Ident::new(&format!("v{}", idx), proc_macro2::Span::call_site()))
                .collect::<Vec<_>>();
            Ok(quote! {
                if !js_sys::Array::is_array(&value) {
                    return Err(brec::Error::Wasm(brec::WasmError::InvalidAggregatorShape(
                        "expected tuple as array".to_owned(),
                    )));
                }
                let arr = js_sys::Array::from(&value);
                if arr.length() != #len {
                    return Err(brec::Error::Wasm(brec::WasmError::InvalidAggregatorShape(
                        format!("expected array with {} elements, got {}", #len, arr.length()),
                    )));
                }
                #(#getters)*
                Ok(#name(#(#ctor_fields),*))
            })
        }
        Fields::Unit => Ok(quote! {
            let _obj: js_sys::Object = brec::wasm_feature::from_value_name("object", value)?;
            Ok(#name)
        }),
    }
}

fn gen_variant_to_wasm(enum_name: &Ident, variant: &Variant) -> Result<TokenStream, E> {
    let vname = &variant.ident;
    let key = vname.to_string();
    match &variant.fields {
        Fields::Unit => Ok(quote! {
            #enum_name::#vname => {
                js_sys::Reflect::set(&obj, &wasm_bindgen::JsValue::from_str(#key), &wasm_bindgen::JsValue::NULL)
                    .map_err(|err| brec::WasmError::invalid_field_name(#key, format!("{err:?}")))?;
            }
        }),
        Fields::Unnamed(fields) => {
            if fields.unnamed.len() == 1 {
                Ok(quote! {
                    #enum_name::#vname(inner) => {
                        let value = brec::WasmConvert::to_wasm_value(inner)?;
                        js_sys::Reflect::set(&obj, &wasm_bindgen::JsValue::from_str(#key), &value)
                            .map_err(|err| brec::WasmError::invalid_field_name(#key, format!("{err:?}")))?;
                    }
                })
            } else {
                let len = fields.unnamed.len() as u32;
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
                            let value = brec::WasmConvert::to_wasm_value(#ident)?;
                            arr.set(#idx_lit, value);
                        }
                    })
                    .collect::<Vec<_>>();
                Ok(quote! {
                    #enum_name::#vname(#(#bindings),*) => {
                        let arr = js_sys::Array::new_with_length(#len);
                        #(#set_elems)*
                        js_sys::Reflect::set(&obj, &wasm_bindgen::JsValue::from_str(#key), &arr)
                            .map_err(|err| brec::WasmError::invalid_field_name(#key, format!("{err:?}")))?;
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
                        let value = brec::WasmConvert::to_wasm_value(#ident)?;
                        js_sys::Reflect::set(&inner, &wasm_bindgen::JsValue::from_str(#fname), &value)
                            .map_err(|err| brec::WasmError::invalid_field_name(#key, format!("{err:?}")))?;
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
                    let inner = js_sys::Object::new();
                    #(#setters)*
                    js_sys::Reflect::set(&obj, &wasm_bindgen::JsValue::from_str(#key), &inner)
                        .map_err(|err| brec::WasmError::invalid_field_name(#key, format!("{err:?}")))?;
                }
            })
        }
    }
}

fn gen_variant_from_wasm(variant: &Variant) -> Result<TokenStream, E> {
    let vname = &variant.ident;
    let key = vname.to_string();
    match &variant.fields {
        Fields::Unit => Ok(quote! { #key => Ok(Self::#vname), }),
        Fields::Unnamed(fields) => {
            if fields.unnamed.len() == 1 {
                let ty = get_first_unnamed_ty(fields)?;
                Ok(quote! {
                    #key => {
                        let value: #ty = <#ty as brec::WasmConvert>::from_wasm_value(inner)?;
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
                        let ident =
                            syn::Ident::new(&format!("v{}", idx), proc_macro2::Span::call_site());
                        let ty = &field.ty;
                        quote! {
                            let raw = arr.get(#idx_lit);
                            let #ident: #ty = <#ty as brec::WasmConvert>::from_wasm_value(raw)?;
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
                        if !js_sys::Array::is_array(&inner) {
                            return Err(brec::WasmError::invalid_field_name(#key, "expected tuple array"));
                        }
                        let arr = js_sys::Array::from(&inner);
                        if arr.length() != #len {
                            return Err(brec::WasmError::invalid_field_name(
                                #key,
                                format!("expected tuple array with {} elements, got {}", #len, arr.length()),
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
                        let raw = js_sys::Reflect::get(&obj, &wasm_bindgen::JsValue::from_str(#fname))
                            .map_err(|err| brec::WasmError::invalid_field_name(#key, format!("{err:?}")))?;
                        let #ident: #ty = <#ty as brec::WasmConvert>::from_wasm_value(raw)?;
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
                    let obj: js_sys::Object = brec::wasm_feature::from_value_name(#key, inner)?;
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
            let to_wasm = gen_struct_to_wasm(&data.fields)?;
            let from_wasm = gen_struct_from_wasm(name, &data.fields)?;
            Ok(quote! {
                impl brec::WasmConvert for #name {
                    fn to_wasm_value(&self) -> Result<wasm_bindgen::JsValue, brec::Error> {
                        #to_wasm
                    }

                    fn from_wasm_value(value: wasm_bindgen::JsValue) -> Result<Self, brec::Error> {
                        #from_wasm
                    }
                }
            })
        }
        Data::Enum(data) => {
            let to_arms = data
                .variants
                .iter()
                .map(|variant| gen_variant_to_wasm(name, variant))
                .collect::<Result<Vec<_>, _>>()?;
            let from_arms = data
                .variants
                .iter()
                .map(gen_variant_from_wasm)
                .collect::<Result<Vec<_>, _>>()?;
            Ok(quote! {
                impl brec::WasmConvert for #name {
                    fn to_wasm_value(&self) -> Result<wasm_bindgen::JsValue, brec::Error> {
                        let obj = js_sys::Object::new();
                        match self {
                            #(#to_arms)*
                        }
                        Ok(obj.into())
                    }

                    fn from_wasm_value(value: wasm_bindgen::JsValue) -> Result<Self, brec::Error> {
                        let obj: js_sys::Object = brec::wasm_feature::from_value_name("object", value)?;
                        let keys = js_sys::Object::keys(&obj);
                        let keys_len = keys.length();
                        if keys_len != 1 {
                            return Err(brec::Error::Wasm(brec::WasmError::InvalidAggregatorShape(
                                format!("expected object with exactly 1 field, got {}", keys_len),
                            )));
                        }
                        let key = keys
                            .get(0)
                            .as_string()
                            .ok_or_else(|| {
                                brec::Error::Wasm(brec::WasmError::InvalidObject(
                                    "enum key must be a string".to_owned(),
                                ))
                            })?;
                        let inner = js_sys::Reflect::get(&obj, &wasm_bindgen::JsValue::from_str(key.as_str()))
                            .map_err(|err| brec::WasmError::invalid_field_name(key.clone(), format!("{err:?}")))?;
                        match key.as_str() {
                            #(#from_arms)*
                            _ => Err(brec::Error::Wasm(brec::WasmError::InvalidAggregatorShape(
                                format!("unknown enum variant key {}", key),
                            ))),
                        }
                    }
                }
            })
        }
        Data::Union(_) => Err(E::NotSupportedBy("Wasm for union".to_owned())),
    }
}
