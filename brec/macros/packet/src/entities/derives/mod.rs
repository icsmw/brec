use syn::DeriveInput;

#[derive(Debug, Clone)]
pub struct Derives {
    derives: Vec<String>,
}

impl Derives {
    pub fn common(derives: Vec<&Self>) -> Vec<String> {
        let mut common = derives
            .first()
            .map(|der| der.derives.clone())
            .unwrap_or_default();
        derives.iter().for_each(|v| {
            common.retain(|der| v.derives.contains(der));
        });
        common
    }
}

impl From<&DeriveInput> for Derives {
    fn from(input: &DeriveInput) -> Self {
        let mut derives = Vec::new();
        input.attrs.iter().for_each(|attr| {
            if attr.path().is_ident("derive") {
                let _ = attr.parse_nested_meta(|meta| {
                    // TODO: consider other than just ident variants. Could be a path, aka serder::Des.....
                    if let Some(ident) = meta.path.get_ident() {
                        derives.push(ident.to_string());
                    }
                    Ok(())
                });
            }
        });
        Self { derives }
    }
}
