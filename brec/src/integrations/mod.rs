#[cfg(feature = "java")]
#[path = "java/mod.rs"]
/// JNI conversion helpers for Java runtimes.
pub mod java_feature;
#[cfg(feature = "napi")]
#[path = "napi/mod.rs"]
/// N-API (Node.js) conversion helpers.
///
/// See: <https://icsmw.github.io/brec/integrations/napi/>
pub mod napi_feature;
#[cfg(feature = "wasm")]
#[path = "wasm/mod.rs"]
/// wasm-bindgen conversion helpers for browser/wasm JavaScript runtimes.
///
/// See: <https://icsmw.github.io/brec/integrations/wasm/>
pub mod wasm_feature;

#[cfg(feature = "java")]
pub use java_feature::*;
#[cfg(feature = "napi")]
pub use napi_feature::*;
#[cfg(feature = "wasm")]
pub use wasm_feature::*;
