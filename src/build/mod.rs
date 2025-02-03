pub fn build_setup() {
    std::env::var("OUT_DIR").expect("OUT_DIR not set; required for brec crate");
    println!("cargo:rerun-if-changed=build.rs");
}
