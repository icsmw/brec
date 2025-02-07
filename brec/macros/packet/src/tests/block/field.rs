use crate::*;
use proc_macro2::TokenStream;
use proptest::prelude::*;
use quote::{format_ident, quote};

#[derive(Debug, Default)]
pub struct BlockField {
    pub name: String,
    pub ty: Ty,
    pub val: TyValue,
}

impl BlockField {
    pub fn as_dec(&self) -> TokenStream {
        let name = format_ident!("{}", self.name);
        let ty = self.ty.direct();
        quote! {
            #name: #ty
        }
    }
    pub fn as_val(&self) -> TokenStream {
        let name = format_ident!("{}", self.name);
        let vl = self.val.into_ts();
        quote! {
            #name: #vl
        }
    }
}

impl Arbitrary for BlockField {
    type Parameters = ();

    type Strategy = BoxedStrategy<Self>;

    fn arbitrary_with(_: ()) -> Self::Strategy {
        (
            "[a-z][a-z0-9]*".prop_map(String::from),
            Ty::arbitrary_with(true),
        )
            .prop_flat_map(|(name, ty)| {
                TyValue::arbitrary_with(ty.clone()).prop_map(move |val| BlockField {
                    name: name.clone(),
                    ty: ty.clone(),
                    val,
                })
            })
            .boxed()
    }
}
