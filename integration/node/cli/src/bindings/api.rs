use crate::*;
use std::marker::PhantomData;

pub struct ApiBlock;
pub struct ApiPayload;
pub struct ApiPacket;

pub trait Api: TsWritable + RustWritable {}

impl<T: TsWritable + RustWritable> Api for T {}

pub struct ApiFile<'a, F: FileName> {
    package: &'a str,
    apis: Vec<Box<dyn Api + 'a>>,
    modules: Vec<Box<dyn PackageModule + 'a>>,
    file: PhantomData<F>,
}

impl<'a, F: FileName> ApiFile<'a, F> {
    pub fn new(
        package: &'a str,
        apis: Vec<Box<dyn Api + 'a>>,
        modules: Vec<Box<dyn PackageModule + 'a>>,
    ) -> Self {
        Self {
            package,
            apis,
            modules,
            file: PhantomData,
        }
    }

    pub fn package(&self) -> &'a str {
        self.package
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

pub trait TsWritable {
    fn write_ts(&self, writer: &mut SourceWriter) -> Result<(), Error>;
}

pub trait RustWritable {
    fn write_rust(&self, writer: &mut SourceWriter) -> Result<(), Error>;
}

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
