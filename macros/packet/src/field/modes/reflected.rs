use crate::*;
use proc_macro2::TokenStream;

impl ReflectedMode for Field {
    fn generate(&self) -> TokenStream {
        let fname = format_ident!("{}", self.name);
        let ty = self.ty.r#static();
        quote! {
            let #fname = unsafe { &*(data.as_ptr() as *const #ty) };
        }
    }
}
