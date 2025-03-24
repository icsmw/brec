use crate::tests::*;
use proptest::prelude::*;
use quote::quote;
use std::collections::HashMap;
use syn::LitStr;

pub(crate) const MAX_VALUE_DEEP: u8 = 1;

#[enum_ids::enum_ids]
#[derive(Debug, Clone)]
pub(crate) enum Value {
    U8(u8),
    U16(u16),
    U32(u32),
    U64(u64),
    U128(u128),
    I8(i8),
    I16(i16),
    I32(i32),
    I64(i64),
    I128(i128),
    F32(f32),
    F64(f64),
    Bool(bool),
    Blob(Vec<u8>),
    String(String),
    Option(Option<Box<Value>>),
    Tuple(Box<Value>, Box<Value>),
    HashMap(HashMap<String, Value>),
    Vec(Vec<Value>),
}

impl Value {
    pub fn is_ordered_ty(&self) -> bool {
        match self {
            Self::U8(..)
            | Self::U16(..)
            | Self::U32(..)
            | Self::U64(..)
            | Self::U128(..)
            | Self::I8(..)
            | Self::I16(..)
            | Self::I32(..)
            | Self::I64(..)
            | Self::I128(..)
            | Self::F32(..)
            | Self::F64(..)
            | Self::Bool(..)
            | Self::Blob(..)
            | Self::String(..) => true,
            Self::HashMap(..) => false,
            Self::Vec(v) => v.first().map(|v| v.is_ordered_ty()).unwrap_or(true),
            Self::Tuple(a, b) => {
                let a = a.is_ordered_ty();
                let b = b.is_ordered_ty();
                a && b
            }
            Self::Option(v) => v.as_ref().map(|v| v.is_ordered_ty()).unwrap_or(true),
        }
    }
}

impl Default for ValueId {
    fn default() -> Self {
        Self::U8
    }
}

impl Arbitrary for Value {
    type Parameters = (ValueId, u8);

    type Strategy = BoxedStrategy<Self>;

    fn arbitrary_with((id, deep): (ValueId, u8)) -> Self::Strategy {
        match id {
            ValueId::U8 => any::<u8>().prop_map(Value::U8).boxed(),
            ValueId::U16 => any::<u16>().prop_map(Value::U16).boxed(),
            ValueId::U32 => any::<u32>().prop_map(Value::U32).boxed(),
            ValueId::U64 => any::<u64>().prop_map(Value::U64).boxed(),
            ValueId::U128 => any::<u128>().prop_map(Value::U128).boxed(),
            ValueId::I8 => any::<i8>().prop_map(Value::I8).boxed(),
            ValueId::I16 => any::<i16>().prop_map(Value::I16).boxed(),
            ValueId::I32 => any::<i32>().prop_map(Value::I32).boxed(),
            ValueId::I64 => any::<i64>().prop_map(Value::I64).boxed(),
            ValueId::I128 => any::<i128>().prop_map(Value::I128).boxed(),
            ValueId::F32 => any::<f32>()
                .prop_filter("not NaN; not Inf", |v| !v.is_infinite() && !v.is_nan())
                .prop_map(Value::F32)
                .boxed(),
            ValueId::F64 => any::<f64>()
                .prop_filter("not NaN; not Inf", |v| !v.is_infinite() && !v.is_nan())
                .prop_map(Value::F64)
                .boxed(),
            ValueId::Bool => any::<bool>().prop_map(Value::Bool).boxed(),
            ValueId::Blob => prop::collection::vec(any::<u8>(), 0..100)
                .prop_map(|v| Value::Blob(v.into_iter().collect()))
                .boxed(),
            ValueId::String => any::<String>().prop_map(Value::String).boxed(),
            ValueId::Vec => if deep > MAX_VALUE_DEEP {
                Target::primitive_values()
            } else {
                Target::nested_values()
            }
            .prop_flat_map(move |id| {
                prop::collection::vec(Value::arbitrary_with((id, deep + 1)), 0..100)
                    .prop_map(Value::Vec)
            })
            .boxed(),
            ValueId::HashMap => if deep > MAX_VALUE_DEEP {
                Target::primitive_values()
            } else {
                Target::nested_values()
            }
            .prop_flat_map(move |id| {
                prop::collection::vec(Value::arbitrary_with((id, deep + 1)), 0..100).prop_map(
                    |els| {
                        let mut map = HashMap::new();
                        for (n, el) in els.into_iter().enumerate() {
                            map.insert(n.to_string(), el);
                        }
                        Value::HashMap(map)
                    },
                )
            })
            .boxed(),
            ValueId::Tuple => if deep > MAX_VALUE_DEEP {
                (Target::primitive_values(), Target::primitive_values())
            } else {
                (Target::nested_values(), Target::nested_values())
            }
            .prop_flat_map(move |(a, b)| {
                (
                    Value::arbitrary_with((a, deep + 1)),
                    Value::arbitrary_with((b, deep + 1)),
                )
                    .prop_map(|(a, b)| Value::Tuple(Box::new(a), Box::new(b)))
            })
            .boxed(),
            ValueId::Option => if deep > MAX_VALUE_DEEP {
                Target::primitive_values()
            } else {
                Target::nested_values()
            }
            .prop_flat_map(move |id| {
                prop::option::of(Value::arbitrary_with((id, deep + 1)))
                    .prop_map(|v| Value::Option(v.map(Box::new)))
            })
            .boxed(),
        }
    }
}

