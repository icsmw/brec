use crate::*;
use proc_macro2::TokenStream;
use quote::{format_ident, quote};

impl Write for Block {
    fn gen(&self) -> Result<TokenStream, E> {
        let block_name = self.name();
        let mut buf_fillers = Vec::new();
        for field in self.fields.iter().filter(|f| !f.injected) {
            let size = field.size();
            match field.ty {
                Ty::u8
                | Ty::u16
                | Ty::u32
                | Ty::u64
                | Ty::u128
                | Ty::i8
                | Ty::i16
                | Ty::i32
                | Ty::i64
                | Ty::i128
                | Ty::f32
                | Ty::f64
                | Ty::bool
                | Ty::linkedToU8(..) => {
                    let as_bytes = field.to_bytes(true)?;
                    buf_fillers.push(quote! {
                        buffer[offset..offset + #size].copy_from_slice(#as_bytes);
                        offset += #size;
                    });
                }
                Ty::blob(..) => {
                    let name = format_ident!("{}", field.name);
                    buf_fillers.push(quote! {
                        unsafe {
                            let dst = buffer.as_mut_ptr().add(offset);
                            let src = self.#name.as_ptr();
                            std::ptr::copy_nonoverlapping(src, dst, #size);
                        }
                        offset += #size;
                    });
                }
            }
        }
        let const_sig = self.const_sig_name();
        let size = self.size();
        Ok(quote! {

            impl brec::WriteTo for #block_name {

                fn write<T: std::io::Write>(&self, writer: &mut T) -> std::io::Result<usize> {
                    use brec::prelude::*;
                    let mut buffer = [0u8; #size];
                    let mut offset = 0;
                    let crc = self.crc();
                    buffer[offset..offset + #SIG_LEN].copy_from_slice(&#const_sig);
                    offset += #SIG_LEN;
                    #(#buf_fillers)*
                    unsafe {
                        let dst = buffer.as_mut_ptr().add(offset);
                        let src = crc.as_ptr();
                        std::ptr::copy_nonoverlapping(src, dst, #CRC_LEN);
                    }
                    writer.write(&buffer)
                }

                fn write_all<T: std::io::Write>(&self, writer: &mut T) -> std::io::Result<()> {
                    use brec::prelude::*;
                    let mut buffer = [0u8; #size];
                    let mut offset = 0;
                    let crc = self.crc();
                    buffer[offset..offset + #SIG_LEN].copy_from_slice(&#const_sig);
                    offset += #SIG_LEN;
                    #(#buf_fillers)*
                    unsafe {
                        let dst = buffer.as_mut_ptr().add(offset);
                        let src = crc.as_ptr();
                        std::ptr::copy_nonoverlapping(src, dst, #CRC_LEN);
                    }
                    writer.write_all(&buffer)
                }

            }

        })
    }
}

impl WriteVectored for Block {
    fn gen(&self) -> Result<TokenStream, E> {
        let block_name = self.name();
        let mut groups = Vec::new();
        let mut slices = Vec::new();
        let mut fields = Vec::new();
        for field in self.fields.iter().filter(|f| !f.injected) {
            match field.ty {
                Ty::u8
                | Ty::u16
                | Ty::u32
                | Ty::u64
                | Ty::u128
                | Ty::i8
                | Ty::i16
                | Ty::i32
                | Ty::i64
                | Ty::i128
                | Ty::f32
                | Ty::f64
                | Ty::bool
                | Ty::linkedToU8(..) => {
                    if !slices.is_empty() {
                        groups.push(slices);
                        slices = Vec::new();
                    }
                    fields.push(field);
                }
                Ty::blob(..) => {
                    if !fields.is_empty() {
                        groups.push(fields);
                        fields = Vec::new();
                    }
                    slices.push(field);
                }
            }
        }
        if !fields.is_empty() {
            groups.push(fields);
        }
        if !slices.is_empty() {
            groups.push(slices);
        }
        let mut fragments = Vec::new();
        for group in groups.into_iter() {
            let len: usize = group.iter().map(|f| f.size()).sum();
            let last = group.len() - 1;
            for (n, field) in group.into_iter().enumerate() {
                let size = field.size();
                match field.ty {
                    Ty::u8
                    | Ty::u16
                    | Ty::u32
                    | Ty::u64
                    | Ty::u128
                    | Ty::i8
                    | Ty::i16
                    | Ty::i32
                    | Ty::i64
                    | Ty::i128
                    | Ty::f32
                    | Ty::f64
                    | Ty::bool
                    | Ty::linkedToU8(..) => {
                        if n == 0 {
                            fragments.push(quote! {
                                let mut buffer = [0u8; #len];
                                let mut offset = 0;
                            });
                        }
                        let as_bytes = field.to_bytes(true)?;
                        fragments.push(quote! {
                            buffer[offset..offset + #size].copy_from_slice(#as_bytes);
                        });
                        if n != last {
                            fragments.push(quote! {
                                offset += #size;
                            });
                        } else {
                            fragments.push(quote! {
                                slices.add_buffered(buffer.to_vec());
                            });
                        }
                    }
                    Ty::blob(..) => {
                        let name = format_ident!("{}", field.name);
                        fragments.push(quote! {
                            slices.add_slice(&self.#name);
                        });
                    }
                }
            }
        }

        let const_sig = self.const_sig_name();
        Ok(quote! {

            impl brec::WriteVectoredTo for #block_name {

                fn slices(&self) -> std::io::Result<brec::IoSlices> {
                    use brec::prelude::*;
                    let mut slices = brec::IoSlices::default();
                    slices.add_buffered(#const_sig.to_vec());
                    #(#fragments)*
                    slices.add_buffered(self.crc().to_vec());
                    Ok(slices)
                }

            }

        })
    }
}
