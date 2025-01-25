use proc_macro2::TokenStream;
use quote::quote;
use syn::{GenericArgument, Ident, PathArguments, Type};

fn get_ty_by_ident(ident: &Ident) -> Result<TokenStream, String> {
    match ident.to_string().as_str() {
        "u8" | "u16" | "u32" | "u64" | "u128" | "usize" | "i8" | "i16" | "i32" | "i64" | "i128"
        | "isize" => Ok(quote! { DeterminedTy::Num }),
        "bool" => Ok(quote! { DeterminedTy::Bool}),
        "String" => Ok(quote! { DeterminedTy::Str}),
        "PathBuf" => Ok(quote! { DeterminedTy::PathBuf }),
        _ => Err("Not supported argument type".to_owned()),
    }
}

pub fn get_ty(ty: &Type) -> Result<TokenStream, String> {
    match ty {
        Type::Path(ty) => {
            if let Some(ident) = ty.path.get_ident() {
                get_ty_by_ident(ident)
            } else if let Some(segment) = &ty.path.segments.first() {
                match segment.ident.to_string().as_str() {
                    "Vec" => {
                        let PathArguments::AngleBracketed(ref args) = segment.arguments else {
                            return Err("Only AngleBracketed is supported for Vec".to_owned());
                        };
                        let Some(GenericArgument::Type(Type::Path(inner_ty))) = args.args.first()
                        else {
                            return Err("Only Path is supported for Vec".to_owned());
                        };
                        let Some(inner_ident) = inner_ty.path.get_ident() else {
                            return Err("Fail to get ident for inner type of Vec".to_owned());
                        };
                        let inner_ref = get_ty_by_ident(inner_ident)?;
                        Ok(quote! {
                            DeterminedTy::Vec(Some(Box::new(#inner_ref)))
                        })
                    }
                    "HashMap" => {
                        return Err("HashMap not implemented".to_owned());
                    }
                    "Option" => {
                        return Err("Option not implemented".to_owned());
                    }
                    "Result" => {
                        return Err("Result not implemented".to_owned());
                    }
                    _ => {
                        return Err("Only Vec is supported".to_owned());
                    }
                }
            } else {
                return Err("Only Type::Path are supported".to_owned());
            }
        }
        Type::Tuple(tuple) => {
            if tuple.elems.is_empty() {
                Ok(quote! { DeterminedTy::Void })
            } else {
                Err("Non-empty tuples arn't supported".to_owned())
            }
        }
        _ => Err("Only Type::Path are supported".to_owned()),
    }
}
