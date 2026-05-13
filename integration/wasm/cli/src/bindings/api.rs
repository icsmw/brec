use crate::*;
use std::marker::PhantomData;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WasmTarget {
    Node,
    Browser,
}

impl WasmTarget {
    pub fn parse(raw: &str) -> Result<Self, Error> {
        match raw {
            "node" => Ok(Self::Node),
            "browser" => Ok(Self::Browser),
            other => Err(Error::Cli(format!(
                "invalid --target value: {other}; expected node or browser"
            ))),
        }
    }

    pub fn wasm_pack_target(self) -> &'static str {
        match self {
            Self::Node => "nodejs",
            Self::Browser => "web",
        }
    }
}

impl std::fmt::Display for WasmTarget {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Node => write!(f, "node"),
            Self::Browser => write!(f, "browser"),
        }
    }
}

/// Generated encode/decode functions for Brec blocks.
pub struct ApiBlock;
/// Generated encode/decode functions for Brec payloads.
pub struct ApiPayload;
/// Generated encode/decode functions for full Brec packets.
pub struct ApiPacket;

/// API surface emitted to both Rust and TypeScript.
///
/// Each API object knows how to render its wasm-bindgen function and the matching
/// TypeScript wrapper, which keeps method names and signatures in one place.
pub trait Api: TsNodeWritable + TsBrowserWritable + RustWritable {}

impl<T: TsNodeWritable + TsBrowserWritable + RustWritable> Api for T {}

/// A generated entry file parameterized by its output file kind.
///
/// `ApiFile` is reused for Rust `lib.rs` and npm `index.ts`: both need the same
/// API set, but they render imports, wrappers, and exports differently.
pub struct ApiFile<'a, F: FileName> {
    apis: Vec<Box<dyn Api + 'a>>,
    modules: Vec<Box<dyn PackageModule + 'a>>,
    file: PhantomData<F>,
    pub model: &'a Model,
    pub target: WasmTarget,
}

impl<'a, F: FileName> ApiFile<'a, F> {
    pub fn new(
        model: &'a Model,
        target: WasmTarget,
        apis: Vec<Box<dyn Api + 'a>>,
        modules: Vec<Box<dyn PackageModule + 'a>>,
    ) -> Self {
        Self {
            model,
            target,
            apis,
            modules,
            file: PhantomData,
        }
    }

    pub fn apis(&self) -> &[Box<dyn Api + 'a>] {
        &self.apis
    }

    pub fn modules(&self) -> &[Box<dyn PackageModule + 'a>] {
        &self.modules
    }

    pub fn file_name(&self) -> &'static str {
        F::FILE_NAME
    }
}

/// Writes TypeScript glue for one API function group.
pub trait TsWritable {
    fn write_ts(&self, writer: &mut SourceWriter) -> Result<(), Error>;
}

pub trait TsNodeWritable {
    fn write_ts_node(&self, writer: &mut SourceWriter) -> Result<(), Error>;
}

pub trait TsBrowserWritable {
    fn write_ts_browser(&self, writer: &mut SourceWriter) -> Result<(), Error>;
}

/// Writes wasm-bindgen Rust glue for one API function group.
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
    fn camel_case_encode_method_name() -> String {
        to_lower_camel_case(Self::ENCODE_METHOD_NAME)
    }
    fn camel_case_decode_method_name() -> String {
        to_lower_camel_case(Self::DECODE_METHOD_NAME)
    }
}

fn to_lower_camel_case(value: &str) -> String {
    let mut parts = value.split('_');
    let Some(first) = parts.next() else {
        return String::new();
    };
    let mut out = first.to_owned();
    for part in parts {
        let mut chars = part.chars();
        if let Some(first) = chars.next() {
            out.push_str(&first.to_uppercase().collect::<String>());
            out.push_str(chars.as_str());
        }
    }
    out
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
