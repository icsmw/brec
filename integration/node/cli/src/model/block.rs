use crate::*;
use brec_scheme::{SchemeBlock, SchemeFieldType};
use std::fmt;

pub struct Block {
    name: String,
    fields: Vec<Field>,
}

impl Block {
    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn interface(&self) -> Interface {
        Interface::new(self.name.clone(), self.fields.clone())
    }

    fn union_variant(&self) -> Result<Type, Error> {
        Ok(Type::object(vec![Field::required(
            self.name(),
            Type::Named(self.name.clone()),
        )?]))
    }
}

pub struct BlockUnion(Vec<Type>);

impl BlockUnion {
    pub fn from_blocks(blocks: &[Block]) -> Result<Self, Error> {
        Ok(Self(
            blocks
                .iter()
                .map(Block::union_variant)
                .collect::<Result<Vec<_>, _>>()?,
        ))
    }
}

impl FormatterWritable for BlockUnion {
    fn write(&self, writer: &mut FormatterWriter) -> fmt::Result {
        if self.0.is_empty() {
            return writer.ln("export type Block = never;");
        }
        writer.write("export type Block =")?;
        for (idx, item) in self.0.iter().enumerate() {
            let sep = if idx + 1 == self.0.len() { ";" } else { " |" };
            item.write(writer)?;
            writer.write(sep)?;
        }
        writer.ln("")
    }
}

impl TryFrom<&SchemeBlock> for Block {
    type Error = Error;

    fn try_from(block: &SchemeBlock) -> Result<Self, Self::Error> {
        Ok(Self {
            name: block.fullname.clone(),
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
                    Field::required(&field.name, Type::from(ty))
                })
                .collect::<Result<Vec<_>, _>>()?,
        })
    }
}
