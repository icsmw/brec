use crate::*;
use proc_macro2::TokenStream;

impl ReflectedMode for Packet {
    fn generate(&self) -> TokenStream {
        let referred = self.referred();
        let stat = self.r#static();
        let referred_name = self.referred_name();
        let packet_name = self.packet_name();
        let const_sig = self.const_sig_name();
        let mut fields = Vec::new();
        let mut fnames = Vec::new();
        let mut offset = 0usize;
        let sig: TokenStream = self.sig();
        for field in self.fields.iter() {
            fields.push(field.downcast_as_ref(offset));
            fnames.push(format_ident!("{}", field.name));
            offset += field.ty.size();
        }
        quote! {
            #stat

            #referred

            const #const_sig: [u8; 4] = #sig;

            impl<'a> brec::Packet<#referred_name <'a>> for #referred_name <'a> {

                fn sig() -> &'static [u8; 4] {
                    &#const_sig
                }

                fn read(data: &[u8]) -> Result<Option<#referred_name<'a>>, brec::Error> {
                    use std::mem;

                    if data.len() < 4 {
                        return Err(brec::Error::NotEnoughtSignatureData(data.len(), 4));
                    }

                    if data[..4] != #const_sig {
                        return Ok(None);
                    }

                    if data.len() < mem::size_of::<#packet_name>() {
                        return Err(brec::Error::NotEnoughtData(data.len(), mem::size_of::<#packet_name>()));
                    }

                    if data.as_ptr() as usize % std::mem::align_of::<#packet_name>() != 0 {
                        return Err(brec::Error::InvalidAlign(
                            data.as_ptr() as usize,
                            mem::size_of::<#packet_name>(),
                            data.as_ptr() as usize % std::mem::align_of::<#packet_name>()
                        ));
                    }

                    #(#fields)*

                    Ok(Some(#referred_name {
                        #(#fnames,)*
                    }))
                }
            }
            #stat
        }
    }
}
