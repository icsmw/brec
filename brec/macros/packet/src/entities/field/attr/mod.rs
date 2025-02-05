use std::fmt;
use syn::Attribute;

#[enum_ids::enum_ids(display_variant_snake)]
#[derive(Debug, Clone)]
pub enum FieldAttr {
    LinkWith(String),
}

impl FieldAttr {
    pub fn has(attr: &Attribute) -> bool {
        attr.path().is_ident(&FieldAttrId::LinkWith.to_string())
    }
}

impl fmt::Display for FieldAttr {
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
