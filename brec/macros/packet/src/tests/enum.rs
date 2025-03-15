use crate::tests::*;
use proptest::prelude::*;
use quote::{format_ident, quote};

#[derive(Debug)]
pub(crate) struct Enum {
    pub name: String,
    pub fields: Vec<Field>,
}

impl Arbitrary for Enum {
    type Parameters = u8;

    type Strategy = BoxedStrategy<Self>;

    fn arbitrary_with(deep: u8) -> Self::Strategy {
        prop::collection::vec(Field::arbitrary_with((Target::Payload, deep + 1)), 1..10)
            .prop_map(move |fields| Enum {
                name: gen_name(true),
                fields,
            })
            .boxed()
    }
}

impl Generate for Enum {
    type Options = ();
    fn declaration(&self, _: ()) -> TokenStream {
        let name = format_ident!("{}", self.name);
        let fields = self
            .fields
            .iter()
            .map(|f| {
                let variant = format_ident!("{}", f.name);
                if matches!(f.value, Value::Option(..)) {
                    quote! { #variant }
                } else {
                    let ty = f.value.declaration(());
                    quote! { #variant: #ty }
                }
            })
            .collect::<Vec<TokenStream>>();
        quote! {
            #[derive(serde::Deserialize, serde::Serialize, PartialEq, PartialOrd, Debug)]
            enum #name {
                #(#fields,)*
            }
        }
    }
    fn instance(&self, _: ()) -> TokenStream {
        let name = format_ident!("{}", self.name);
        let variant = self
            .fields
            .first()
            .map(|f| {
                let variant = format_ident!("{}", f.name);
                if matches!(f.value, Value::Option(..)) {
                    quote! { #variant }
                } else {
                    let ty = f.value.declaration(());
                    quote! { #variant( #ty )}
                }
            })
            .unwrap_or_default();
        quote! {
            #name::#variant
        }
    }
}
