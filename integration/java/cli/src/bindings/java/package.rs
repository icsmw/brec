use crate::*;
use brec_scheme::SchemeFile;
use std::fs;
use std::io::ErrorKind;
use std::path::PathBuf;
use std::process::Command;

/// Writable Java package assembled from generated source files and a native
/// JNI binding artifact.
pub struct JavaPackage<'a> {
    dir: PathBuf,
    model: &'a Model,
    binding: PathBuf,
}

impl<'a> JavaPackage<'a> {
    pub fn new(dir: impl Into<PathBuf>, model: &'a Model, binding: impl Into<PathBuf>) -> Self {
        Self {
            dir: dir.into(),
            model,
            binding: binding.into(),
        }
    }

    pub fn write(&self) -> Result<(), Error> {
        fs::create_dir_all(self.source_dir())?;
        fs::create_dir_all(self.native_dir())?;
        self.clean_owned_files()?;
        self.clean_classes()?;

        let files = JavaFiles::new(self.model, self.scheme()?);
        files.write_to(&self.source_dir())?;
        self.copy_binding()?;
        self.build()
    }

    fn scheme(&self) -> Result<SchemeFile, Error> {
        let content = fs::read_to_string(self.model.scheme_path())?;
        Ok(serde_json::from_str(&content)?)
    }

    fn source_dir(&self) -> PathBuf {
        self.dir.join("src").join(JAVA_PACKAGE_PATH)
    }

    fn native_dir(&self) -> PathBuf {
        self.dir.join("native")
    }

    fn copy_binding(&self) -> Result<(), Error> {
        fs::copy(&self.binding, self.native_dir().join(artifact_name()))?;
        Ok(())
    }

    fn clean_classes(&self) -> Result<(), Error> {
        let classes = self.dir.join("classes");
        if classes.is_dir() {
            fs::remove_dir_all(classes)?;
        }
        Ok(())
    }

    /// Removes files owned by the generator before writing fresh output.
    ///
    /// User-created files are left intact; only known generated files and the
    /// native artifact path are touched.
    fn clean_owned_files(&self) -> Result<(), Error> {
        for file in self.owned_files()? {
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

    fn owned_files(&self) -> Result<Vec<PathBuf>, Error> {
        let generated = JavaFiles::paths(self.model, self.scheme()?)?;
        let mut files = generated
            .iter()
            .cloned()
            .map(|file| PathBuf::from("src").join(JAVA_PACKAGE_PATH).join(file))
            .collect::<Vec<_>>();

        files.extend(
            generated
                .into_iter()
                .filter(|file| file.parent().is_some())
                .filter_map(|file| file.file_name().map(PathBuf::from))
                .map(|file| PathBuf::from("src").join(JAVA_PACKAGE_PATH).join(file)),
        );

        files.extend([
            PathBuf::from("src")
                .join(JAVA_PACKAGE_PATH)
                .join("ClientBindings.java"),
            PathBuf::from("src")
                .join(JAVA_PACKAGE_PATH)
                .join("Blocks.java"),
            PathBuf::from("src")
                .join(JAVA_PACKAGE_PATH)
                .join("Payloads.java"),
        ]);
        files.push(PathBuf::from("native").join(artifact_name()));
        Ok(files)
    }

    fn build(&self) -> Result<(), Error> {
        let mut command = Command::new("javac");
        command.arg("-d").arg(self.dir.join("classes"));
        for file in JavaFiles::paths(self.model, self.scheme()?)? {
            command.arg(self.source_dir().join(file));
        }
        let status = command.status()?;

        if !status.success() {
            return Err(Error::CommandFailed {
                command: "javac generated Java sources".to_owned(),
                status,
            });
        }
        Ok(())
    }
}

fn artifact_name() -> String {
    format!(
        "{}bindings{}",
        std::env::consts::DLL_PREFIX,
        std::env::consts::DLL_SUFFIX
    )
}
