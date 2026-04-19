#![doc = include_str!("../README.md")]
#[cfg(test)]
#[cfg(feature = "generate_macro_test")]
mod tests;

mod codegen;
mod collector;
mod entities;
mod error;
mod generate;
mod integrations;
mod modificators;
mod parser;
mod parsing;
mod tokenized;

use codegen::*;
use collector::*;
use entities::*;
use error::*;
use generate::*;
use tokenized::*;

use proc_macro::TokenStream;
use syn::{DeriveInput, parse_macro_input};

/// Marks a struct as a `Block` type used within the `brec` system.
///
/// This macro enables code generation for the given struct, including:
/// - A companion `BlockReferred` struct (with signature and CRC metadata)
/// - Implementations of required `brec` traits for serialization, deserialization, and streaming
///
/// ## Supported Field Types
///
/// The struct may contain any combination of the following field types:
///
/// | Type     |
/// |----------|
/// | `u8`     |
/// | `u16`    |
/// | `u32`    |
/// | `u64`    |
/// | `u128`   |
/// | `i8`     |
/// | `i16`    |
/// | `i32`    |
/// | `i64`    |
/// | `i128`   |
/// | `f32`    |
/// | `f64`    |
/// | `bool`   |
/// | `[u8; N]`|
///
/// ## Enums
///
/// You may use custom `enum` fields in your block if you provide conversions to and from a supported base type.
/// Here's an example using `Level`:
///
/// ```ignore
/// pub enum Level {
///     Error,
///     Warn,
///     Info,
///     Debug,
/// }
///
/// impl From<&Level> for u8 {
///     fn from(value: &Level) -> Self { ... }
/// }
///
/// impl TryFrom<u8> for Level {
///     type Error = String;
///     fn try_from(value: u8) -> Result<Self, Self::Error> { ... }
/// }
///
/// #[block]
/// pub struct LogBlock {
///     pub level: Level,
///     pub message: [u8; 200],
/// }
/// ```
///
/// ## Visibility
///
/// The macro inherits the visibility of the original struct.  
/// If the struct is `pub`, the generated `BlockReferred` and trait implementations will also be `pub`.
///
/// ## Integration with Code Generator
///
/// The `#[block]` macro marks this struct for inclusion in the `brec::generate!()` macro.  
/// For this to work correctly, the block must be **visible** at the macro invocation site. Example:
///
/// ```ignore
/// pub use blocks::*;
/// brec::generate!();
/// ```
///
/// If you cannot import the block directly, you may specify the full module path via the `path` directive:
///
/// ```ignore
/// #[block(path = mycrate::some_module)]
/// pub struct ExternalBlock { ... }
/// ```
///
/// Shortcut syntax is also supported:
/// ```ignore
/// #[block(mycrate::some_module)]
/// pub struct ExternalBlock { ... }
/// ```
///
/// Using `path` works but is not recommended, as it makes the code harder to maintain.
///
/// ## Optional Attributes
///
/// The macro accepts the following optional directives:
///
/// - `path = mod::mod`  
///   Specifies the module path for this block type (used if it's not directly imported).
///
/// - `no_crc`  
///   Disables CRC calculation and verification. The CRC field is still included in the block's binary layout,  
///   but is filled with zeroes and not checked during read/write operations.
#[proc_macro_attribute]
pub fn block(attr: TokenStream, input: TokenStream) -> TokenStream {
    let attrs = parse_macro_input!(attr as BlockAttrs);
    let input = parse_macro_input!(input as DeriveInput);
    parser::block::parse(attrs, input).into()
}

