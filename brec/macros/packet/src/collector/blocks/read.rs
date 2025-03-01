use crate::*;

use proc_macro2::TokenStream;
use quote::quote;

pub fn read_from(blocks: &[&Block]) -> Result<TokenStream, E> {
    let mut variants = Vec::new();
    for blk in blocks.iter() {
        let fullname = blk.fullname()?;
        let fullpath = blk.fullpath()?;
        variants.push(quote! {
            match <#fullpath as brec::ReadBlockFrom>::read(buf, false) {
                Ok(blk) => return Ok(Block::#fullname(blk)),
                Err(err) => {
                    if !matches!(err, brec::Error::SignatureDismatch) {
                        return Err(err);
                    }
                }
            }
        });
    }
    Ok(quote! {
        impl brec::ReadFrom for Block {
            fn read<T: std::io::Read>(buf: &mut T) -> Result<Self, brec::Error>
            where
                Self: Sized,
            {
                #(#variants)*
                Err(brec::Error::SignatureDismatch)
            }
        }
    })
}

pub fn read_block_from(blocks: &[&Block]) -> Result<TokenStream, E> {
    let mut variants = Vec::new();
    for blk in blocks.iter() {
        let fullname = blk.fullname()?;
        let fullpath = blk.fullpath()?;
        variants.push(quote! {
            match <#fullpath as brec::ReadBlockFrom>::read(buf, skip_sig) {
                Ok(blk) => return Ok(Block::#fullname(blk)),
                Err(err) => {
                    if !matches!(err, brec::Error::SignatureDismatch) {
                        return Err(err);
                    }
                }
            }
        });
    }
    Ok(quote! {
        impl brec::ReadBlockFrom for Block {
            fn read<T: std::io::Read>(buf: &mut T, skip_sig: bool) -> Result<Self, brec::Error>
            where
                Self: Sized,
            {
                #(#variants)*
                Err(brec::Error::SignatureDismatch)
            }
        }
    })
}

pub fn try_read_from(blocks: &[&Block]) -> Result<TokenStream, E> {
    let mut variants = Vec::new();
    for blk in blocks.iter() {
        let fullname = blk.fullname()?;
        let fullpath = blk.fullpath()?;
        variants.push(quote! {
            match <#fullpath as brec::TryReadFrom>::try_read(buf) {
                Ok(brec::ReadStatus::Success(blk)) => {
                    return Ok(brec::ReadStatus::Success(Block::#fullname(blk)))
                }
                Ok(brec::ReadStatus::NotEnoughData(needed)) => {
                    return Ok(brec::ReadStatus::NotEnoughData(needed))
                }
                Err(err) => {
                    if !matches!(err, brec::Error::SignatureDismatch) {
                        return Err(err);
                    }
                }
            }
        });
    }
    Ok(quote! {
        impl brec::TryReadFrom for Block {
            fn try_read<T: std::io::Read + std::io::Seek>(
                buf: &mut T,
            ) -> Result<brec::ReadStatus<Self>, brec::Error>
            where
                Self: Sized,
            {
                #(#variants)*
                Err(brec::Error::SignatureDismatch)
            }
        }
    })
}

pub fn try_read_from_buffered(blocks: &[&Block]) -> Result<TokenStream, E> {
    let mut variants = Vec::new();
    for blk in blocks.iter() {
        let fullname = blk.fullname()?;
        let fullpath = blk.fullpath()?;
        variants.push(quote! {
            match <#fullpath as brec::TryReadFromBuffered>::try_read(buf) {
                Ok(brec::ReadStatus::Success(blk)) => {
                    return Ok(brec::ReadStatus::Success(Block::#fullname(blk)))
                }
                Ok(brec::ReadStatus::NotEnoughData(needed)) => {
                    return Ok(brec::ReadStatus::NotEnoughData(needed))
                }
                Err(err) => {
                    if !matches!(err, brec::Error::SignatureDismatch) {
                        return Err(err);
                    }
                }
            }
        });
    }
    Ok(quote! {
        impl brec::TryReadFromBuffered for Block {
            fn try_read<T: std::io::BufRead>(buf: &mut T) -> Result<brec::ReadStatus<Self>, brec::Error>
            where
                Self: Sized,
            {
                #(#variants)*
                Err(brec::Error::SignatureDismatch)
            }
        }
    })
}

pub fn read_from_slice(blocks: &[&Block]) -> Result<TokenStream, E> {
    let mut variants = Vec::new();
    for blk in blocks.iter() {
        let referred_name = blk.referred_name();
        let fullname = blk.fullname()?;
        variants.push(quote! {
            match #referred_name::read_from_slice(buf, skip_sig) {
                Ok(blk) => return Ok(BlockReferred::#fullname(blk)),
                Err(err) => {
                    if !matches!(err, brec::Error::SignatureDismatch) {
                        return Err(err);
                    }
                }
            }
        });
    }
    Ok(quote! {
        impl<'a> brec::ReadBlockFromSlice for BlockReferred<'a> {
            fn read_from_slice<'b>(buf: &'b [u8], skip_sig: bool) -> Result<Self, brec::Error>
            where
                Self: 'b + Sized,
            {
                #(#variants)*
                Err(brec::Error::SignatureDismatch)
            }
        }
    })
}
