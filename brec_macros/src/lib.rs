#![doc = include_str!("../README.md")]
#[cfg(test)]
#[cfg(feature = "generate_macro_test")]
mod tests;

mod codegen;
mod collector;
mod entities;
mod error;
mod generate;
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
use syn::{parse_macro_input, DeriveInput};

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
/// It automatically implements most required traits for payload integration, **except** the following:
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
/// - `bincode` – available only when the bincode feature is enabled. It allows using any structure as a payload as
///   long as it meets the requirements of the bincode crate, i.e., it implements serde serialization and deserialization.
///   Please note that bincode has a number of limitations, which you can review in its official documentation.
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

/// Inserts the generated glue code that connects user-defined `Block` and `Payload` types with the `brec` framework.
///
/// This macro must be called exactly once per crate and is responsible for:
///
/// - Implementing required `brec` traits for all user-defined `Block` types
/// - Implementing required `brec` traits for all user-defined `Payload` types
/// - Generating unified enums for blocks: `enum Block { ... }`
/// - Generating unified enums for payloads: `enum Payload { ... }`
/// - Exporting several convenience type aliases to simplify usage
///
/// ### Generated Aliases
/// The macro defines the following aliases to reduce verbosity when using `brec` types:
///
/// | Alias                    | Expanded to                                                                 |
/// |-------------------------|------------------------------------------------------------------------------|
/// | `Packet`                | `PacketDef<Block, Payload, Payload>`                                        |
/// | `PacketBufReader<'a, R>`| `PacketBufReaderDef<'a, R, Block, BlockReferred<'a>, Payload, Payload>`     |
/// | `Rules<'a>`             | `RulesDef<Block, BlockReferred<'a>, Payload, Payload>`                      |
/// | `Rule<'a>`              | `RuleDef<Block, BlockReferred<'a>, Payload, Payload>`                       |
/// | `RuleFnDef<D, S>`       | `RuleFnDef<D, S>`                                                            |
/// | `Storage<S>`            | `StorageDef<S, Block, BlockReferred<'static>, Payload, Payload>`            |
///
/// These aliases make it easier to work with generated structures and remove the need to repeat generic parameters.
///
/// ---
///
/// ### Required Build Script
///
/// To enable this macro, you **must** include a `build.rs` file with the following content:
/// ```ignore
///     brec::build_setup();
/// ```
/// This step ensures the code generator runs during build and provides all required metadata.
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
/// - `no_default_payload` – Disables the built-in payloads (`String` and `Vec<u8>`).  
///   This has no impact on runtime performance but may slightly improve compile times and reduce binary size.
///
/// - `payloads_derive = "Trait"` –  
///   By default, `brec` automatically collects all `derive` attributes that are common across user-defined payloads
///   and applies them to the generated `Payload` enum.  
///   This parameter allows you to **manually** specify additional derives for the `Payload` enum—useful if you are
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
