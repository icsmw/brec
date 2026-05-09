use crate::*;

impl FormattableTs for ApiBlock {
    fn write_ts(&self, writer: &mut FormatterWriter) -> std::fmt::Result {
        writer.ln(format!(
            "export const {} = pick('{}', '{}') as (bytes: Uint8Array) => Block;",
            Self::camel_case_decode_method_name(),
            Self::camel_case_decode_method_name(),
            Self::snake_case_decode_method_name()
        ))?;
        writer.ln(format!(
            "export const {} = pick('{}', '{}') as (block: Block) => Uint8Array;",
            Self::camel_case_encode_method_name(),
            Self::camel_case_encode_method_name(),
            Self::snake_case_encode_method_name()
        ))
    }
}

impl FormattableTs for ApiPayload {
    fn write_ts(&self, writer: &mut FormatterWriter) -> std::fmt::Result {
        writer.ln(format!(
            "export const {} = pick('{}', '{}') as (bytes: Uint8Array) => Payload;",
            Self::camel_case_decode_method_name(),
            Self::camel_case_decode_method_name(),
            Self::snake_case_decode_method_name()
        ))?;
        writer.ln(format!(
            "export const {} = pick('{}', '{}') as (payload: Payload) => Uint8Array;",
            Self::camel_case_encode_method_name(),
            Self::camel_case_encode_method_name(),
            Self::snake_case_encode_method_name()
        ))
    }
}

impl FormattableTs for ApiPacket {
    fn write_ts(&self, writer: &mut FormatterWriter) -> std::fmt::Result {
        writer.ln(format!(
            "export const {} = pick('{}', '{}') as (bytes: Uint8Array) => Packet;",
            Self::camel_case_decode_method_name(),
            Self::camel_case_decode_method_name(),
            Self::snake_case_decode_method_name()
        ))?;
        writer.ln(format!(
            "export const {} = pick('{}', '{}') as (packet: Packet) => Uint8Array;",
            Self::camel_case_encode_method_name(),
            Self::camel_case_encode_method_name(),
            Self::snake_case_encode_method_name()
        ))
    }
}
