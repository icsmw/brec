use std::fmt;
use syn::Attribute;

#[derive(Debug, Default)]
pub struct BlockAttrs(pub Vec<BlockAttr>);

#[enum_ids::enum_ids(display_variant_snake)]
#[derive(Debug)]
pub enum BlockAttr {
    Path(String),
}

impl BlockAttr {
    pub fn has(attr: &Attribute) -> bool {
        attr.path().is_ident(&BlockAttrId::Path.to_string())
    }
}

impl fmt::Display for BlockAttr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Self::Path(path) => format!("{}({path})", self.id()),
            }
        )
    }
}
