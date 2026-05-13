use super::cargo_toml::CargoToml;
use super::lib_rs::BindingsLibFile;
use crate::*;
use std::path::{Path, PathBuf};
use std::process::Command;

/// Generated Rust crate that exposes protocol encode/decode functions to WASM.
///
/// The crate is written next to the scheme by default, depends on the user's
/// protocol crate, and is packaged with `wasm-pack`.
pub struct BindingsCrate<'a> {
    dir: PathBuf,
    cargo_toml: CargoToml<'a>,
    model: &'a Model,
}

impl<'a> BindingsCrate<'a> {
    pub const DIR_NAME: &'static str = "bindings";

    pub fn new(
        dir: impl Into<PathBuf>,
        model: &'a Model,
        protocol_dir: impl Into<PathBuf>,
        deps: Option<PathBuf>,
    ) -> Result<Self, Error> {
        let dir = dir.into();
        Ok(Self {
            cargo_toml: CargoToml::new(model, &dir, protocol_dir, deps),
            dir,
            model,
        })
    }

    pub fn write(&self) -> Result<(), Error> {
        let src = self.dir.join("src");
        std::fs::create_dir_all(&src)?;
        self.cargo_toml
            .write_to_path(&self.dir.join(CargoToml::FILE_NAME))?;
        self.write_lib_rs(&src.join(BindingsLibFile::FILE_NAME))
    }

    /// Builds the generated crate into a wasm-pack npm package directory.
    pub fn build_package(&self, out_dir: &Path, target: WasmTarget) -> Result<(), Error> {
        let out_dir = out_dir.canonicalize()?;
        let status = Command::new("wasm-pack")
            .arg("build")
            .arg("--target")
            .arg(target.wasm_pack_target())
            .arg("--out-dir")
            .arg(&out_dir)
            .arg("--out-name")
            .arg("wasmjs")
            .arg(&self.dir)
            .status()?;

        if !status.success() {
            return Err(Error::CommandFailed {
                command: format!(
                    "wasm-pack build --target {} --out-dir {} --out-name wasmjs {}",
                    target.wasm_pack_target(),
                    out_dir.display(),
                    self.dir.display()
                ),
                status,
            });
        }

        Ok(())
    }

    fn write_lib_rs(&self, path: &Path) -> Result<(), Error> {
        let module = ApiFile::<BindingsLibFile>::new(
            self.model,
            WasmTarget::Node,
            vec![
                Box::new(ApiBlock),
                Box::new(ApiPayload),
                Box::new(ApiPacket),
            ],
            Vec::new(),
        );
        module.write_to_path(path)
    }
}
