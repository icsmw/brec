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
    blocks: HashMap<String, HashMap<String, Block>>,
    payloads: HashMap<String, HashMap<String, Payload>>,
}

impl Collector {
    pub fn get<'a>() -> Result<MutexGuard<'a, Collector>, E> {
        COLLECTOR.lock().map_err(|_| E::NoAccessToCollector)
    }
    pub fn add_block(&mut self, block: Block) -> Result<(), E> {
        let blocks = self.blocks.entry(get_pkg_name()).or_default();
        let fname = block.fullname()?.to_string();
        blocks.entry(fname).or_insert(block);
        Ok(())
    }
    pub fn add_payload(&mut self, payload: Payload) -> Result<(), E> {
        let payloads = self.payloads.entry(get_pkg_name()).or_default();
        let fname = payload.fullname()?.to_string();
        payloads.entry(fname).or_insert(payload);
        Ok(())
    }
    pub fn is_blocks_empty(&mut self) -> bool {
        self.blocks.entry(get_pkg_name()).or_default().is_empty()
    }
    pub fn is_payloads_empty(&mut self) -> bool {
        self.payloads.entry(get_pkg_name()).or_default().is_empty()
    }
    pub fn write(&mut self, cfg: &Config) -> Result<(), E> {
        let pkg_name = get_pkg_name();
        let block = if self.is_blocks_empty() {
            quote! {}
        } else {
            blocks::gen(
                self.blocks
                    .entry(pkg_name.clone())
                    .or_default()
                    .values()
                    .collect::<Vec<&Block>>(),
                cfg,
            )?
        };
        let payload = if self.is_payloads_empty() && cfg.is_no_default_payloads() {
            quote! {}
        } else {
            payloads::gen(
                self.payloads
                    .entry(pkg_name)
                    .or_default()
                    .values()
                    .collect::<Vec<&Payload>>(),
                cfg,
            )?
        };
        let packet = if self.is_blocks_empty()
            || (self.is_payloads_empty() && cfg.is_no_default_payloads())
        {
            quote! {}
        } else {
            packet::gen()?
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
