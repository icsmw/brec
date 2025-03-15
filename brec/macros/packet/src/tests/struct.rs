use crate::tests::*;
use proptest::prelude::*;
use quote::{format_ident, quote};

#[derive(Debug)]
pub(crate) struct Struct {
    pub name: String,
    pub fields: Vec<Field>,
}

impl Arbitrary for Struct {
    type Parameters = (Target, u8);

    type Strategy = BoxedStrategy<Self>;

    fn arbitrary_with((target, deep): (Target, u8)) -> Self::Strategy {
        (
            "[a-z][A-Z]*".prop_filter("name already exist", |s| chk_name(s)),
            prop::collection::vec(Field::arbitrary_with((target, deep + 1)), 1..10),
        )
            .prop_map(move |(name, fields)| Struct { name, fields })
            .boxed()
    }
}

impl Generate for Struct {
    type Options = Target;
    fn declaration(&self, target: Target) -> TokenStream {
        let name = format_ident!("{}", self.name);
        let fields = self
            .fields
            .iter()
            .map(|f| f.declaration(()))
            .collect::<Vec<TokenStream>>();
        let mc = match target {
            Target::Block => quote! {
                #[block]
                #[derive(Debug)]
                #[allow(non_snake_case, non_camel_case_types)]
            },
            Target::Payload => quote! {
                #[payload(bincode)]
                #[derive(serde::Deserialize, serde::Serialize, Debug)]
                #[allow(non_snake_case, non_camel_case_types)]
            },
        };
        quote! {
            #mc
            struct #name {
                #(#fields,)*
            }
        }
    }
    fn instance(&self, _: Target) -> TokenStream {
        let name = format_ident!("{}", self.name);
        let fields = self
            .fields
            .iter()
            .map(|f| f.instance(()))
            .collect::<Vec<TokenStream>>();
        quote! {
            #name {
                #(#fields,)*
            }
        }
    }
}
