`brec_macros` is a component of the `brec` crate. While it can technically be used independently, it does **not** provide the full functionality that `brec` offers on its own.

`brec` is a toolkit for quickly and easily creating custom message exchange protocols with built-in resilience to data corruption and the ability to extract messages from mixed streams (i.e., streams that contain both `brec` packets and arbitrary non-`brec` data).

`brec_macros` provides proc-macros used to define blocks/payloads and to generate protocol glue code, including optional N-API conversion paths when the `napi` feature is enabled in `brec`.

For complete documentation and usage details:

- Repository: https://github.com/icsmw/brec
- Documentation: https://icsmw.github.io/brec/
- N-API feature: https://icsmw.github.io/brec/parts/napi/
