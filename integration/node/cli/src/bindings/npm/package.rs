use super::TsConfigJson;
use crate::*;
use serde_json::json;
use std::fs;
use std::io::ErrorKind;
use std::path::PathBuf;
use std::process::Command;

pub struct NpmPackage<'a> {
    dir: PathBuf,
    type_files: &'a NpmTypeFiles<'a>,
    binding: PathBuf,
}

struct PackageJson<'a> {
    model: &'a Model,
}

const NATIVE_BINDING_PATH: &str = "native/bindings.node";

impl<'a> NpmPackage<'a> {
    pub fn new(
        dir: impl Into<PathBuf>,
        type_files: &'a NpmTypeFiles<'a>,
        binding: impl Into<PathBuf>,
    ) -> Self {
        Self {
            dir: dir.into(),
            type_files,
            binding: binding.into(),
        }
    }

    pub fn write(&self) -> Result<(), Error> {
        fs::create_dir_all(&self.dir)?;
        fs::create_dir_all(self.dir.join("native"))?;

        self.clean_owned_files()?;
        self.type_files.write_to(&self.dir)?;
        self.write_index_ts()?;
        PackageJson::new(self.type_files.model())
            .write_to_path(&self.dir.join(PackageJson::FILE_NAME))?;
        TsConfigJson::new().write_to_path(&self.dir.join(TsConfigJson::FILE_NAME))?;
        fs::copy(&self.binding, self.dir.join(NATIVE_BINDING_PATH))?;
        self.build()
    }

    fn write_index_ts(&self) -> Result<(), Error> {
        let model = self.type_files.model();
        let api = ApiFile::<NpmIndexFile>::new(
            &model.package,
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
        files.extend(
            PackageJson::new(self.type_files.model())
                .files()
                .into_iter()
                .map(PathBuf::from),
        );
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

impl<'a> PackageJson<'a> {
    const DEV_DEPS: &'static [(&'static str, &'static str)] = &[("typescript", "^5.9.2")];

    pub fn new(model: &'a Model) -> Self {
        Self { model }
    }
}

impl FileName for PackageJson<'_> {
    const FILE_NAME: &'static str = "package.json";
}

impl SourceWritable for PackageJson<'_> {
    fn write(&self, writer: &mut SourceWriter) -> Result<(), Error> {
        let dev_dependencies = Self::DEV_DEPS
            .iter()
            .map(|(name, version)| ((*name).to_owned(), json!(version)))
            .collect::<serde_json::Map<_, _>>();
        let package = json!({
            "name": self.model.package,
            "version": self.model.version,
            "private": true,
            "main": "index.js",
            "types": "index.d.ts",
            "scripts": {
                "build": "tsc -p tsconfig.json"
            },
            "files": self.files(),
            "devDependencies": dev_dependencies
        });

        writer.write(format!("{}\n", serde_json::to_string_pretty(&package)?))?;
        Ok(())
    }
}

impl PackageJson<'_> {
    fn files(&self) -> Vec<String> {
        fn compiled_file(source: &str, ext: &str) -> String {
            let stem = source.strip_suffix(".ts").unwrap_or(source);
            format!("{stem}.{ext}")
        }
        let mut files = vec!["index.js".to_owned(), "index.d.ts".to_owned()];
        for source in [
            BlocksFile::FILE_NAME,
            PayloadFile::FILE_NAME,
            PacketFile::FILE_NAME,
        ] {
            files.push(compiled_file(source, "js"));
            files.push(compiled_file(source, "d.ts"));
        }
        files.push(NATIVE_BINDING_PATH.to_owned());
        files
    }
}
