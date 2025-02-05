mod field;
use std::{
    collections::HashSet,
    env,
    fs::{self, File},
    io::{self, Write},
    path::PathBuf,
};

pub(crate) use field::*;

use proc_macro2::TokenStream;
use proptest::prelude::*;
use quote::{format_ident, quote};
use uuid::Uuid;

#[derive(Debug, Default)]
struct Block {
    pub name: String,
    pub fields: Vec<BlockField>,
}

impl Block {
    pub fn as_dec(&self) -> TokenStream {
        let name = format_ident!("{}", self.name);
        let fields = self
            .fields
            .iter()
            .map(|f| f.as_dec())
            .collect::<Vec<TokenStream>>();
        quote! {
            #[block]
            struct #name {
                #(#fields,)*
            }
        }
    }
    pub fn as_val(&self) -> TokenStream {
        let name = format_ident!("{}", self.name);
        let fields = self
            .fields
            .iter()
            .map(|f| f.as_val())
            .collect::<Vec<TokenStream>>();
        quote! {
            #name {
                #(#fields,)*
            }
        }
    }
}

impl Arbitrary for Block {
    type Parameters = ();

    type Strategy = BoxedStrategy<Self>;

    fn arbitrary_with(_: ()) -> Self::Strategy {
        (1usize..=20)
            .prop_flat_map(|len| {
                (
                    "[a-z][a-z0-9]*".prop_map(String::from),
                    prop::collection::vec(BlockField::arbitrary_with(()), len).boxed(),
                )
            })
            .prop_map(|(name, mut fields)| {
                let mut seen = HashSet::new();
                fields.retain(|f| seen.insert(f.name.clone()));
                Block { name, fields }
            })
            .boxed()
    }
}

struct Project {
    pub blocks: Vec<Block>,
    pub name: Uuid,
}

impl Project {
    pub fn write(&self) -> io::Result<()> {
        let root = self.root();
        let tests = root.join("../../tests");
        if !tests.exists() {
            fs::create_dir(&tests)?;
        }
        let proj = tests.join(self.name.to_string());
        let src = proj.join("src");
        fs::create_dir(&proj)?;
        fs::create_dir(&src)?;
        let mut file = File::create(proj.join("Cargo.toml"))?;
        file.write_all(self.cargo_toml().as_bytes())?;
        let mut file = File::create(proj.join("build.rs"))?;
        file.write_all(self.build_rs().as_bytes())?;
        let mut file = File::create(src.join("main.rs"))?;
        file.write_all(self.main_rs().as_bytes())?;
        Ok(())
    }
    pub fn root(&self) -> PathBuf {
        PathBuf::from(env::var("CARGO_MANIFEST_DIR").expect("CARGO_MANIFEST_DIR is not set"))
    }
    pub fn main_rs(&self) -> String {
        let decs = self
            .blocks
            .iter()
            .map(|blk| blk.as_dec())
            .collect::<Vec<TokenStream>>();
        let mut vars = Vec::new();
        let insts = self
            .blocks
            .iter()
            .map(|blk| {
                let name = format_ident!("inst_{}", blk.name);
                vars.push(name.clone());
                let val = blk.as_val();
                quote! { let #name = #val;}
            })
            .collect::<Vec<TokenStream>>();
        quote! {
            use brec::*;

            #(#decs)*

            fn main() {
                #(#insts)*
            }
        }
        .to_string()
    }
    pub fn cargo_toml(&self) -> String {
        r#"[package]
name = "test_case"
version = "0.0.0"
edition = "2021"

[dependencies]
brec = { path = "../../"}

[build-dependencies]
brec = { path = "../../", features=["build"]}"#
            .to_string()
    }
    pub fn build_rs(&self) -> String {
        r#"fn main() {
    brec::build_setup();
}"#
        .to_string()
    }
}

proptest! {
    #![proptest_config(ProptestConfig {
        max_shrink_iters: 50,
        ..ProptestConfig::with_cases(5)
    })]


    #[test]
    fn test(mut blocks in proptest::collection::vec(Block::arbitrary(), 1..20)) {
        let mut seen = HashSet::new();
        blocks.retain(|blk| seen.insert(blk.name.clone()));
        let pro = Project { blocks, name: Uuid::new_v4()};
        pro.write()?;
        // for blk in blks.into_iter() {

        // }
    }

}
