use std::fmt;
use syn::Attribute;

#[enum_ids::enum_ids(display_variant_snake)]
#[derive(Debug)]
pub enum Attr {
    LinkWith(String),
}

impl Attr {
    pub fn has(attr: &Attribute) -> bool {
        attr.path().is_ident(&AttrId::LinkWith.to_string())
    }
}

impl fmt::Display for Attr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Self::LinkWith(target) => format!("{}({target})", self.id()),
            }
        )
    }
}
