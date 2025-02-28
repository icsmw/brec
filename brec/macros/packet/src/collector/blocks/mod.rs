use proc_macro2::TokenStream;
use quote::quote;

use crate::*;

mod enums;
mod props;
mod read;
mod write;

pub fn gen(blocks: Vec<&Block>) -> Result<TokenStream, E> {
    let derives = Derives::common(blocks.iter().map(|b| &b.derives).collect());
    let block = enums::gen(&blocks, derives)?;
    let block_referred = enums::gen_referred(&blocks)?;
    let prop = props::gen(&blocks)?;
    let prop_referred = props::gen_referred(&blocks)?;
    let referred_into = props::referred_into(&blocks)?;
    let read_from = read::read_from(&blocks)?;
    let read_block_from = read::read_block_from(&blocks)?;
    let read_from_slice = read::read_from_slice(&blocks)?;
    let try_read_from = read::try_read_from(&blocks)?;
    let try_read_from_buffered = read::try_read_from_buffered(&blocks)?;
    let write_to = write::write_to(&blocks)?;
    let write_vectored_to = write::write_vectored_to(&blocks)?;
    Ok(quote! {
        #block
        #block_referred
        #prop
        #prop_referred
        #referred_into
        #read_from
        #read_block_from
        #read_from_slice
        #try_read_from
        #try_read_from_buffered
        #write_to
        #write_vectored_to
    })
}