/// Marks a user-defined type as a `Payload` for use in `brec`-compatible binary streams.
///
/// This macro enables participation of a struct or enum in the `Payload` system and code generation.
/// It also supports payload runtime context declarations via `#[payload(ctx)]`.
///
/// For regular payloads, the macro automatically implements most required traits for payload integration,
/// **except** the following:
///
/// ## Required Manual Trait Implementations
///
/// You must implement the following traits for your payload type:
///
/// - `PayloadSize`
/// - `PayloadSignature`
/// - `StaticPayloadSignature`
/// - `PayloadEncode`
/// - `PayloadEncodeReferred`
/// - `PayloadDecode<T>`
///
/// These are required to support serialization, CRC validation, and integration into packet streams.
///
/// ## Built-in Supported Payloads
///
/// `brec` automatically includes support for the following payload types without additional setup:
///
/// - [`String`](std::string::String)
/// - [`Vec<u8>`](std::vec::Vec)
///
/// When you invoke `brec::generate!()`, they are added to the generated `Payload` enum:
///
/// ```ignore
/// pub enum Payload {
///     // Your custom payloads
///     MyCustomType(...),
///     // Built-in default payloads
///     Bytes(Vec<u8>),
///     String(String),
/// }
/// ```
///
/// ## Using `#[payload(bincode)]`
///
/// To simplify payload integration, enable the `bincode` feature in your `Cargo.toml`.  
/// You can then use `#[payload(bincode)]` to automatically derive all required traits for
/// any `serde`-compatible type.
///
/// ```ignore
/// #[payload(bincode)]
/// #[derive(serde::Serialize, serde::Deserialize)]
/// pub struct MyPayload { ... }
///
/// #[payload(bincode)]
/// #[derive(serde::Serialize, serde::Deserialize)]
/// pub enum MyPayloadEnum { ... }
/// ```
///
/// The generated payload schema uses the crate-local `PayloadContext<'a>` type that is later emitted
/// by `brec::generate!()`.
///
/// ## Using `#[payload(ctx)]`
///
/// `#[payload(ctx)]` marks a type as payload runtime context instead of a regular payload.
///
/// Such a type:
///
/// - is **not** added to the generated `Payload` enum
/// - is used to build the generated crate-local `PayloadContext<'a>` enum
/// - is passed by mutable reference into payload encode/decode/size operations
///
/// Example:
///
/// ```ignore
/// #[payload(ctx)]
/// pub struct MyOptions {
///     pub prefix: String,
/// }
///
/// brec::generate!();
///
/// pub enum PayloadContext<'a> {
///     None,
///     MyOptions(&'a mut MyOptions),
/// }
/// ```
///
/// If there are no custom context types and no encrypted payloads, `brec::generate!()` emits:
///
/// ```ignore
/// pub type PayloadContext<'a> = ();
/// ```
///
/// ## Using `#[payload(bincode, crypt)]`
///
/// With both the `bincode` and `crypt` features enabled, `#[payload(bincode, crypt)]` generates
/// transparent payload encryption/decryption on top of `bincode` serialization.
///
/// The generated code expects the operation context to be one of the crypto variants created by
/// `brec::generate!()`:
///
/// ```ignore
/// let mut encrypt = PayloadContext::Encrypt(&mut encrypt_options);
/// let mut decrypt = PayloadContext::Decrypt(&mut decrypt_options);
/// ```
///
/// In other words:
///
/// - writing encrypted payloads uses `EncryptOptions`
/// - reading encrypted payloads uses `DecryptOptions`
/// - blocks remain unencrypted
/// - mixed protocols are supported: encrypted and non-encrypted payloads may coexist
///
/// ## Optional Parameters
///
/// The macro accepts several optional directives:
///
/// - `path = mod::mod`  
///   Use if the payload type is defined outside the module where `generate!()` is invoked.  
///   This avoids having to re-export the payload, but is not recommended due to maintenance concerns.
///
/// - `no_crc`  
///   Disables CRC verification for this payload.  
///   CRC field will be written as `0`s and skipped during validation.
///
/// - `no_auto_crc`  
///   Disables CRC calculation only for `#[payload(bincode)]`, requiring a **manual implementation** of `PayloadCrc`.
///
/// - `bincode` - available only when the bincode feature is enabled. It allows using any structure as a payload as
///   long as it meets the requirements of the bincode crate, i.e., it implements serde serialization and deserialization.
///   Please note that bincode has a number of limitations, which you can review in its official documentation.
///
/// - `ctx` - marks this type as payload runtime context instead of a regular payload.  
///   `brec::generate!()` collects such types into the generated `PayloadContext<'a>` enum.
///
/// - `crypt` - available only when the `crypt` feature is enabled and intended to be used together with `bincode`
///   as `#[payload(bincode, crypt)]`. It generates internal crypto-wrapper-based payload encode/decode
///   implementations using `brec::prelude::EncryptOptions` and `brec::prelude::DecryptOptions`.
///
/// ## CRC Verification Caveats
///
/// CRC is calculated based on the serialized byte representation of the payload. This implies:
///
/// - Serialization must be deterministic. Types like `HashMap` do **not** guarantee key order, causing CRC mismatch.
/// - Any nondeterministic or unstable byte layout will result in failed deserialization.
///
/// **Problematic example**:
/// ```ignore
/// #[payload(bincode)]
/// #[derive(serde::Serialize, serde::Deserialize)]
/// pub struct Problematic {
///     items: std::collections::HashMap<String, String>, // Non-deterministic order
/// }
/// ```
///
/// **Solutions**:
/// - Avoid unstable types like `HashMap`
/// - Use `#[payload(no_crc)]` to skip CRC validation
/// - Use `#[payload(bincode, no_auto_crc)]` and implement [`PayloadCrc`] manually
///
/// ## Visibility and Module Paths
///
/// If the payload is defined in another module not directly visible at the generator site, specify `path = mod::mod`:
///
/// ```ignore
/// #[payload(path = mycrate::data::types)]
/// pub struct MyPayload { ... }
/// ```
///
/// This works but is not the recommended way. Prefer explicitly importing your payload at the call site of `brec::generate!()`.
#[proc_macro_attribute]
pub fn payload(attr: TokenStream, input: TokenStream) -> TokenStream {
    let attrs = parse_macro_input!(attr as PayloadAttrs);
    let input = parse_macro_input!(input as DeriveInput);
    parser::payload::parse(attrs, input).into()
}

