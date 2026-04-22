use brec_macros_parser::*;
use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use syn::{Ident, LitStr};

fn to_java_field_set(field: &Field) -> Result<TokenStream, E> {
    let rust_field = format_ident!("{}", field.name);
    let java_field = LitStr::new(&field.name, proc_macro2::Span::call_site());
    let value = match &field.ty {
        Ty::LinkedToU8(_) => quote! {{
            let value: u8 = (&self.#rust_field).into();
            <u8 as brec::JavaConvert>::to_java_value(&value, env)?
        }},
        _ => {
            let ty = field.ty.direct();
            quote! { <#ty as brec::JavaConvert>::to_java_value(&self.#rust_field, env)? }
        }
    };
    Ok(quote! {
        let field_value = #value;
        brec::java_feature::map_put(env, &obj, #java_field, &field_value)
            .map_err(|err| brec::JavaError::invalid_field_name(#java_field, err))?;
    })
}

fn from_java_field_get(field: &Field) -> Result<TokenStream, E> {
    let rust_field = format_ident!("{}", field.name);
    let java_field = LitStr::new(&field.name, proc_macro2::Span::call_site());
    let ty = field.ty.direct();
    Ok(match &field.ty {
        Ty::LinkedToU8(enum_name) => quote! {
            let raw = brec::java_feature::map_get(env, &obj, #java_field)
                .map_err(|err| brec::JavaError::invalid_field_name(#java_field, err))?;
            let raw: u8 = <u8 as brec::JavaConvert>::from_java_value(env, raw)?;
            let #rust_field = #ty::try_from(raw)
                .map_err(|err| brec::Error::FailedConverting(#enum_name.to_owned(), err))?;
        },
        _ => quote! {
            let raw = brec::java_feature::map_get(env, &obj, #java_field)
                .map_err(|err| brec::JavaError::invalid_field_name(#java_field, err))?;
            let #rust_field: #ty = <#ty as brec::JavaConvert>::from_java_value(env, raw)?;
        },
    })
}

pub fn generate(block_name: &Ident, fields: &[Field]) -> Result<TokenStream, E> {
    let to_java = fields
        .iter()
        .filter(|field| !field.injected)
        .map(to_java_field_set)
        .collect::<Result<Vec<_>, _>>()?;
    let from_java = fields
        .iter()
        .filter(|field| !field.injected)
        .map(from_java_field_get)
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
            fn to_java_object<'local>(&self, env: &mut jni::JNIEnv<'local>) -> Result<jni::objects::JObject<'local>, brec::Error> {
                let obj = brec::java_feature::new_hash_map(env)?;
                #(#to_java)*
                Ok(obj)
            }

            fn from_java_object<'local>(env: &mut jni::JNIEnv<'local>, value: jni::objects::JObject<'local>) -> Result<Self, brec::Error> {
                let obj = value;
                #(#from_java)*
                Ok(Self {
                    #(#ctor_fields)*
                })
            }

            pub fn decode_java<'local>(env: &mut jni::JNIEnv<'local>, bytes: &[u8]) -> Result<jni::objects::JObject<'local>, brec::Error> {
                let mut src = bytes;
                let block = <#block_name as brec::ReadBlockFrom>::read(&mut src, false)?;
                block.to_java_object(env)
            }

            pub fn encode_java<'local>(env: &mut jni::JNIEnv<'local>, value: jni::objects::JObject<'local>, out: &mut Vec<u8>) -> Result<(), brec::Error> {
                let block = #block_name::from_java_object(env, value)?;
                brec::WriteTo::write_all(&block, out)?;
                Ok(())
            }
        }

        impl brec::JavaObject for #block_name {
            fn to_java_object<'local>(&self, env: &mut jni::JNIEnv<'local>) -> Result<jni::objects::JObject<'local>, brec::Error> {
                #block_name::to_java_object(self, env)
            }

            fn from_java_object<'local>(env: &mut jni::JNIEnv<'local>, value: jni::objects::JObject<'local>) -> Result<Self, brec::Error> {
                #block_name::from_java_object(env, value)
            }
        }
    })
}
