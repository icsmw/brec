use crate::*;

use brec_consts::*;
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
                    if !matches!(err, brec::Error::SignatureDismatch(_)) {
                        return Err(err);
                    }
                }
            }
        });
    }
    Ok(quote! {
        impl brec::ReadFrom for Block {
            fn read<T: std::io::Read>(buf: &mut T) -> Result<Self, brec::Error> {
                #(#variants)*
                Err(brec::Error::SignatureDismatch(
                    brec::Unrecognized::block_from(buf)?
                ))
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
                    if !matches!(err, brec::Error::SignatureDismatch(_)) {
                        return Err(err);
                    }
                }
            }
        });
    }
    let tail = if blocks.is_empty() {
        quote! {
            if skip_sig {
                return Err(brec::Error::SignatureDismatch(brec::Unrecognized::default()));
            }
            Err(brec::Error::SignatureDismatch(
                brec::Unrecognized::block_from(buf)?
            ))
        }
    } else {
        quote! {
            if skip_sig {
                // If signature skipping is enabled, concrete block readers do not perform
                // signature checks, so reaching this branch means the caller violated the
                // expected control flow for block dispatch.
                unreachable!("block dispatch cannot fail with skip_sig = true")
            }
            Err(brec::Error::SignatureDismatch(
                brec::Unrecognized::block_from(buf)?
            ))
        }
    };
    Ok(quote! {
        impl brec::ReadBlockFrom for Block {
            fn read<T: std::io::Read>(buf: &mut T, skip_sig: bool) -> Result<Self, brec::Error>
            where
                Self: Sized,
            {
                #(#variants)*
                #tail
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
                    if !matches!(err, brec::Error::SignatureDismatch(_)) {
                        return Err(err);
                    }
                }
            }
        });
    }
    let tail = if blocks.is_empty() {
        let required_sig = if cfg!(feature = "resilient") {
            BLOCK_SIG_LEN + BLOCK_SIZE_FIELD_LEN
        } else {
            BLOCK_SIG_LEN
        };
        quote! {
            let start_pos = buf.stream_position()?;
            let len = buf.seek(std::io::SeekFrom::End(0))? - start_pos;
            buf.seek(std::io::SeekFrom::Start(start_pos))?;
            if len < #required_sig {
                return Ok(brec::ReadStatus::NotEnoughData(#required_sig - len));
            }
            let unrecognized = brec::Unrecognized::block_from(buf)?;
            buf.seek(std::io::SeekFrom::Start(start_pos))?;
            Err(brec::Error::SignatureDismatch(unrecognized))
        }
    } else {
        quote! {
            let start_pos = buf.stream_position()?;
            let result = brec::Unrecognized::block_from(buf);
            buf.seek(std::io::SeekFrom::Start(start_pos))?;
            result
                .map(|unrecognized| brec::Error::SignatureDismatch(unrecognized))
                .map_or_else(
                    |err| err.into_read_status::<Self>(),
                    Err
                )
        }
    };
    Ok(quote! {
        impl brec::TryReadFrom for Block {
            fn try_read<T: std::io::Read + std::io::Seek>(buf: &mut T) -> Result<brec::ReadStatus<Self>, brec::Error> {
                #(#variants)*
                #tail
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
                    if !matches!(err, brec::Error::SignatureDismatch(_)) {
                        return Err(err);
                    }
                }
            }
        });
    }

    Ok(quote! {
        impl brec::TryReadFromBuffered for Block {
            fn try_read<T: std::io::BufRead>(buf: &mut T) -> Result<brec::ReadStatus<Self>, brec::Error> {
                #(#variants)*

                brec::Unrecognized::block_from_buffer(buf).map_or_else(
                    |err| err.into_read_status::<Self>(),
                    |unrecognized| Err(brec::Error::SignatureDismatch(unrecognized))
                )
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
                    if !matches!(err, brec::Error::SignatureDismatch(_)) {
                        return Err(err);
                    }
                }
            }
        });
    }
    let tail = if blocks.is_empty() {
        quote! {
            if skip_sig {
                return Err(brec::Error::SignatureDismatch(brec::Unrecognized::default()));
            }
            Err(brec::Error::SignatureDismatch(
                brec::Unrecognized::block_from_slice(buf)?
            ))
        }
    } else {
        quote! {
            if skip_sig {
                // If signature skipping is enabled, concrete block readers do not perform
                // signature checks, so reaching this branch means the caller violated the
                // expected control flow for block dispatch.
                unreachable!("block dispatch cannot fail with skip_sig = true")
            }
            Err(brec::Error::SignatureDismatch(
                brec::Unrecognized::block_from_slice(buf)?
            ))
        }
    };
    Ok(quote! {
        impl<'a> brec::ReadBlockFromSlice for BlockReferred<'a> {
            fn read_from_slice<'b>(buf: &'b [u8], skip_sig: bool) -> Result<Self, brec::Error>
            where
                Self: 'b + Sized,
            {
                #(#variants)*
                #tail
            }
        }
    })
}