/// Derives `brec::NapiConvert` for regular Rust `struct` / `enum` types.
///
/// Use it for nested types used inside `#[payload]` objects when `napi` conversion
/// should be schema-driven and lossless for numeric edge cases.
///
/// See: <https://icsmw.github.io/brec/integrations/napi/>
#[proc_macro_derive(Napi)]
pub fn derive_napi(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let name = &input.ident;
    match integrations::codegen::napi::generate_impl(name, &input.data) {
        Ok(tokens) => tokens.into(),
        Err(err) => syn::Error::new_spanned(&input, err)
            .to_compile_error()
            .into(),
    }
}

/// Derives `brec::WasmConvert` for regular Rust `struct` / `enum` types.
///
/// Use it for nested types used inside `#[payload]` objects when `wasm` conversion
/// should be schema-driven and lossless for numeric edge cases.
///
/// See: <https://icsmw.github.io/brec/integrations/wasm/>
#[proc_macro_derive(Wasm)]
pub fn derive_wasm(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let name = &input.ident;
    match integrations::codegen::wasm::generate_impl(name, &input.data) {
        Ok(tokens) => tokens.into(),
        Err(err) => syn::Error::new_spanned(&input, err)
            .to_compile_error()
            .into(),
    }
}

/// Derives `brec::JavaConvert` for regular Rust `struct` / `enum` types.
///
/// Use it for nested types used inside `#[payload]` objects when `java` conversion
/// should be schema-driven for JNI-backed integrations.
///
/// See: <https://icsmw.github.io/brec/integrations/java/>
#[proc_macro_derive(Java)]
pub fn derive_java(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let name = &input.ident;
    match integrations::codegen::java::generate_impl(name, &input.data) {
        Ok(tokens) => tokens.into(),
        Err(err) => syn::Error::new_spanned(&input, err)
            .to_compile_error()
            .into(),
    }
}

