use crate::*;

impl RustWritable for ApiBlock {
    fn write_rust(&self, writer: &mut SourceWriter) -> Result<(), Error> {
        writer.ln("#[wasm_bindgen]")?;
        writer.ln(format!(
            "pub fn {}(buf: &[u8]) -> Result<JsValue, JsValue> {{",
            Self::snake_case_decode_method_name()
        ))?;
        writer.tab();
        writer.ln("Block::decode_wasm(buf).map_err(to_js_error)")?;
        writer.back();
        writer.ln("}")?;
        writer.ln("")?;
        writer.ln("#[wasm_bindgen]")?;
        writer.ln(format!(
            "pub fn {}(val: JsValue) -> Result<Vec<u8>, JsValue> {{",
            Self::snake_case_encode_method_name()
        ))?;
        writer.tab();
        writer.ln("let mut buf: Vec<u8> = Vec::new();")?;
        writer.ln("Block::encode_wasm(val, &mut buf).map_err(to_js_error)?;")?;
        writer.ln("Ok(buf)")?;
        writer.back();
        writer.ln("}")?;
        writer.ln("")
    }
}

impl RustWritable for ApiPayload {
    fn write_rust(&self, writer: &mut SourceWriter) -> Result<(), Error> {
        writer.ln("#[wasm_bindgen]")?;
        writer.ln(format!(
            "pub fn {}(buf: &[u8]) -> Result<JsValue, JsValue> {{",
            Self::snake_case_decode_method_name()
        ))?;
        writer.tab();
        writer.ln("let mut ctx = ();")?;
        writer.ln("Payload::decode_wasm(buf, &mut ctx).map_err(to_js_error)")?;
        writer.back();
        writer.ln("}")?;
        writer.ln("")?;
        writer.ln("#[wasm_bindgen]")?;
        writer.ln(format!(
            "pub fn {}(val: JsValue) -> Result<Vec<u8>, JsValue> {{",
            Self::snake_case_encode_method_name()
        ))?;
        writer.tab();
        writer.ln("let mut buf: Vec<u8> = Vec::new();")?;
        writer.ln("let mut ctx = ();")?;
        writer.ln("Payload::encode_wasm(val, &mut buf, &mut ctx).map_err(to_js_error)?;")?;
        writer.ln("Ok(buf)")?;
        writer.back();
        writer.ln("}")?;
        writer.ln("")
    }
}

impl RustWritable for ApiPacket {
    fn write_rust(&self, writer: &mut SourceWriter) -> Result<(), Error> {
        writer.ln("#[wasm_bindgen]")?;
        writer.ln(format!(
            "pub fn {}(buf: &[u8]) -> Result<JsValue, JsValue> {{",
            Self::snake_case_decode_method_name()
        ))?;
        writer.tab();
        writer.ln("let mut ctx = ();")?;
        writer.ln("Packet::decode_wasm(buf, &mut ctx).map_err(to_js_error)")?;
        writer.back();
        writer.ln("}")?;
        writer.ln("")?;
        writer.ln("#[wasm_bindgen]")?;
        writer.ln(format!(
            "pub fn {}(packet: JsValue) -> Result<Vec<u8>, JsValue> {{",
            Self::snake_case_encode_method_name()
        ))?;
        writer.tab();
        writer.ln("let mut buf: Vec<u8> = Vec::new();")?;
        writer.ln("let mut ctx = ();")?;
        writer.ln("Packet::encode_wasm(packet, &mut buf, &mut ctx).map_err(to_js_error)?;")?;
        writer.ln("Ok(buf)")?;
        writer.back();
        writer.ln("}")?;
        writer.ln("")
    }
}
