`brec_macros` is a component of the `brec` crate. While it can technically be used independently, it does **not** provide the full functionality that `brec` offers on its own.

`brec` is a toolkit for quickly and easily creating custom message exchange protocols with built-in resilience to data corruption and the ability to extract messages from mixed streams (i.e., streams that contain both `brec` packets and arbitrary non-`brec` data).

`brec_macros` provides proc-macros used to define blocks/payloads and to generate protocol glue code. It is the generation layer used by the public `brec` crate from `lib/core`, while integration-specific generators live under `integration/*`.

- C# conversion when the `csharp` feature is enabled in `brec`
- N-API conversion when the `napi` feature is enabled in `brec`
- wasm-bindgen conversion when the `wasm` feature is enabled in `brec`
- Java conversion via JNI when the `java` feature is enabled in `brec`

For complete documentation and usage details:

- Repository: https://github.com/icsmw/brec
- Documentation: https://icsmw.github.io/brec/
- C# feature: https://icsmw.github.io/brec/integrations/csharp/
- N-API feature: https://icsmw.github.io/brec/integrations/napi/
- WASM feature: https://icsmw.github.io/brec/integrations/wasm/
- Java feature: https://icsmw.github.io/brec/integrations/java/
