use crate::tests::*;
use proc_macro2::TokenStream;
use proptest::prelude::*;
use quote::{format_ident, quote};

#[derive(Debug)]
pub struct Packet {
    blocks: Vec<Struct>,
    payload: Option<Struct>,
    pub name: String,
}

impl Arbitrary for Packet {
    type Parameters = ();

    type Strategy = BoxedStrategy<Self>;

    fn arbitrary_with(_: ()) -> Self::Strategy {
        (
            gen_name(),
            prop::collection::vec(Struct::arbitrary_with((Target::Block, 0)), 1..10),
            prop::option::of(Struct::arbitrary_with((Target::Payload, 0))),
        )
            .prop_map(move |(name, blocks, payload)| Packet {
                name,
                blocks,
                payload,
            })
            .boxed()
    }
}

fn variant_name<S: AsRef<str>>(name: S) -> String {
    name.as_ref()
        .split("::")
        .map(|s| {
            let mut chars = s.trim().chars();
            match chars.next() {
                Some(first) => first.to_uppercase().chain(chars).collect(),
                None => String::new(),
            }
        })
        .collect::<Vec<String>>()
        .join("")
}
impl Generate for Packet {
    type Options = ();
    fn declaration(&self, _: Self::Options) -> TokenStream {
        let blocks = self
            .blocks
            .iter()
            .map(|b| b.declaration(Target::Block))
            .collect::<Vec<TokenStream>>();
        let payload = self
            .payload
            .as_ref()
            .map(|p| p.declaration(Target::Payload))
            .unwrap_or_default();
        quote! {
            #(#blocks)*
            #payload
        }
    }
    fn instance(&self, _: Self::Options) -> TokenStream {
        let blocks = self
            .blocks
            .iter()
            .map(|b| {
                let instance = b.instance(Target::Block);
                let name = format_ident!("{}", variant_name(&b.name));
                quote! {
                    Block::#name(#instance)
                }
            })
            .collect::<Vec<TokenStream>>();
        let payload = self
            .payload
            .as_ref()
            .map(|p| {
                let payload = p.instance(Target::Payload);
                let name = format_ident!("{}", variant_name(&p.name));
                quote! { Some( Payload::#name(#payload) ) }
            })
            .unwrap_or(quote! { None });
        quote! {
            Packet::new(
                vec![#(#blocks,)*],
                #payload
            )
        }
    }
}
