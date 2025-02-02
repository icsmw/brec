use std::env;

fn main() {
    env::var("OUT_DIR").expect("OUT_DIR not set");
    println!("cargo:rerun-if-changed=build.rs");
}
