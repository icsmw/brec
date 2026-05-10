use crate::*;

fn write_api_binding(
    writer: &mut SourceWriter,
    decode_method: String,
    encode_method: String,
    decode_snake: &str,
    encode_snake: &str,
    decode_ty: &str,
    encode_ty: &str,
) -> Result<(), Error> {
    writer.ln(format!(
        "export const {decode_method} = pick('{decode_method}', '{decode_snake}') as (bytes: Uint8Array) => {decode_ty};",
    ))?;
    writer.ln(format!(
        "export const {encode_method} = pick('{encode_method}', '{encode_snake}') as ({encode_ty}) => Uint8Array;",
    ))
}

impl TsNodeWritable for ApiBlock {
    fn write_ts_node(&self, writer: &mut SourceWriter) -> Result<(), Error> {
        self.write_ts_browser(writer)
    }
}

impl TsBrowserWritable for ApiBlock {
    fn write_ts_browser(&self, writer: &mut SourceWriter) -> Result<(), Error> {
        write_api_binding(
            writer,
            Self::camel_case_decode_method_name(),
            Self::camel_case_encode_method_name(),
            Self::snake_case_decode_method_name(),
            Self::snake_case_encode_method_name(),
            "Block",
            "block: Block",
        )
    }
}

impl TsNodeWritable for ApiPayload {
    fn write_ts_node(&self, writer: &mut SourceWriter) -> Result<(), Error> {
        self.write_ts_browser(writer)
    }
}

impl TsBrowserWritable for ApiPayload {
    fn write_ts_browser(&self, writer: &mut SourceWriter) -> Result<(), Error> {
        write_api_binding(
            writer,
            Self::camel_case_decode_method_name(),
            Self::camel_case_encode_method_name(),
            Self::snake_case_decode_method_name(),
            Self::snake_case_encode_method_name(),
            "Payload",
            "payload: Payload",
        )
    }
}

impl TsNodeWritable for ApiPacket {
    fn write_ts_node(&self, writer: &mut SourceWriter) -> Result<(), Error> {
        self.write_ts_browser(writer)
    }
}

impl TsBrowserWritable for ApiPacket {
    fn write_ts_browser(&self, writer: &mut SourceWriter) -> Result<(), Error> {
        write_api_binding(
            writer,
            Self::camel_case_decode_method_name(),
            Self::camel_case_encode_method_name(),
            Self::snake_case_decode_method_name(),
            Self::snake_case_encode_method_name(),
            "Packet",
            "packet: Packet",
        )
    }
}

impl TsWritable for ApiBlock {
    fn write_ts(&self, writer: &mut SourceWriter) -> Result<(), Error> {
        self.write_ts_node(writer)
    }
}

impl TsWritable for ApiPayload {
    fn write_ts(&self, writer: &mut SourceWriter) -> Result<(), Error> {
        self.write_ts_node(writer)
    }
}

impl TsWritable for ApiPacket {
    fn write_ts(&self, writer: &mut SourceWriter) -> Result<(), Error> {
        self.write_ts_node(writer)
    }
}
