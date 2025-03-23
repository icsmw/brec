/// Prepares the build environment to enable the `brec::include_generated!()` macro.
///
/// This function should be called from your `build.rs` script as follows:
///
/// ```rust
/// fn main() {
///     brec::build_setup();
/// }
/// ```
///
/// Calling this ensures that the required code generation step is executed during
/// the build process, allowing the `brec::include_generated!()` macro to function properly.
pub fn build_setup() {
    std::env::var("OUT_DIR").expect("OUT_DIR not set; required for brec crate");
    println!("cargo:rerun-if-changed=build.rs");
}
