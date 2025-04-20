
`brec` generates code in two stages:

- When the `#[block]` macro is used, it generates code specific to the corresponding block.  
  The same applies to the `#[payload]` macro, which generates code related to a specific payload type.

- However, the protocol also requires unified types such as `enum Block` (enumerating all user-defined blocks)  
  and `enum Payload` (enumerating all payloads). These general-purpose enums allow the system to handle  
  various blocks and payloads dynamically.

To generate this unified code, the `generate!()` macro must be invoked:

```rust
pub use blocks::*;
pub use payloads::*;

brec::generate!();
```

This macro must be called exactly **once per crate** and is responsible for:

- Implementing required `brec` traits for all user-defined `Block` types
- Implementing required `brec` traits for all user-defined `Payload` types
- Generating unified enums for blocks: `enum Block { ... }`
- Generating unified enums for payloads: `enum Payload { ... }`
- Exporting several convenience type aliases to simplify usage

### Generated Aliases
The macro defines the following aliases to reduce verbosity when using `brec` types:

| Alias                    | Expanded to                                                                 |
|-------------------------|------------------------------------------------------------------------------|
| `Packet`                | `PacketDef<Block, Payload, Payload>`                                        |
| `PacketBufReader<'a, R>`| `PacketBufReaderDef<'a, R, Block, BlockReferred<'a>, Payload, Payload>`     |
| `Rules<'a>`             | `RulesDef<Block, BlockReferred<'a>, Payload, Payload>`                      |
| `Rule<'a>`              | `RuleDef<Block, BlockReferred<'a>, Payload, Payload>`                       |
| `RuleFnDef<D, S>`       | `RuleFnDef<D, S>`                                                            |
| `Storage<S>`            | `StorageDef<S, Block, BlockReferred<'static>, Payload, Payload>`            |

These aliases make it easier to work with generated structures and remove the need to repeat generic parameters.

### Required Build Script

To enable this macro, you **must** include a `build.rs` file with the following content:
```rust
    brec::build_setup();
```
This step ensures the code generator runs during build and provides all required metadata.

### Usage Constraints

- The macro **must only be called once** per crate. Calling it more than once will result in compilation errors due to duplicate types and impls.
- The macro **must see all relevant types** (`Block`, `Payload`) in scope. You must ensure they are visible in the location where you call the macro.

### Visibility Requirements

Ensure that all blocks and payloads are imported at the location where the macro is used:
```rust
pub use blocks::*;
pub use payloads::*;

brec::generate!();
```

### Parameters

The macro can be used with the following parameters:

- `no_default_payload` – Disables the built-in payloads (`String` and `Vec<u8>`).  
  This has no impact on runtime performance but may slightly improve compile times and reduce binary size.

- `payloads_derive = "Trait"` –  
  By default, `brec` automatically collects all `derive` attributes that are common across user-defined payloads
  and applies them to the generated `Payload` enum.  
  This parameter allows you to **manually** specify additional derives for the `Payload` enum—useful if you are
  only using the built-in payloads (`String`, `Vec<u8>`) and do not define custom ones.

For example,

```rust
pub use blocks::*;

// You don't define any custom payloads and only want to use the built-in ones (`String`, `Vec<u8>`)
brec::generate!(payloads_derive = "Debug, Clone");
```

```rust
pub use blocks::*;

// You don't define any payloads and explicitly disable the built-in ones
brec::generate!(no_default_payload);
```

If the user **fully disables** payload support (as in the example above),
the macro will **not generate any packet-related types** (see *Generated Aliases*).
