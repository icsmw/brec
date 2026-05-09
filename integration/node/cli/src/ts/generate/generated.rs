use crate::*;
use std::fs;
use std::path::Path;

pub struct GeneratedFiles<'a> {
    model: &'a Model,
}

impl<'a> GeneratedFiles<'a> {
    pub fn new(model: &'a Model) -> Self {
        Self { model }
    }

    pub fn model(&self) -> &'a Model {
        self.model
    }

    fn render_module(&self, module: &dyn FormatterWritable) -> Result<String, Error> {
        let mut content = String::new();
        let mut tab = Tab::default();
        let mut writer = FormatterWriter::new(&mut content, &mut tab);
        module.write(&mut writer)?;
        Ok(content)
    }

    pub fn write_to(&self, out: &Path) -> Result<(), Error> {
        let blocks = BlocksFile::from(self.model);
        let payloads = PayloadFile::from(self.model);
        let packet = PacketFile::new(self.model, vec![&blocks, &payloads]);

        let modules: [(&str, &dyn FormatterWritable); 3] = [
            (BlocksFile::FILE_NAME, &blocks),
            (PayloadFile::FILE_NAME, &payloads),
            (PacketFile::FILE_NAME, &packet),
        ];

        for (file_name, module) in modules {
            fs::write(out.join(file_name), self.render_module(module)?)?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Model;
    use brec_scheme::Vis;
    use brec_scheme::{
        BlockTy, PayloadTy, SchemeBlock, SchemeBlockField, SchemeConfig, SchemeFieldType,
        SchemeFile, SchemePayload, SchemePayloadField, SchemePayloadVariant, SchemeType,
    };

    #[test]
    fn renders_expected_types_for_sample_scheme() {
        let scheme = sample_scheme();
        let model = Model::try_from(&scheme).expect("model");
        let generated = GeneratedFiles::new(&model);
        let blocks_file = BlocksFile::from(&model);
        let blocks = generated.render_module(&blocks_file).expect("render blocks");
        let payloads_file = PayloadFile::from(&model);
        let payloads = generated
            .render_module(&payloads_file)
            .expect("render payloads");
        let packet = generated
            .render_module(&PacketFile::new(&model, vec![&blocks_file, &payloads_file]))
            .expect("render packet");
        let index = generated
            .render_module(&IndexFile::new(&model, vec![&blocks_file, &payloads_file]))
            .expect("render index");

        assert!(blocks.contains("export interface BlockAlpha"));
        assert!(blocks.contains("export type Block ="));
        assert!(payloads.contains("export interface PayloadAlpha"));
        assert!(payloads.contains("field_str: string;"), "{payloads}");
        assert!(payloads.contains("field_nested: NestedStructCA;"), "{payloads}");
        assert!(payloads.contains("field_optional?: boolean;"), "{payloads}");
        assert!(payloads.contains("export interface NestedStructCA"), "{payloads}");
        assert!(payloads.contains("export type PayloadBeta ="), "{payloads}");
        assert!(payloads.contains("One: string;"), "{payloads}");
        assert!(payloads.contains("Two: [number, boolean];"), "{payloads}");
        assert!(payloads.contains("Three: null;"), "{payloads}");
        assert!(packet.contains("payload?: Payload;"), "{packet}");
        assert!(index.contains("export * from \"./payloads\";"), "{index}");
    }

    #[test]
    fn fails_when_named_type_is_not_included() {
        let mut scheme = sample_scheme();
        scheme.types.clear();

        let err = match Model::try_from(&scheme) {
            Ok(_) => panic!("expected missing included type error"),
            Err(err) => err,
        };
        let message = err.to_string();

        assert!(message.contains("#[payload(include)]"));
        assert!(message.contains("NestedStructCA"));
    }

    fn sample_scheme() -> SchemeFile {
        SchemeFile {
            version: "0.1.0".to_owned(),
            package: "sample".to_owned(),
            config: SchemeConfig {
                no_default_payloads: false,
                default_payloads: vec!["Bytes".to_owned(), "String".to_owned()],
            },
            blocks: vec![SchemeBlock {
                name: "BlockAlpha".to_owned(),
                fullname: "BlockAlpha".to_owned(),
                fullpath: "BlockAlpha".to_owned(),
                visibility: Vis::Public,
                no_crc: false,
                fields: vec![
                    SchemeBlockField {
                        name: "field_u32".to_owned(),
                        visibility: Vis::Public,
                        ty: SchemeFieldType::Block(BlockTy::U32),
                    },
                    SchemeBlockField {
                        name: "field_blob".to_owned(),
                        visibility: Vis::Public,
                        ty: SchemeFieldType::Block(BlockTy::Blob(4)),
                    },
                ],
            }],
            payloads: vec![
                SchemePayload {
                    name: "PayloadAlpha".to_owned(),
                    fullname: "PayloadAlpha".to_owned(),
                    fullpath: "PayloadAlpha".to_owned(),
                    is_ctx: false,
                    is_bincode: true,
                    is_crypt: false,
                    no_crc: false,
                    no_auto_crc: false,
                    no_default_sig: false,
                    hooks: false,
                    fields: vec![
                        SchemePayloadField {
                            name: Some("field_str".to_owned()),
                            visibility: Some(Vis::Public),
                            ty: SchemeFieldType::Payload(PayloadTy::String),
                        },
                        SchemePayloadField {
                            name: Some("field_nested".to_owned()),
                            visibility: Some(Vis::Public),
                            ty: SchemeFieldType::Payload(PayloadTy::Struct(
                                "NestedStructCA".to_owned(),
                            )),
                        },
                        SchemePayloadField {
                            name: Some("field_optional".to_owned()),
                            visibility: Some(Vis::Public),
                            ty: SchemeFieldType::Payload(PayloadTy::Option(Box::new(
                                PayloadTy::Bool,
                            ))),
                        },
                    ],
                    variants: Vec::new(),
                },
                SchemePayload {
                    name: "PayloadBeta".to_owned(),
                    fullname: "PayloadBeta".to_owned(),
                    fullpath: "PayloadBeta".to_owned(),
                    is_ctx: false,
                    is_bincode: true,
                    is_crypt: false,
                    no_crc: false,
                    no_auto_crc: false,
                    no_default_sig: false,
                    hooks: false,
                    fields: Vec::new(),
                    variants: vec![
                        SchemePayloadVariant {
                            name: "One".to_owned(),
                            fields: vec![SchemePayloadField {
                                name: None,
                                visibility: None,
                                ty: SchemeFieldType::Payload(PayloadTy::String),
                            }],
                        },
                        SchemePayloadVariant {
                            name: "Two".to_owned(),
                            fields: vec![
                                SchemePayloadField {
                                    name: None,
                                    visibility: None,
                                    ty: SchemeFieldType::Payload(PayloadTy::U32),
                                },
                                SchemePayloadField {
                                    name: None,
                                    visibility: None,
                                    ty: SchemeFieldType::Payload(PayloadTy::Bool),
                                },
                            ],
                        },
                        SchemePayloadVariant {
                            name: "Three".to_owned(),
                            fields: Vec::new(),
                        },
                    ],
                },
            ],
            types: vec![SchemeType {
                name: "NestedStructCA".to_owned(),
                fullname: "NestedStructCA".to_owned(),
                fullpath: "NestedStructCA".to_owned(),
                fields: vec![SchemePayloadField {
                    name: Some("value".to_owned()),
                    visibility: Some(Vis::Public),
                    ty: SchemeFieldType::Payload(PayloadTy::String),
                }],
                variants: Vec::new(),
            }],
        }
    }
}
