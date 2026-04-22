use lazy_static::lazy_static;
use std::{
    collections::HashMap,
    sync::{Mutex, MutexGuard},
};

use crate::*;

lazy_static! {
    static ref COLLECTOR: Mutex<Collector> = Mutex::new(Collector::default());
}

pub fn get_pkg_name() -> String {
    std::env::var("CARGO_PKG_NAME").unwrap_or_else(|_| "unknown".to_string())
}

#[derive(Debug, Default)]
pub struct Collector {
    pub blocks: HashMap<String, HashMap<String, Block>>,
    pub payloads: HashMap<String, HashMap<String, Payload>>,
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
}
