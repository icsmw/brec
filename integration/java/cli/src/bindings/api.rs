use crate::*;
use std::marker::PhantomData;

/// Generated encode/decode functions for Brec blocks.
pub struct ApiBlock;
/// Generated encode/decode functions for Brec payloads.
pub struct ApiPayload;
/// Generated encode/decode functions for full Brec packets.
pub struct ApiPacket;

/// API surface emitted to Rust JNI bindings.
///
/// Each API object knows how to render its native JNI function.
pub trait Api: RustWritable {}

impl<T: RustWritable> Api for T {}

/// A generated entry file parameterized by its output file kind.
///
/// `ApiFile` keeps the API set grouped while the Rust binding file renders it.
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

/// Writes JNI Rust glue for one API function group.
pub trait RustWritable {
    fn write_rust(&self, writer: &mut SourceWriter) -> Result<(), Error>;
}
