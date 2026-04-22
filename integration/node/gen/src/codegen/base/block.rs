use brec_gen_tys::*;
use proc_macro2::{Ident, TokenStream};
use quote::{format_ident, quote};
use syn::LitStr;

fn to_napi_field_set(field: &Field) -> Result<TokenStream, E> {
    let rust_field = format_ident!("{}", field.name);
    let js_field = LitStr::new(&field.name, proc_macro2::Span::call_site());
    let value = match &field.ty {
        Ty::F32 => quote! { self.#rust_field.to_bits() },
        Ty::F64 => quote! { napi::bindgen_prelude::BigInt::from(self.#rust_field.to_bits()) },
        Ty::I64 | Ty::U64 | Ty::I128 | Ty::U128 => {
            quote! { napi::bindgen_prelude::BigInt::from(self.#rust_field) }
        }
        Ty::LinkedToU8(_) => quote! {{
            let value: u8 = (&self.#rust_field).into();
            value
        }},
        _ => quote! { self.#rust_field },
    };
    Ok(quote! {
        obj.set_named_property(#js_field, #value)
            .map_err(|err| brec::NapiError::invalid_field_name(#js_field, err))?;
    })
}

fn from_napi_field_get(field: &Field, ty: TokenStream) -> Result<TokenStream, E> {
    let rust_field = format_ident!("{}", field.name);
    let js_field = LitStr::new(&field.name, proc_macro2::Span::call_site());
    Ok(match &field.ty {
        Ty::F32 => quote! {
            let #rust_field: u32 = obj
                .get_named_property(#js_field)
                .map_err(|err| brec::NapiError::invalid_field_name(#js_field, err))?;
            let #rust_field = f32::from_bits(#rust_field);
        },
        Ty::F64 => quote! {
            let raw: napi::Unknown<'_> = obj
                .get_named_property(#js_field)
                .map_err(|err| brec::NapiError::invalid_field_name(#js_field, err))?;
            let bits: u64 = match raw
                .get_type()
                .map_err(|err| brec::NapiError::invalid_field_name(#js_field, err))?
            {
                napi::ValueType::BigInt => {
                    let raw_big: napi::bindgen_prelude::BigInt =
                        brec::napi_feature::from_unknown_name(#js_field, raw)?;
                    let (sign, bits, lossless) = raw_big.get_u64();
                    if sign || !lossless {
                        return Err(brec::NapiError::invalid_field_name(
                            #js_field,
                            "BigInt is out of range for f64",
                        ));
                    }
                    bits
                }
                other => {
                    return Err(brec::NapiError::invalid_field_name(
                        #js_field,
                        format!("expected BigInt, got {:?}", other),
                    ));
                }
            };
            let #rust_field = f64::from_bits(bits);
        },
        Ty::I64 => quote! {
            let raw: napi::Unknown<'_> = obj
                .get_named_property(#js_field)
                .map_err(|err| brec::NapiError::invalid_field_name(#js_field, err))?;
            let #rust_field: #ty = match raw
                .get_type()
                .map_err(|err| brec::NapiError::invalid_field_name(#js_field, err))?
            {
                napi::ValueType::BigInt => {
                    let raw_big: napi::bindgen_prelude::BigInt =
                        brec::napi_feature::from_unknown_name(#js_field, raw)?;
                    let (value, lossless) = raw_big.get_i64();
                    if !lossless {
                        return Err(brec::NapiError::invalid_field_name(
                            #js_field,
                            "BigInt is out of range for i64",
                        ));
                    }
                    value
                }
                other => {
                    return Err(brec::NapiError::invalid_field_name(
                        #js_field,
                        format!("expected BigInt, got {:?}", other),
                    ));
                }
            };
        },
        Ty::U64 => quote! {
            let raw: napi::Unknown<'_> = obj
                .get_named_property(#js_field)
                .map_err(|err| brec::NapiError::invalid_field_name(#js_field, err))?;
            let #rust_field: #ty = match raw
                .get_type()
                .map_err(|err| brec::NapiError::invalid_field_name(#js_field, err))?
            {
                napi::ValueType::BigInt => {
                    let raw_big: napi::bindgen_prelude::BigInt =
                        brec::napi_feature::from_unknown_name(#js_field, raw)?;
                    let (sign, value, lossless) = raw_big.get_u64();
                    if sign || !lossless {
                        return Err(brec::NapiError::invalid_field_name(
                            #js_field,
                            "BigInt is out of range for u64",
                        ));
                    }
                    value
                }
                other => {
                    return Err(brec::NapiError::invalid_field_name(
                        #js_field,
                        format!("expected BigInt, got {:?}", other),
                    ));
                }
            };
        },
        Ty::I128 => quote! {
            let raw: napi::Unknown<'_> = obj
                .get_named_property(#js_field)
                .map_err(|err| brec::NapiError::invalid_field_name(#js_field, err))?;
            let #rust_field: #ty = match raw
                .get_type()
                .map_err(|err| brec::NapiError::invalid_field_name(#js_field, err))?
            {
                napi::ValueType::BigInt => {
                    let raw_big: napi::bindgen_prelude::BigInt =
                        brec::napi_feature::from_unknown_name(#js_field, raw)?;
                    let (value, lossless) = raw_big.get_i128();
                    if !lossless {
                        return Err(brec::NapiError::invalid_field_name(
                            #js_field,
                            "BigInt is out of range for i128",
                        ));
                    }
                    value
                }
                other => {
                    return Err(brec::NapiError::invalid_field_name(
                        #js_field,
                        format!("expected BigInt, got {:?}", other),
                    ));
                }
            };
        },
        Ty::U128 => quote! {
            let raw: napi::Unknown<'_> = obj
                .get_named_property(#js_field)
                .map_err(|err| brec::NapiError::invalid_field_name(#js_field, err))?;
            let #rust_field: #ty = match raw
                .get_type()
                .map_err(|err| brec::NapiError::invalid_field_name(#js_field, err))?
            {
                napi::ValueType::BigInt => {
                    let raw_big: napi::bindgen_prelude::BigInt =
                        brec::napi_feature::from_unknown_name(#js_field, raw)?;
                    let (sign, value, lossless) = raw_big.get_u128();
                    if sign || !lossless {
                        return Err(brec::NapiError::invalid_field_name(
                            #js_field,
                            "BigInt is out of range for u128",
                        ));
                    }
                    value
                }
                other => {
                    return Err(brec::NapiError::invalid_field_name(
                        #js_field,
                        format!("expected BigInt, got {:?}", other),
                    ));
                }
            };
        },
        Ty::Blob(len) => quote! {
            let raw: Vec<u8> = obj
                .get_named_property(#js_field)
                .map_err(|err| brec::NapiError::invalid_field_name(#js_field, err))?;
            let #rust_field: [u8; #len] = raw.try_into().map_err(|bytes: Vec<u8>| {
                brec::NapiError::invalid_field_name(
                    #js_field,
                    format!("expected {} bytes, got {}", #len, bytes.len()),
                )
            })?;
        },
        Ty::LinkedToU8(enum_name) => quote! {
            let raw: u8 = obj
                .get_named_property(#js_field)
                .map_err(|err| brec::NapiError::invalid_field_name(#js_field, err))?;
            let #rust_field = #ty::try_from(raw)
                .map_err(|err| brec::Error::FailedConverting(#enum_name.to_owned(), err))?;
        },
        _ => quote! {
            let #rust_field: #ty = obj
                .get_named_property(#js_field)
                .map_err(|err| brec::NapiError::invalid_field_name(#js_field, err))?;
        },
    })
}

pub fn generate(name: &Ident, fields: &[Field]) -> Result<TokenStream, E> {
    let to_napi = fields
        .iter()
        .filter(|field| !field.injected)
        .map(to_napi_field_set)
        .collect::<Result<Vec<_>, _>>()?;
    let from_napi = fields
        .iter()
        .filter(|field| !field.injected)
        .map(|field| from_napi_field_get(field, field.ty.direct()))
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
        impl #name {
            fn to_napi_object<'env>(&self, env: &'env napi::Env) -> Result<napi::Unknown<'env>, brec::Error> {
                use napi::bindgen_prelude::{JsObjectValue, JsValue, Object};
                let mut obj: Object<'env> = Object::new(env)
                    .map_err(|err| brec::Error::Napi(brec::NapiError::InvalidObject(err.to_string())))?;
                #(#to_napi)*
                Ok(obj.to_unknown())
            }

            fn from_napi_object(value: napi::Unknown<'_>) -> Result<Self, brec::Error> {
                use napi::bindgen_prelude::{JsObjectValue, JsValue};
                let obj: napi::bindgen_prelude::Object<'_> =
                    brec::napi_feature::from_unknown_name("object", value)
                        .map_err(|err| brec::Error::Napi(brec::NapiError::InvalidObject(err.to_string())))?;
                #(#from_napi)*
                Ok(Self {
                    #(#ctor_fields)*
                })
            }

            pub fn decode_napi<'env>(
                env: &'env napi::Env,
                bytes: napi::bindgen_prelude::Buffer,
            ) -> Result<napi::Unknown<'env>, brec::Error> {
                let mut src = bytes.as_ref();
                let block = <#name as brec::ReadBlockFrom>::read(&mut src, false)?;
                block.to_napi_object(env)
            }

            pub fn encode_napi(value: napi::Unknown<'_>, out: &mut Vec<u8>) -> Result<(), brec::Error> {
                let block = #name::from_napi_object(value)?;
                brec::WriteTo::write_all(&block, out)?;
                Ok(())
            }
        }

        impl brec::NapiObject for #name {
            fn to_napi_object<'env>(&self, env: &'env napi::Env) -> Result<napi::Unknown<'env>, brec::Error> {
                #name::to_napi_object(self, env)
            }

            fn from_napi_object(_env: &napi::Env, value: napi::Unknown<'_>) -> Result<Self, brec::Error> {
                #name::from_napi_object(value)
            }
        }
    })
}
