mod def;
mod referred;
mod statics;
#[cfg(test)]
mod tests;

use std::{convert::TryFrom, fmt};
use syn::Type;

use crate::*;

pub(crate) use def::*;

#[derive(Debug, PartialEq)]
pub struct Ty {
    pub referred: bool,
    pub def: TyDef,
}

impl Ty {
    pub fn new(def: TyDef, referred: bool) -> Self {
        Self { referred, def }
    }
    pub fn size(&self) -> usize {
        self.def.size()
    }
}

impl fmt::Display for Ty {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}{}", if self.referred { "&" } else { "" }, self.def)
    }
}

impl TryFrom<&Type> for Ty {
    type Error = syn::Error;

    fn try_from(ty: &Type) -> Result<Self, Self::Error> {
        match ty {
            Type::Path(ty) => Ok(Self::new(TyDef::try_as_primitive(ty)?, false)),
            Type::Array(ty) => Ok(Self::new(TyDef::try_as_array(ty)?, false)),
            Type::Reference(ty_ref) => match ty_ref.elem.as_ref() {
                Type::Path(ty) => Ok(Self::new(TyDef::try_as_primitive(ty)?, true)),
                Type::Array(ty) => Ok(Self::new(TyDef::try_as_array(ty)?, true)),
                _ => Err(syn::Error::new_spanned(ty_ref, E::UnsupportedType)),
            },
            _ => Err(syn::Error::new_spanned(ty, E::UnsupportedType)),
        }
    }
}
