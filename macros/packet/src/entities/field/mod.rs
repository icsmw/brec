use crate::*;

pub const FIELD_SIG: &str = "__sig";
pub const FIELD_CRC: &str = "__crc";
pub const FIELD_NEXT: &str = "__next";

#[derive(Debug)]
pub struct Field {
    pub name: String,
    pub attrs: Vec<Attr>,
    pub ty: Ty,
    pub injected: bool,
}

impl Field {
    pub fn injected<S: AsRef<str>>(name: S, ty: Ty) -> Self {
        Self {
            name: name.as_ref().to_string(),
            attrs: Vec::new(),
            ty,
            injected: true,
        }
    }
    pub fn is_reserved_name<S: AsRef<str>>(name: S) -> bool {
        [FIELD_SIG, FIELD_CRC, FIELD_NEXT].contains(&name.as_ref())
    }
    pub fn size(&self) -> usize {
        self.ty.size()
    }
}
