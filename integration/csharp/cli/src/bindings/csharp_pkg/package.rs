use super::bindings_file::BindingsFile;
use super::blocks_file::BlocksFile;
use super::packet_file::PacketFile;
use super::payloads_file::PayloadsFile;
use super::project_file::ProjectFile;
use crate::*;
use std::fs;
use std::path::PathBuf;
use std::process::Command;

pub struct CSharpPackage<'a> {
    dir: PathBuf,
    model: &'a Model,
    binding: PathBuf,
}

impl<'a> CSharpPackage<'a> {
    pub fn new(dir: impl Into<PathBuf>, model: &'a Model, binding: impl Into<PathBuf>) -> Self {
        Self {
            dir: dir.into(),
            model,
            binding: binding.into(),
        }
    }

    pub fn write(&self) -> Result<(), Error> {
        fs::create_dir_all(&self.dir)?;
        fs::create_dir_all(self.dir.join("native"))?;

        ProjectFile::new(self.model).write_to_path(&self.dir.join(ProjectFile::FILE_NAME))?;
        BindingsFile::new(self.model).write_to_path(&self.dir.join(BindingsFile::FILE_NAME))?;
        BlocksFile::new(self.model).write_to_path(&self.dir.join(BlocksFile::FILE_NAME))?;
        PayloadsFile::new(self.model).write_to_path(&self.dir.join(PayloadsFile::FILE_NAME))?;
        PacketFile::new(self.model).write_to_path(&self.dir.join(PacketFile::FILE_NAME))?;

        let native_file_name = native_binding_name();
        fs::copy(
            &self.binding,
            self.dir.join("native").join(native_file_name),
        )?;
        self.build()
    }

    fn build(&self) -> Result<(), Error> {
        let status = match Command::new("dotnet")
            .arg("build")
            .arg(ProjectFile::FILE_NAME)
            .current_dir(&self.dir)
            .status()
        {
            Ok(status) => status,
            Err(err) if err.kind() == std::io::ErrorKind::NotFound => return Ok(()),
            Err(err) => return Err(err.into()),
        };

        if !status.success() {
            return Err(Error::CommandFailed {
                command: format!("dotnet build {}", ProjectFile::FILE_NAME),
                status,
            });
        }

        Ok(())
    }
}

fn native_binding_name() -> &'static str {
    match std::env::consts::OS {
        "windows" => "bindings.dll",
        "macos" => "libbindings.dylib",
        _ => "libbindings.so",
    }
}
