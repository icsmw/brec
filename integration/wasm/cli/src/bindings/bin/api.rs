use crate::*;

impl RustWritable for ApiBlock {
    fn write_rust(&self, writer: &mut SourceWriter) -> Result<(), Error> {
        writer.block(format!(
            r#"
#[wasm_bindgen]
pub fn {}(buf: &[u8]) -> Result<JsValue, JsValue> {{
	Block::decode_wasm(buf).map_err(to_js_error)
}}

#[wasm_bindgen]
pub fn {}(val: JsValue) -> Result<Vec<u8>, JsValue> {{
	let mut buf: Vec<u8> = Vec::new();
	Block::encode_wasm(val, &mut buf).map_err(to_js_error)?;
	Ok(buf)
}}
"#,
            Self::snake_case_decode_method_name(),
            Self::snake_case_encode_method_name()
        ))
    }
}

impl RustWritable for ApiPayload {
    fn write_rust(&self, writer: &mut SourceWriter) -> Result<(), Error> {
        writer.block(format!(
            r#"
#[wasm_bindgen]
pub fn {}(buf: &[u8]) -> Result<JsValue, JsValue> {{
	let mut ctx = ();
	Payload::decode_wasm(buf, &mut ctx).map_err(to_js_error)
}}

#[wasm_bindgen]
pub fn {}(val: JsValue) -> Result<Vec<u8>, JsValue> {{
	let mut buf: Vec<u8> = Vec::new();
	let mut ctx = ();
	Payload::encode_wasm(val, &mut buf, &mut ctx).map_err(to_js_error)?;
	Ok(buf)
}}
"#,
            Self::snake_case_decode_method_name(),
            Self::snake_case_encode_method_name()
        ))
    }
}

impl RustWritable for ApiPacket {
    fn write_rust(&self, writer: &mut SourceWriter) -> Result<(), Error> {
        writer.block(format!(
            r#"
#[wasm_bindgen]
pub fn {}(buf: &[u8]) -> Result<JsValue, JsValue> {{
	let mut ctx = ();
	Packet::decode_wasm(buf, &mut ctx).map_err(to_js_error)
}}

#[wasm_bindgen]
pub fn {}(packet: JsValue) -> Result<Vec<u8>, JsValue> {{
	let mut buf: Vec<u8> = Vec::new();
	let mut ctx = ();
	Packet::encode_wasm(packet, &mut buf, &mut ctx).map_err(to_js_error)?;
	Ok(buf)
}}
"#,
            Self::snake_case_decode_method_name(),
            Self::snake_case_encode_method_name()
        ))
    }
}
