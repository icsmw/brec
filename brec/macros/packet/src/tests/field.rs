use crate::tests::*;
use proptest::prelude::*;
use quote::{format_ident, quote};

#[derive(Debug)]
pub(crate) struct Field {
    pub name: String,
    pub value: Value,
}

impl Field {
    pub fn is_ordered_ty(&self) -> bool {
        self.value.is_ordered_ty()
    }
}

impl Arbitrary for Field {
    type Parameters = (Target, u8);

    type Strategy = BoxedStrategy<Self>;

    fn arbitrary_with((owner, deep): (Target, u8)) -> Self::Strategy {
        (
            gen_name(),
            if deep > MAX_VALUE_DEEP || matches!(owner, Target::Block) {
                Target::block_values()
            } else {
                Target::payload_values()
            },
        )
            .prop_flat_map(move |(name, id)| {
                Value::arbitrary_with((id, deep + 1)).prop_map(move |value: Value| Field {
                    name: name.clone(),
                    value,
                })
            })
            .boxed()
    }
}

impl Generate for Field {
    type Options = ();

    fn declaration(&self, _: Self::Options) -> TokenStream {
        let name = format_ident!("{}", self.name);
        let ty = self.value.declaration(());
        quote! {
            #name: #ty
        }
    }
    fn instance(&self, _: Self::Options) -> TokenStream {
        let name = format_ident!("{}", self.name);
        let vl = self.value.instance(());
        quote! {
            #name: #vl
        }
    }
}
