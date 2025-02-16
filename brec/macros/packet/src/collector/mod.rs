mod blocks;

pub(crate) use blocks::*;

use std::{
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

#[derive(Debug, Default)]
pub struct Collector {
    blocks: Vec<Block>,
    payloads: Vec<Payload>,
}

impl Collector {
    pub fn get<'a>() -> Result<MutexGuard<'a, Collector>, E> {
        COLLECTOR.lock().map_err(|_| E::NoAccessToCollector)
    }
    pub fn add_block(&mut self, block: Block) -> Result<(), E> {
        self.blocks.push(block);
        self.write()
    }
    pub fn add_payload(&mut self, payload: Payload) -> Result<(), E> {
        self.payloads.push(payload);
        self.write()
    }
    fn write(&self) -> Result<(), E> {
        let block = blocks::gen(&self.blocks)?;
        let output = quote! {
            #block
        };
        let out_dir = env::var("OUT_DIR")?;
        let path = PathBuf::from(out_dir).join("brec.rs");
        let mut file = File::create(&path)?;
        file.write_all(output.to_string().as_bytes())?;
        Ok(())
    }
}