impl Generate for Value {
    type Options = ();
    fn declaration(&self, _: ()) -> TokenStream {
        match self {
            Self::U8(..) => quote! { u8 },
            Self::U16(..) => quote! { u16 },
            Self::U32(..) => quote! { u32 },
            Self::U64(..) => quote! { u64 },
            Self::U128(..) => quote! { u128 },
            Self::I8(..) => quote! { i8 },
            Self::I16(..) => quote! { i16 },
            Self::I32(..) => quote! { i32 },
            Self::I64(..) => quote! { i64 },
            Self::I128(..) => quote! { i128 },
            Self::F32(..) => quote! { f32 },
            Self::F64(..) => quote! { f64 },
            Self::Bool(..) => quote! { bool },
            Self::Blob(v) => {
                let len = v.len();
                quote! { [u8; #len] }
            }
            Self::String(..) => quote! { String },
            Self::HashMap(v) => {
                let ty = v
                    .values()
                    .next()
                    .map(|v| v.declaration(()))
                    .unwrap_or(quote! { u8 });
                quote! { std::collections::HashMap<String, #ty>}
            }
            Self::Vec(v) => {
                let ty = v
                    .first()
                    .map(|v| v.declaration(()))
                    .unwrap_or(quote! { u8 });
                quote! { Vec<#ty> }
            }
            Self::Tuple(a, b) => {
                let a = a.declaration(());
                let b = b.declaration(());
                quote! { (#a, #b) }
            }
            Self::Option(v) => {
                let ty = v
                    .as_ref()
                    .map(|v| v.declaration(()))
                    .unwrap_or(quote! { u8 });
                quote! { Option<#ty> }
            }
        }
    }
    fn instance(&self, _: ()) -> TokenStream {
        match self {
            Self::U8(v) => quote! { #v },
            Self::U16(v) => quote! { #v },
            Self::U32(v) => quote! { #v },
            Self::U64(v) => quote! { #v },
            Self::U128(v) => quote! { #v },
            Self::I8(v) => quote! { #v },
            Self::I16(v) => quote! { #v },
            Self::I32(v) => quote! { #v },
            Self::I64(v) => quote! { #v },
            Self::I128(v) => quote! { #v },
            Self::F32(v) => quote! { #v },
            Self::F64(v) => quote! { #v },
            Self::Bool(v) => quote! { #v },
            Self::Blob(v) => {
                let vals = v
                    .iter()
                    .map(|v| quote! { #v })
                    .collect::<Vec<TokenStream>>();
                quote! {[#(#vals,)*]}
            }
            Self::String(s) => {
                let s = LitStr::new(s, proc_macro2::Span::call_site());
                quote! { String::from(#s) }
            }
            Self::HashMap(v) => {
                let vals = v
                    .iter()
                    .map(|(key, v)| {
                        let v = v.instance(());
                        quote! {(String::from(#key), #v)}
                    })
                    .collect::<Vec<TokenStream>>();
                quote! { std::collections::HashMap::from([#(#vals,)*]) }
            }
            Self::Vec(v) => {
                let vals = v
                    .iter()
                    .map(|v| v.instance(()))
                    .collect::<Vec<TokenStream>>();
                quote! {vec![#(#vals,)*]}
            }
            Self::Tuple(a, b) => {
                let a = a.instance(());
                let b = b.instance(());
                quote! { (#a, #b) }
            }
            Self::Option(v) => {
                if let Some(v) = v {
                    let v = v.instance(());
                    quote! { Some(#v) }
                } else {
                    quote! { None }
                }
            }
        }
    }
}
