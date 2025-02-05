use crate::*;
use proc_macro2::TokenStream;
use quote::quote;

impl Write for Block {
    fn gen(&self) -> Result<TokenStream, E> {
        let block_name = self.name();
        let mut write_pushes = Vec::new();
        let mut write_all_pushes = Vec::new();
        for field in self.fields.iter().filter(|f| !f.injected) {
            let as_bytes = field.to_bytes()?;
            if let Ty::Slice(.., inner_ty) = &field.ty {
                if matches!(**inner_ty, Ty::u8) {
                    write_pushes.push(quote! {
                        bytes += buf.write(#as_bytes)?;
                    });
                    write_all_pushes.push(quote! {
                        buf.write_all(#as_bytes)?;
                    });
                } else {
                    write_pushes.push(quote! {
                        let bts = #as_bytes;
                        bytes += buf.write(&bts)?;
                    });
                    write_all_pushes.push(quote! {
                        let bts = #as_bytes;
                        buf.write_all(&bts)?;
                    });
                }
            } else {
                write_pushes.push(quote! {
                    bytes += buf.write(#as_bytes)?;
                });
                write_all_pushes.push(quote! {
                    buf.write_all(#as_bytes)?;
                });
            };
        }
        let const_sig = self.const_sig_name();
        Ok(quote! {

            impl brec::Write for #block_name {

                fn write<T: std::io::Write>(&self, buf: &mut T) -> std::io::Result<usize> {
                    let mut bytes: usize = buf.write(&#const_sig)?;
                    #(#write_pushes)*
                    bytes += buf.write(&self.crc())?;
                    Ok(bytes)
                }

                fn write_all<T: std::io::Write>(&self, buf: &mut T) -> std::io::Result<()> {
                    buf.write_all(&#const_sig)?;
                    #(#write_all_pushes)*
                    buf.write_all(&self.crc())?;
                    Ok(())
                }

            }

        })
    }
}
