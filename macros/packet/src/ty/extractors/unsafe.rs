use proc_macro2::TokenStream;
use syn::Ident;

use crate::*;

impl Unsafe for Ty {
    fn unsafe_extr(&self, src: &Ident, offset: usize) -> TokenStream {
        let ty = self.r#static();
        if offset == 0 {
            if matches!(self, Ty::u8) {
                quote! {
                    unsafe { &*#src.as_ptr() }
                }
            } else {
                quote! {
                    unsafe { &*(#src.as_ptr() as *const #ty) }
                }
            }
        } else if matches!(self, Ty::u8) {
            quote! {
                unsafe { &*#src.as_ptr().add(#offset) }
            }
        } else {
            quote! {
               unsafe { &*(#src.as_ptr().add(#offset) as *const #ty) }
            }
        }
    }
}
