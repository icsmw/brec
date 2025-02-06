use crate::*;
use proc_macro2::TokenStream;
use quote::{format_ident, quote};

impl Write for Block {
    fn gen(&self) -> Result<TokenStream, E> {
        let block_name = self.name();
        let mut buf_fillers = Vec::new();
        for field in self.fields.iter().filter(|f| !f.injected) {
            let size = field.size();
            let as_bytes = field.to_bytes(true)?;
            buf_fillers.push(quote! {
                buffer[offset..offset + #size].copy_from_slice(#as_bytes);
                offset += #size;
            });
        }
        let const_sig = self.const_sig_name();
        let size = self.size();
        Ok(quote! {

            impl brec::Write for #block_name {

                fn write<T: std::io::Write>(&self, writer: &mut T) -> std::io::Result<usize> {
                    let mut buffer = [0u8; #size];
                    let mut offset = 0;
                    buffer[offset..offset + #SIG_LEN].copy_from_slice(&#const_sig);
                    offset += #SIG_LEN;
                    #(#buf_fillers)*
                    buffer[offset..offset + #CRC_LEN].copy_from_slice(&self.crc());
                    writer.write(&buffer)
                }

                fn write_all<T: std::io::Write>(&self, writer: &mut T) -> std::io::Result<()> {
                    let mut buffer = [0u8; #size];
                    let mut offset = 0;
                    buffer[offset..offset + #SIG_LEN].copy_from_slice(&#const_sig);
                    offset += #SIG_LEN;
                    #(#buf_fillers)*
                    buffer[offset..offset + #CRC_LEN].copy_from_slice(&self.crc());
                    writer.write_all(&buffer)
                }

            }

        })
    }
}
impl WriteOwned for Block {
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
                | Ty::bool => {
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

            impl brec::WriteOwned for #block_name {

                fn write<T: std::io::Write>(self, writer: &mut T) -> std::io::Result<usize> {
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
                    buffer[offset..offset + #CRC_LEN].copy_from_slice(&crc);
                    writer.write(&buffer)
                }

                fn write_all<T: std::io::Write>(self, writer: &mut T) -> std::io::Result<()> {
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
