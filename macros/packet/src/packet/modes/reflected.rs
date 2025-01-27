use crate::*;
use proc_macro2::TokenStream;

impl ReflectedMode for Packet {
    fn generate(&self) -> TokenStream {
        let referred = self.referred();
        let stat = self.r#static();
        let referred_name = format_ident!("{}", self.referred_name());
        quote! {
            #referred

            impl Read for #referred_name {
                fn read() -> Result<Option<#referred_name>, String> {

                }
            }
            #stat
        }
    }
}
