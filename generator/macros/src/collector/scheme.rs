use crate::*;
use brec_scheme::{
    SchemeBlock, SchemeBlockField, SchemeConfig, SchemeFieldType, SchemeFile, SchemePayload,
    SchemePayloadField, SchemePayloadVariant,
};
use std::{
    env, fs,
    path::{Path, PathBuf},
};

pub struct Scheme<'a> {
    collector: &'a Collector,
    cfg: &'a Config,
    package: String,
}

impl<'a> Scheme<'a> {
    pub fn generate(collector: &'a Collector, cfg: &'a Config) -> Result<(), E> {
        let scheme = Self {
            collector,
            cfg,
            package: get_pkg_name(),
        };
        scheme.write()
    }

    fn write(&self) -> Result<(), E> {
        let output = SchemeFile {
            version: 1,
            package: self.package.clone(),
            config: self.config(),
            blocks: self.blocks()?,
            payloads: self.payloads()?,
        };
        let content = serde_json::to_vec_pretty(&output).map_err(std::io::Error::other)?;
        let path = self.resolve_target_path()?;
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        fs::write(path, content)?;
        Ok(())
    }

    fn config(&self) -> SchemeConfig {
        let default_payloads = if self.cfg.is_no_default_payloads() {
            Vec::new()
        } else {
            vec!["Bytes".to_owned(), "String".to_owned()]
        };
        SchemeConfig {
            no_default_payloads: self.cfg.is_no_default_payloads(),
            default_payloads,
        }
    }

    fn blocks(&self) -> Result<Vec<SchemeBlock>, E> {
        let package = self.package.clone();
        let mut blocks = self
            .collector
            .blocks
            .get(&package)
            .into_iter()
            .flat_map(|blocks| blocks.values())
            .map(|block| {
                Ok(SchemeBlock {
                    name: block.name.clone(),
                    fullname: block.fullname()?.to_string(),
                    fullpath: block.fullpath()?.to_string(),
                    visibility: block.vis.clone(),
                    no_crc: block.attrs.is_no_crc(),
                    fields: block
                        .fields
                        .iter()
                        .filter(|field| !field.injected)
                        .map(Self::field_to_scheme)
                        .collect(),
                })
            })
            .collect::<Result<Vec<_>, E>>()?;
        blocks.sort_by(|a, b| a.fullname.cmp(&b.fullname));
        Ok(blocks)
    }

    fn payloads(&self) -> Result<Vec<SchemePayload>, E> {
        let package = self.package.clone();
        let mut payloads = self
            .collector
            .payloads
            .get(&package)
            .into_iter()
            .flat_map(|payloads| payloads.values())
            .map(|payload| {
                Ok(SchemePayload {
                    name: payload.name.clone(),
                    fullname: payload.fullname()?.to_string(),
                    fullpath: payload.fullpath()?.to_string(),
                    is_ctx: payload.attrs.is_ctx(),
                    is_bincode: payload.attrs.is_bincode(),
                    is_crypt: payload.attrs.is_crypt(),
                    no_crc: payload.attrs.is_no_crc(),
                    no_auto_crc: payload.attrs.is_no_auto_crc(),
                    no_default_sig: payload.attrs.no_default_sig(),
                    hooks: payload.attrs.hooks(),
                    fields: Self::payload_fields(&payload.kind),
                    variants: Self::payload_variants(&payload.kind),
                })
            })
            .collect::<Result<Vec<_>, E>>()?;
        payloads.sort_by(|a, b| a.fullname.cmp(&b.fullname));
        Ok(payloads)
    }

    fn field_to_scheme(field: &BlockField) -> SchemeBlockField {
        SchemeBlockField {
            name: field.name.clone(),
            visibility: field.vis.clone(),
            ty: Self::ty_to_scheme(&field.ty),
        }
    }

    fn payload_fields(kind: &PayloadKind) -> Vec<SchemePayloadField> {
        match kind {
            PayloadKind::Struct(fields) => Self::payload_fields_group(fields),
            PayloadKind::Enum(_) => Vec::new(),
        }
    }

    fn payload_variants(kind: &PayloadKind) -> Vec<SchemePayloadVariant> {
        match kind {
            PayloadKind::Struct(_) => Vec::new(),
            PayloadKind::Enum(variants) => variants
                .iter()
                .map(|variant| SchemePayloadVariant {
                    name: variant.name.clone(),
                    fields: Self::payload_fields_group(&variant.fields),
                })
                .collect(),
        }
    }

    fn payload_fields_group(fields: &PayloadFields) -> Vec<SchemePayloadField> {
        match fields {
            PayloadFields::Named(fields) => fields
                .iter()
                .map(|field| SchemePayloadField {
                    name: Some(field.name.clone()),
                    visibility: Some(field.vis.clone()),
                    ty: Self::payload_ty_to_scheme(&field.ty),
                })
                .collect(),
            PayloadFields::Unnamed(fields) => fields
                .iter()
                .map(|ty| SchemePayloadField {
                    name: None,
                    visibility: None,
                    ty: Self::payload_ty_to_scheme(ty),
                })
                .collect(),
            PayloadFields::Unit => Vec::new(),
        }
    }

    fn ty_to_scheme(ty: &BlockTy) -> SchemeFieldType {
        SchemeFieldType::Block(ty.clone())
    }

    fn payload_ty_to_scheme(ty: &PayloadTy) -> SchemeFieldType {
        SchemeFieldType::Payload(ty.clone())
    }

    fn resolve_target_path(&self) -> Result<PathBuf, E> {
        if let Ok(out_dir) = env::var("OUT_DIR") {
            let out_dir = PathBuf::from(out_dir);
            if let Some(target_dir) = self.find_target_dir(&out_dir) {
                return Ok(target_dir.join("brec.scheme.json"));
            }
            return Ok(out_dir.join("brec.scheme.json"));
        }
        if let Ok(target_dir) = env::var("CARGO_TARGET_DIR") {
            return Ok(PathBuf::from(target_dir).join("brec.scheme.json"));
        }
        Ok(PathBuf::from(env::var("CARGO_MANIFEST_DIR")?).join("target/brec.scheme.json"))
    }

    fn find_target_dir<'b>(&self, path: &'b Path) -> Option<&'b Path> {
        path.ancestors()
            .find(|ancestor| ancestor.file_name().is_some_and(|name| name == "target"))
    }
}
