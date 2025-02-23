use proc_macro2::TokenStream;
use quote::quote;

use crate::*;

mod enums;
mod props;
mod read;
mod write;

pub fn gen(payloads: &[Payload]) -> Result<TokenStream, E> {
    let payload = enums::gen(payloads)?;
    let encode = props::encode(payloads)?;
    let encode_referred = props::encode_referred(payloads)?;
    let sig = props::sig(payloads)?;
    let crc = props::crc(payloads)?;
    let size = props::size(payloads)?;
    let extract_from = read::extract_from(payloads)?;
    let try_extract_from = read::try_extract_from(payloads)?;
    let try_extract_from_buffered = read::try_extract_from_buffered(payloads)?;
    let writing_to = write::writing_to(payloads)?;
    let writing_vectored_to = write::writing_vectored_to(payloads)?;
    Ok(quote! {
        #payload
        #encode
        #encode_referred
        #sig
        #crc
        #size
        #extract_from
        #try_extract_from
        #try_extract_from_buffered
        #writing_to
        #writing_vectored_to
    })
}
