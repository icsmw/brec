use syn::{parse_quote, DeriveInput};

pub fn inject_repr_c(input: &mut DeriveInput) -> Result<(), syn::Error> {
    let mut has_repr_c = false;
    for attr in input.attrs.iter() {
        if attr.path().is_ident("repr") {
            attr.parse_nested_meta(|meta| {
                if let Some(ident) = meta.path.get_ident() {
                    has_repr_c = &ident.to_string() == "C";
                }
                Ok(())
            })?;
        }
    }

    if !has_repr_c {
        input.attrs.insert(0, parse_quote!(#[repr(C)]));
    }
    Ok(())
}
