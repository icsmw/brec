use crate::*;
use proc_macro2::TokenStream;
use quote::quote;
use syn::Path;

#[enum_ids::enum_ids(display_variant_snake)]
#[derive(Debug, Clone)]
pub enum Setting {
    NoDefaultPayload,
    PayloadsDerive(String),
}

pub struct Config(pub Vec<Setting>);

impl Config {
    pub fn is_no_default_payloads(&self) -> bool {
        self.0
            .iter()
            .any(|attr| matches!(attr, Setting::NoDefaultPayload))
    }
    pub fn get_payload_derive(&self) -> Result<Vec<TokenStream>, E> {
        let Some(Setting::PayloadsDerive(derives)) = self
            .0
            .iter()
            .find(|attr| matches!(attr, Setting::PayloadsDerive(..)))
        else {
            return Ok(Vec::new());
        };
        derives
            .split(',')
            .map(|s| {
                let s = s.trim();
                syn::parse_str::<Path>(s)
                    .map(|path| quote!(#path))
                    .map_err(|_e| E::FailParseDerive(s.to_owned()))
            })
            .collect()
    }
}
