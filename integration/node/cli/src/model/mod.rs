mod block;
mod names;
mod payload_union;
mod resolver;
mod type_def;

pub use block::{Block, BlockUnion};
pub use payload_union::PayloadUnion;
pub use type_def::TypeDef;

use crate::Error;
use brec_scheme::SchemeFile;
use names::TypeNames;
use resolver::Resolver;

pub struct Model {
    pub version: String,
    pub package: String,
    pub blocks: Vec<Block>,
    pub block_union: BlockUnion,
    pub included_types: Vec<TypeDef>,
    pub payloads: Vec<TypeDef>,
    pub payload_union: PayloadUnion,
}

impl TryFrom<&SchemeFile> for Model {
    type Error = Error;

    fn try_from(scheme: &SchemeFile) -> Result<Self, Self::Error> {
        let names = TypeNames::try_from(scheme)?;
        let resolver = Resolver::new(&names);
        let blocks = scheme
            .blocks
            .iter()
            .map(Block::try_from)
            .collect::<Result<Vec<_>, _>>()?;

        Ok(Self {
            version: scheme.version.clone(),
            package: scheme.package.clone(),
            block_union: BlockUnion::from_blocks(&blocks)?,
            blocks,
            included_types: scheme
                .types
                .iter()
                .map(|ty| TypeDef::from_scheme_type(ty, &resolver))
                .collect::<Result<Vec<_>, _>>()?,
            payloads: scheme
                .payloads
                .iter()
                .map(|payload| TypeDef::from_payload(payload, &resolver))
                .collect::<Result<Vec<_>, _>>()?,
            payload_union: PayloadUnion::from_scheme(scheme)?,
        })
    }
}