/// Inserts the generated glue code that connects user-defined `Block` and `Payload` types with the `brec` framework.
///
/// This macro must be called exactly once per crate and is responsible for:
///
/// - Implementing required `brec` traits for all user-defined `Block` types
/// - Implementing required `brec` traits for all user-defined `Payload` types
/// - Generating unified enums for blocks: `enum Block { ... }`
/// - Generating unified enums for payloads: `enum Payload { ... }`
/// - Generating the crate-local payload runtime context type: `type PayloadContext<'a> = ()`
///   or `enum PayloadContext<'a> { ... }`
/// - Exporting several convenience type aliases to simplify usage
///
/// ### Generated Aliases
/// The macro defines the following aliases to reduce verbosity when using `brec` types:
///
/// | Alias                            | Expanded to                                                                                |
/// |----------------------------------|--------------------------------------------------------------------------------------------|
/// | `Packet`                         | `PacketDef<Block, Payload, Payload>`                                                       |
/// | `BorrowedPacketBufReader<'a, R>` | `PacketBufReaderDef<'a, R, Block, BlockReferred<'a>, Payload, Payload>`             |
/// | `PacketBufReader<'a, R>`         | same as `BorrowedPacketBufReader<'a, R>`                                                   |
/// | `PeekedBlocks<'a>`               | `PeekedBlocksDef<'a, BlockReferred<'a>>`                                                 |
/// | `PeekedBlock<'a>`                | `PeekedBlockDef<'a, BlockReferred<'a>>`                                                  |
/// | `BorrowedRules<'a>`              | `RulesDef<Block, BlockReferred<'a>, Payload, Payload>`                                    |
/// | `Rules<'a>`                      | same as `BorrowedRules<'a>`                                                               |
/// | `BorrowedRule<'a>`               | `RuleDef<Block, BlockReferred<'a>, Payload, Payload>`                                     |
/// | `Rule<'a>`                       | same as `BorrowedRule<'a>`                                                                |
/// | `RuleFnDef<D, S>`                | `RuleFnDef<D, S>`                                                                        |
/// | `BorrowedReader<'a, S>`          | `ReaderDef<S, Block, BlockReferred<'a>, Payload, Payload>`                               |
/// | `Reader<S>`                      | `ReaderDef<S, Block, BlockReferred<'static>, Payload, Payload>`                          |
/// | `Writer<S>`                      | `WriterDef<S, Block, Payload, Payload>`                                                  |
///
/// These aliases make it easier to work with generated structures and remove the need to repeat generic parameters.
///
/// `PayloadContext<'a>` is also generated alongside these aliases:
///
/// - `pub type PayloadContext<'a> = ()` when no custom `#[payload(ctx)]` types and no encrypted payloads exist
/// - `pub enum PayloadContext<'a> { None, ... }` when at least one context entry exists
/// - `PayloadContext::Encrypt(...)` / `PayloadContext::Decrypt(...)` are added automatically if at least one payload
///   uses `#[payload(bincode, crypt)]`
///
/// When `brec` is built with the `observer` feature, the macro also generates:
///
/// | Alias                    | Expanded to                                                                                 |
/// |--------------------------|---------------------------------------------------------------------------------------------|
/// | `SubscriptionUpdate`     | `brec::SubscriptionUpdate`                                                                  |
/// | `SubscriptionErrorAction`| `brec::SubscriptionErrorAction`                                                             |
/// | `Subscription`           | Local trait facade over `SubscriptionDef<Block, BlockReferred<'static>, Payload, Payload, ()>` |
/// | `FileObserverOptions<S>` | Local wrapper over `brec::FileObserverOptions<..., SubscriptionWrapper<S>>`                 |
/// | `FileObserver`           | Local wrapper over `FileObserverDef<Block, BlockReferred<'static>, Payload, Payload, ()>`  |
/// | `FileObserverStream`     | `brec::FileObserverStreamDef<Block, BlockReferred<'static>, Payload, Payload, ()>`         |
///
/// `Subscription` uses `on_*` callbacks: `on_update`, `on_packet`, `on_error`, `on_stopped`, `on_aborted`.
///
/// When `brec` is built with the `locked_storage` feature, the macro also generates:
///
/// | Alias         | Expanded to                                      |
/// |---------------|--------------------------------------------------|
/// | `FileStorage` | `brec::FileWriterDef<Block, Payload, Payload, ()>` |
///
/// ---
///
/// ### Usage Constraints
///
/// - The macro **must only be called once** per crate. Calling it more than once will result in compilation errors due to duplicate types and impls.
/// - The macro **must see all relevant types** (`Block`, `Payload`) in scope. You must ensure they are visible in the location where you call the macro.
///
/// ### Visibility Requirements
///
/// Ensure that all blocks and payloads are imported at the location where the macro is used:
/// ```ignore
/// pub use blocks::*;
/// pub use payloads::*;
///
/// brec::generate!();
/// ```
///
/// ### Parameters
///
/// The macro can be used with the following parameters:
///
/// - `no_default_payload` - Disables the built-in payloads (`String` and `Vec<u8>`).  
///   This has no impact on runtime performance but may slightly improve compile times and reduce binary size.
///
/// - `payloads_derive = "Trait"` -  
///   By default, `brec` automatically collects all `derive` attributes that are common across user-defined payloads
///   and applies them to the generated `Payload` enum.  
///   This parameter allows you to **manually** specify additional derives for the `Payload` enum-useful if you are
///   only using the built-in payloads (`String`, `Vec<u8>`) and do not define custom ones.
///
/// #### Examples
///
/// ```ignore
/// pub use blocks::*;
///
/// // You don't define any custom payloads and only want to use the built-in ones (`String`, `Vec<u8>`)
/// brec::generate!(payloads_derive = "Debug, Clone");
/// ```
///
/// ```ignore
/// pub use blocks::*;
///
/// // You don't define any payloads and explicitly disable the built-in ones
/// brec::generate!(no_default_payload);
/// ```
///
/// If the user **fully disables** payload support (as in the example above),
/// the macro will **not generate any packet-related types** (see *Generated Aliases*).
#[proc_macro]
pub fn generate(input: TokenStream) -> TokenStream {
    let config = parse_macro_input!(input as Config);
    generate::generate(&config).into()
}
