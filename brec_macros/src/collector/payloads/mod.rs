use proc_macro2::TokenStream;
use quote::quote;

use crate::*;

mod enums;
mod napi;
mod props;
mod read;
mod write;

pub fn generate(payloads: Vec<&Payload>, cfg: &Config) -> Result<TokenStream, E> {
    let ordinary_payloads = payloads
        .iter()
        .copied()
        .filter(|p| !p.attrs.is_ctx())
        .collect::<Vec<_>>();
    let derives = Derives::common(ordinary_payloads.iter().map(|p| &p.derives).collect())?;
    let payload = enums::generate(&payloads, derives, cfg)?;
    let encode = props::encode(&ordinary_payloads)?;
    let encode_referred = props::encode_referred(&ordinary_payloads)?;
    let sig = props::sig(&ordinary_payloads)?;
    let crc = props::crc(&ordinary_payloads)?;
    let size = props::size(&ordinary_payloads)?;
    let extract_from = read::extract_from(&ordinary_payloads)?;
    let try_extract_from = read::try_extract_from(&ordinary_payloads)?;
    let try_extract_from_buffered = read::try_extract_from_buffered(&ordinary_payloads)?;
    let writing_to = write::writing_to(&ordinary_payloads)?;
    let writing_vectored_to = write::writing_vectored_to(&ordinary_payloads)?;
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
