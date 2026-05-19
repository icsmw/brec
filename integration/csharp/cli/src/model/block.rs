use super::CSharpType;
use super::field::{FieldDef, write_class};
use super::ident::{csharp_property_name, csharp_type_name};
use crate::{Error, SourceWritable};
use brec_scheme::{SchemeBlock, SchemeFieldType};

pub struct BlockDef {
    pub key: String,
    pub name: String,
    pub fields: Vec<FieldDef>,
}

impl TryFrom<&SchemeBlock> for BlockDef {
    type Error = Error;

    fn try_from(block: &SchemeBlock) -> Result<Self, Self::Error> {
        Ok(Self {
            key: block.fullname.clone(),
            name: csharp_type_name(&block.fullname),
            fields: block
                .fields
                .iter()
                .map(|field| {
                    let SchemeFieldType::Block(ty) = &field.ty else {
                        return Err(Error::InvalidScheme(format!(
                            "block {} contains non-block field type: {:?}",
                            block.fullname, field.ty
                        )));
                    };
                    Ok(FieldDef {
                        key: field.name.clone(),
                        name: csharp_property_name(&field.name),
                        ty: CSharpType::from_block_ty(ty),
                        nullable: false,
                    })
                })
                .collect::<Result<Vec<_>, _>>()?,
        })
    }
}

impl SourceWritable for BlockDef {
    fn write(&self, writer: &mut crate::SourceWriter) -> Result<(), Error> {
        write_class(writer, &self.name, Some("Block"), &self.fields)
    }
}
