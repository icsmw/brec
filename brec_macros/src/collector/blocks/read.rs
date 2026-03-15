use crate::*;

use proc_macro2::TokenStream;
use quote::quote;

pub fn read_from(blocks: &[&Block]) -> Result<TokenStream, E> {
    let sig_len = BLOCK_SIG_LEN;
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
                let mut sig = [0u8; #sig_len];
                buf.read_exact(&mut sig)?;
                Err(brec::Error::SignatureDismatch(brec::Unrecognized::block(sig)))
            }
        }
    })
}

pub fn read_block_from(blocks: &[&Block]) -> Result<TokenStream, E> {
    let sig_len = BLOCK_SIG_LEN;
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
            let mut sig = [0u8; #sig_len];
            buf.read_exact(&mut sig)?;
            Err(brec::Error::SignatureDismatch(brec::Unrecognized::block(sig)))
        }
    } else {
        quote! {
            if skip_sig {
                // If signature skipping is enabled, concrete block readers do not perform
                // signature checks, so reaching this branch means the caller violated the
                // expected control flow for block dispatch.
                unreachable!("block dispatch cannot fail with skip_sig = true")
            }
            let mut sig = [0u8; #sig_len];
            buf.read_exact(&mut sig)?;
            Err(brec::Error::SignatureDismatch(brec::Unrecognized::block(sig)))
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
    let sig_len = BLOCK_SIG_LEN;
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
        quote! {
            let start_pos = buf.stream_position()?;
            let len = buf.seek(std::io::SeekFrom::End(0))? - start_pos;
            buf.seek(std::io::SeekFrom::Start(start_pos))?;
            if len < #sig_len {
                return Ok(brec::ReadStatus::NotEnoughData(#sig_len - len));
            }
            let mut sig = [0u8; #sig_len];
            buf.read_exact(&mut sig)?;
            buf.seek(std::io::SeekFrom::Start(start_pos))?;
            Err(brec::Error::SignatureDismatch(brec::Unrecognized::block(sig)))
        }
    } else {
        quote! {
            let mut sig = [0u8; #sig_len];
            buf.read_exact(&mut sig)?;
            buf.seek(std::io::SeekFrom::Current(-( #sig_len as i64 )))?;
            Err(brec::Error::SignatureDismatch(brec::Unrecognized::block(sig)))
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
    let sig_len = BLOCK_SIG_LEN;
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
    let tail = if blocks.is_empty() {
        quote! {
            let bytes = buf.fill_buf()?;
            if bytes.len() < #sig_len {
                return Ok(brec::ReadStatus::NotEnoughData((#sig_len - bytes.len()) as u64));
            }
            let sig = <[u8; #sig_len]>::try_from(&bytes[..#sig_len])?;
            Err(brec::Error::SignatureDismatch(brec::Unrecognized::block(sig)))
        }
    } else {
        quote! {
            let bytes = buf.fill_buf()?;
            let sig = <[u8; #sig_len]>::try_from(&bytes[..#sig_len])?;
            Err(brec::Error::SignatureDismatch(brec::Unrecognized::block(sig)))
        }
    };
    Ok(quote! {
        impl brec::TryReadFromBuffered for Block {
            fn try_read<T: std::io::BufRead>(buf: &mut T) -> Result<brec::ReadStatus<Self>, brec::Error> {
                #(#variants)*
                #tail
            }
        }
    })
}

pub fn read_from_slice(blocks: &[&Block]) -> Result<TokenStream, E> {
    let sig_len = BLOCK_SIG_LEN;
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
            if buf.len() < #sig_len {
                return Err(brec::Error::NotEnoughtSignatureData(buf.len(), #sig_len));
            }
            let sig = <[u8; #sig_len]>::try_from(&buf[..#sig_len])?;
            Err(brec::Error::SignatureDismatch(brec::Unrecognized::block(sig)))
        }
    } else {
        quote! {
            if skip_sig {
                // If signature skipping is enabled, concrete block readers do not perform
                // signature checks, so reaching this branch means the caller violated the
                // expected control flow for block dispatch.
                unreachable!("block dispatch cannot fail with skip_sig = true")
            }
            let sig = <[u8; #sig_len]>::try_from(&buf[..#sig_len])?;
            Err(brec::Error::SignatureDismatch(brec::Unrecognized::block(sig)))
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
