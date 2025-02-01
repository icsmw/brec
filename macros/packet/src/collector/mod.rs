use std::{
    env,
    fs::File,
    io::Write,
    path::PathBuf,
    sync::{Mutex, MutexGuard},
};

use crate::*;

use lazy_static::lazy_static;

lazy_static! {
    static ref COLLECTOR: Mutex<Collector> = Mutex::new(Collector::default());
}

#[derive(Debug, Default)]
pub struct Collector {
    blocks: Vec<Block>,
}

impl Collector {
    pub fn get<'a>() -> Result<MutexGuard<'a, Collector>, E> {
        COLLECTOR.lock().map_err(|_| E::NoAccessToCollector)
    }
    pub fn add_block(&mut self, block: Block) -> Result<(), E> {
        self.blocks.push(block);
        self.write()
    }
    fn write(&self) -> Result<(), E> {
        let mut variants = Vec::new();
        for blk in self.blocks.iter() {
            let fullname = blk.fullname()?;
            let fullpath = blk.fullpath()?;
            variants.push(quote! {#fullname(#fullpath)});
        }
        let enum_block = quote! {
            pub (crate) enum Block {
                #(#variants,)*
            }
        };
        let out_dir = env::var("OUT_DIR")?;
        let path = PathBuf::from(out_dir).join("brec.rs");
        let mut file = File::create(&path)?;
        file.write_all(enum_block.to_string().as_bytes())?;
        Ok(())
    }
}
