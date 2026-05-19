use crate::*;
use std::marker::PhantomData;

/// Generated encode/decode functions for Brec blocks.
pub struct ApiBlock;
/// Generated encode/decode functions for Brec payloads.
pub struct ApiPayload;
/// Generated encode/decode functions for full Brec packets.
pub struct ApiPacket;

/// API surface emitted by the generator.
pub trait Api: RustWritable {}

impl<T: RustWritable> Api for T {}

/// A generated entry file parameterized by its output file kind.
pub struct ApiFile<'a, F: FileName> {
    apis: Vec<Box<dyn Api + 'a>>,
    file: PhantomData<F>,
    pub model: &'a Model,
}

impl<'a, F: FileName> ApiFile<'a, F> {
    pub fn new(model: &'a Model, apis: Vec<Box<dyn Api + 'a>>) -> Self {
        Self {
            model,
            apis,
            file: PhantomData,
        }
    }

    pub fn apis(&self) -> &[Box<dyn Api + 'a>] {
        &self.apis
    }

    pub fn file_name(&self) -> &'static str {
        F::FILE_NAME
    }
}

/// Writes generated Rust glue for one API function group.
pub trait RustWritable {
    fn write_rust(&self, writer: &mut SourceWriter) -> Result<(), Error>;
}

/// Shared method names for a generated encode/decode API pair.
pub trait ApiMethods {
    const ENCODE_METHOD_NAME: &'static str;
    const DECODE_METHOD_NAME: &'static str;
    fn snake_case_encode_method_name() -> &'static str {
        Self::ENCODE_METHOD_NAME
    }
    fn snake_case_decode_method_name() -> &'static str {
        Self::DECODE_METHOD_NAME
    }
}

impl ApiMethods for ApiBlock {
    const ENCODE_METHOD_NAME: &'static str = "encode_block";
    const DECODE_METHOD_NAME: &'static str = "decode_block";
}

impl ApiMethods for ApiPayload {
    const ENCODE_METHOD_NAME: &'static str = "encode_payload";
    const DECODE_METHOD_NAME: &'static str = "decode_payload";
}

impl ApiMethods for ApiPacket {
    const ENCODE_METHOD_NAME: &'static str = "encode_packet";
    const DECODE_METHOD_NAME: &'static str = "decode_packet";
}
