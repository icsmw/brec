mod blocks;
mod packet;
mod payloads;

use std::{
    collections::HashMap,
    env,
    fs::File,
    io::Write,
    path::PathBuf,
    sync::{Mutex, MutexGuard},
};

use crate::*;

use lazy_static::lazy_static;
use quote::quote;

lazy_static! {
    static ref COLLECTOR: Mutex<Collector> = Mutex::new(Collector::default());
}

pub fn get_pkg_name() -> String {
    std::env::var("CARGO_PKG_NAME").unwrap_or_else(|_| "unknown".to_string())
}

#[derive(Debug, Default)]
pub struct Collector {
    blocks: HashMap<String, Vec<Block>>,
    payloads: HashMap<String, Vec<Payload>>,
}

impl Collector {
    pub fn get<'a>() -> Result<MutexGuard<'a, Collector>, E> {
        COLLECTOR.lock().map_err(|_| E::NoAccessToCollector)
    }
    pub fn add_block(&mut self, block: Block) -> Result<(), E> {
        self.blocks.entry(get_pkg_name()).or_default().push(block);
        self.write()
    }
    pub fn add_payload(&mut self, payload: Payload) -> Result<(), E> {
        self.payloads
            .entry(get_pkg_name())
            .or_default()
            .push(payload);
        self.write()
    }
    pub fn has_payloads(&self) -> bool {
        self.payloads.contains_key(&get_pkg_name())
    }
    fn write(&self) -> Result<(), E> {
        let pkg_name = get_pkg_name();
        let block = self
            .blocks
            .get(&pkg_name)
            .map(|blks| blocks::gen(blks))
            .unwrap_or(Ok(quote! {}))?;
        let payload = self
            .payloads
            .get(&pkg_name)
            .map(|plds| payloads::gen(plds))
            .unwrap_or(Ok(quote! {}))?;
        let packet = if self.has_payloads() {
            packet::gen()?
        } else {
            quote! {}
        };
        let output = quote! {
            #block
            #payload
            #packet
        };
        let out_dir = env::var("OUT_DIR")?;
        let path = PathBuf::from(out_dir).join("brec.rs");
        let mut file = File::create(&path)?;
        file.write_all(output.to_string().as_bytes())?;
        Ok(())
    }
}
