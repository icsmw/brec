use crate::*;

use proc_macro2::TokenStream;
use quote::quote;

pub fn write_to(blocks: &[Block]) -> Result<TokenStream, E> {
    let mut write = Vec::new();
    let mut write_all = Vec::new();
    for blk in blocks.iter() {
        let fullname = blk.fullname()?;
        write.push(quote! {
            Block::#fullname(blk) => blk.write(buf),
        });
        write_all.push(quote! {
            Block::#fullname(blk) => blk.write_all(buf),
        });
    }
    Ok(quote! {
        impl brec::WriteTo for Block {
            fn write<T: std::io::Write>(&self, buf: &mut T) -> std::io::Result<usize> {
                match self {
                    #(#write,)*
                }
            }
            fn write_all<T: std::io::Write>(&self, buf: &mut T) -> std::io::Result<()> {
                match self {
                    #(#write_all,)*
                }
            }
        }
    })
}

pub fn write_vectored_to(blocks: &[Block]) -> Result<TokenStream, E> {
    let mut slices = Vec::new();
    for blk in blocks.iter() {
        let fullname = blk.fullname()?;
        slices.push(quote! {
            Block::#fullname(blk) => blk.slices(buf),
        });
    }
    Ok(quote! {
        impl brec::WriteVectoredTo for Block {
            fn slices(&self) -> std::io::Result<brec::IoSlices> {
                match self {
                    #(#slices,)*
                }
            }
        }
    })
}
