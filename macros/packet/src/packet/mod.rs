mod modes;

use crate::*;
use crc32fast::Hasher;
use proc_macro2::TokenStream;
use syn::Ident;

#[derive(Debug)]
pub struct Packet {
    pub name: String,
    pub fields: Vec<Field>,
}

impl Packet {
    pub fn new(name: String, fields: Vec<Field>) -> Self {
        Self { name, fields }
    }
    pub fn from_input(input: &DeriveInput) -> Result<Self, syn::Error> {
        let name = &input.ident;
        if !input.generics.params.is_empty() {
            return Err(syn::Error::new_spanned(
                &input.generics,
                E::GenericTypesNotSupported,
            ));
        }
        let mut extracted = Vec::new();
        if let Data::Struct(data_struct) = &input.data {
            if let Fields::Named(fields) = &data_struct.fields {
                for field in &fields.named {
                    extracted.push(Field::try_from(field)?);
                }
            }
        }
        extracted.insert(
            0,
            Field::injected(FIELD_SIG, Ty::Slice(4, Box::new(Ty::u8))),
        );
        extracted.push(Field::injected(FIELD_CRC, Ty::u32));
        extracted.push(Field::injected(FIELD_NEXT, Ty::Slice(4, Box::new(Ty::u8))));
        Ok(Self::new(name.to_string(), extracted))
    }
    pub fn sig(&self) -> TokenStream {
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
        let sig = hasher.finalize().to_le_bytes();
        quote! { [#(#sig),*] }
    }
    fn const_sig_name(&self) -> Ident {
        format_ident!("{}", self.name.to_ascii_uppercase())
    }
    fn name(&self) -> Ident {
        format_ident!("{}", self.name)
    }
    fn referred_name(&self) -> Ident {
        format_ident!("{}Referred", self.name())
    }
}
