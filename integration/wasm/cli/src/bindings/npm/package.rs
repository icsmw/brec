use super::TsConfigJson;
use super::package_json::PackageJson;
use crate::*;
use std::fs;
use std::io::ErrorKind;
use std::path::{Path, PathBuf};
use std::process::Command;

/// Writable npm package assembled from wasm-pack output and generated
/// TypeScript files.
///
/// `NpmPackage` is the final packaging step: it cleans only known generated
/// files, writes fresh TypeScript declarations and metadata, then runs the
/// local npm build.
pub struct NpmPackage<'a> {
    dir: PathBuf,
    type_files: &'a NpmTypeFiles<'a>,
    deps: Option<PathBuf>,
    target: WasmTarget,
}

impl<'a> NpmPackage<'a> {
    pub fn new(
        dir: impl Into<PathBuf>,
        type_files: &'a NpmTypeFiles<'a>,
        deps: Option<PathBuf>,
        target: WasmTarget,
    ) -> Self {
        Self {
            dir: dir.into(),
            type_files,
            deps,
            target,
        }
    }

    pub fn write(&self) -> Result<(), Error> {
        fs::create_dir_all(&self.dir)?;

        self.clean_owned_files()?;
        self.type_files.write_to(&self.dir)?;
        self.write_index_ts()?;
        PackageJson::new(
            self.type_files.model(),
            &self.dir,
            self.deps.as_deref(),
            self.target,
        )
        .write_to_path(&self.dir.join(PackageJson::FILE_NAME))?;
        TsConfigJson::new(self.target).write_to_path(&self.dir.join(TsConfigJson::FILE_NAME))?;
        self.build()
    }

    pub fn validate_dependencies(
        dir: impl Into<PathBuf>,
        model: &'a Model,
        deps: Option<&Path>,
        target: WasmTarget,
    ) -> Result<(), Error> {
        PackageJson::new(model, dir, deps, target).validate_dependencies()
    }

    fn write_index_ts(&self) -> Result<(), Error> {
        let model = self.type_files.model();
        let api = ApiFile::<NpmIndexFile>::new(
            model,
            self.target,
            vec![
                Box::new(ApiBlock),
                Box::new(ApiPayload),
                Box::new(ApiPacket),
            ],
            vec![
                Box::new(BlocksFile::from(model)),
                Box::new(PayloadFile::from(model)),
                Box::new(PacketFile::new(model)),
            ],
        );
        api.write_to_path(&self.dir.join(NpmIndexFile::FILE_NAME))
    }

    /// Removes files owned by the generator before writing fresh output.
    ///
    /// User-created files are left intact; only known generated wrapper files
    /// are touched. wasm-pack artifacts are preserved.
    fn clean_owned_files(&self) -> Result<(), Error> {
        for file in self.owned_files() {
            let path = self.dir.join(file);
            match fs::symlink_metadata(&path) {
                Ok(meta) if meta.is_file() || meta.file_type().is_symlink() => {
                    fs::remove_file(path)?;
                }
                Ok(meta) if meta.is_dir() => {
                    return Err(Error::Cli(format!(
                        "cannot overwrite generated file {}; path is a directory",
                        path.display()
                    )));
                }
                Ok(_) => {}
                Err(err) if err.kind() == ErrorKind::NotFound => {}
                Err(err) => return Err(err.into()),
            }
        }
        Ok(())
    }

    fn owned_files(&self) -> Vec<PathBuf> {
        let mut files = vec![
            PathBuf::from(NpmIndexFile::FILE_NAME),
            PathBuf::from(BlocksFile::FILE_NAME),
            PathBuf::from(PayloadFile::FILE_NAME),
            PathBuf::from(PacketFile::FILE_NAME),
            PathBuf::from(PackageJson::FILE_NAME),
            PathBuf::from(TsConfigJson::FILE_NAME),
        ];
        for source in [
            NpmIndexFile::FILE_NAME,
            BlocksFile::FILE_NAME,
            PayloadFile::FILE_NAME,
            PacketFile::FILE_NAME,
        ] {
            let stem = source.strip_suffix(".ts").unwrap_or(source);
            files.push(PathBuf::from(format!("{stem}.js")));
            files.push(PathBuf::from(format!("{stem}.d.ts")));
        }
        files
    }

    fn build(&self) -> Result<(), Error> {
        self.run("npm", ["install", "--package-lock=false"])?;
        self.run("npm", ["run", "build"])
    }

    fn run<const N: usize>(&self, program: &str, args: [&str; N]) -> Result<(), Error> {
        let status = Command::new(program)
            .args(args)
            .current_dir(&self.dir)
            .status()?;

        if !status.success() {
            return Err(Error::CommandFailed {
                command: format!("{} {}", program, args.join(" ")),
                status,
            });
        }

        Ok(())
    }
}
