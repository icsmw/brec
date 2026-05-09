use crate::*;
use serde_json::json;

pub(super) const NATIVE_BINDING_PATH: &str = "native/bindings.node";

pub(super) struct PackageJson<'a> {
    model: &'a Model,
}

impl<'a> PackageJson<'a> {
    const DEV_DEPS: &'static [(&'static str, &'static str)] = &[("typescript", "^5.9.2")];

    pub fn new(model: &'a Model) -> Self {
        Self { model }
    }

    pub fn files(&self) -> Vec<String> {
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
