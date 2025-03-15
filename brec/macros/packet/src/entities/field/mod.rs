use proc_macro2::TokenStream;

use crate::*;

pub(crate) const FIELD_SIG: &str = "__sig";
pub(crate) const FIELD_CRC: &str = "__crc";

#[derive(Debug, Clone)]
pub struct Field {
    pub name: String,
    pub ty: Ty,
    pub injected: bool,
    pub vis: Vis,
}

impl Field {
    pub fn injected<S: AsRef<str>>(name: S, ty: Ty) -> Self {
        Self {
            name: name.as_ref().to_string(),
            ty,
            injected: true,
            vis: Vis::default(),
        }
    }
    pub fn is_reserved_name<S: AsRef<str>>(name: S) -> bool {
        [FIELD_SIG, FIELD_CRC].contains(&name.as_ref())
    }
    pub fn size(&self) -> usize {
        self.ty.size()
    }
    pub fn vis_token(&self) -> Result<TokenStream, E> {
        self.vis.as_token()
    }
}
