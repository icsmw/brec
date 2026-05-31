use crate::*;
use proc_macro2::TokenStream;
use quote::quote;
use syn::Path;

#[enum_ids::enum_ids(display_variant_snake)]
#[derive(Debug, Clone)]
pub enum Setting {
    NoDefaultPayload,
    Scheme,
    PayloadsDerive(String),
    DefaultMaxPacketLen(u32),
    DefaultMaxPayloadLen(u32),
    DefaultInitialPacketBufferCapacity(usize),
}

pub struct Config(pub Vec<Setting>);

impl Config {
    pub fn is_no_default_payloads(&self) -> bool {
        self.0
            .iter()
            .any(|attr| matches!(attr, Setting::NoDefaultPayload))
    }
    pub fn is_scheme(&self) -> bool {
        self.0.iter().any(|attr| matches!(attr, Setting::Scheme))
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
    pub fn get_default_max_payload_len(&self) -> Option<u32> {
        self.0.iter().find_map(|attr| {
            if let Setting::DefaultMaxPayloadLen(len) = attr {
                Some(*len)
            } else {
                None
            }
        })
    }
    pub fn get_default_max_packet_len(&self) -> Option<u32> {
        self.0.iter().find_map(|attr| {
            if let Setting::DefaultMaxPacketLen(len) = attr {
                Some(*len)
            } else {
                None
            }
        })
    }
    pub fn get_default_initial_packet_buffer_capacity(&self) -> Option<usize> {
        self.0.iter().find_map(|attr| {
            if let Setting::DefaultInitialPacketBufferCapacity(cap) = attr {
                Some(*cap)
            } else {
                None
            }
        })
    }
}
