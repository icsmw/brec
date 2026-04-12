use thiserror::Error;
use wasm_bindgen::JsValue;

#[derive(Error, Debug)]
pub enum ConvertorError {
    #[error("Conversion error: {0}")]
    Conversion(String),
}

impl From<brec::Error> for ConvertorError {
    fn from(err: brec::Error) -> Self {
        Self::Conversion(err.to_string())
    }
}

impl From<ConvertorError> for JsValue {
    fn from(val: ConvertorError) -> Self {
        JsValue::from_str(&val.to_string())
    }
}
