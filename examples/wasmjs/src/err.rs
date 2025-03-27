use thiserror::Error;
use wasm_bindgen::JsValue;

#[derive(Error, Debug)]
pub enum ConvertorError {
    #[error("Write error {0}")]
    WriteError(String),
    #[error("Read error {0}")]
    ReadError(String),
    #[error("Payload header reading: {0}")]
    PayloadHeaderReading(String),
    #[error("Serialize error: {0}")]
    SerializeError(String),
    #[error("Serde error: {0}")]
    Serde(serde_wasm_bindgen::Error),
}

impl From<serde_wasm_bindgen::Error> for ConvertorError {
    fn from(err: serde_wasm_bindgen::Error) -> Self {
        Self::Serde(err)
    }
}

impl From<ConvertorError> for JsValue {
    fn from(val: ConvertorError) -> Self {
        JsValue::from_str(&val.to_string())
    }
}
