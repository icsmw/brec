#[cfg(feature = "csharp")]
#[path = "csharp.rs"]
/// Rust-side C# conversion helpers built on a stable value ABI.
///
/// See: <https://icsmw.github.io/brec/integrations/csharp/>
pub mod csharp_feature;
#[cfg(feature = "java")]
#[path = "java.rs"]
/// JNI conversion helpers for Java runtimes.
///
/// See: <https://icsmw.github.io/brec/integrations/java/>
pub mod java_feature;
#[cfg(feature = "napi")]
/// N-API conversion helpers for Node.js runtimes.
///
/// See: <https://icsmw.github.io/brec/integrations/napi/>
pub mod napi;
#[cfg(feature = "wasm")]
#[path = "wasm.rs"]
/// wasm-bindgen conversion helpers for browser/wasm JavaScript runtimes.
///
/// See: <https://icsmw.github.io/brec/integrations/wasm/>
pub mod wasm_feature;
