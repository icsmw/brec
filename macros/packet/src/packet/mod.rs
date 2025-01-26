mod gen;
mod reflected;
mod structured;

use crate::*;
use crc32fast::Hasher;

#[derive(Debug)]
pub struct Packet {
    pub name: String,
    pub fields: Vec<Field>,
}

impl Packet {
    pub fn new(name: String, fields: Vec<Field>) -> Self {
        Self { name, fields }
    }
    pub fn try_from_input(input: &DeriveInput) -> Result<Self, syn::Error> {
        let name = &input.ident;
        let mut extracted = Vec::new();
        if let Data::Struct(data_struct) = &input.data {
            if let Fields::Named(fields) = &data_struct.fields {
                for field in &fields.named {
                    extracted.push(Field::try_from(field)?);
                }
            }
        }
        Ok(Self::new(name.to_string(), extracted))
    }
    pub fn sig(&self) -> [u8; 4] {
        let mut hasher = Hasher::new();
        let snap = format!(
            "{};{}",
            self.name,
            self.fields
                .iter()
                .map(|f| format!("{}:{}", f.name, f.ty))
                .collect::<Vec<String>>()
                .join(";")
        );
        hasher.update(snap.as_bytes());
        hasher.finalize().to_le_bytes()
    }
}

impl Names for Packet {
    fn origin_name(&self) -> String {
        self.name.clone()
    }
    fn packet_name(&self) -> String {
        format!("{}Packet", self.name)
    }
}
