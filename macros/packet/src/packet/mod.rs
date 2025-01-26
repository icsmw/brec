use crate::*;

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
}
