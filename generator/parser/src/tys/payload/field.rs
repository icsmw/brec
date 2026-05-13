use crate::*;

#[derive(Debug, Clone)]
pub struct PayloadField {
    pub name: String,
    pub ty: PayloadTy,
    pub vis: Vis,
}
