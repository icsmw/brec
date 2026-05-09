use super::TsConfigJson;
use crate::*;
use brec_scheme::SchemeFile;
use serde_json::json;
use std::fs;
use std::path::PathBuf;
use std::process::Command;

pub struct NpmPackage<'a> {
    dir: PathBuf,
    scheme: &'a SchemeFile,
    generated: &'a GeneratedFiles<'a>,
    binding: PathBuf,
}

pub struct PackageJson<'a> {
    scheme: &'a SchemeFile,
}

impl<'a> NpmPackage<'a> {
    pub fn new(
        dir: impl Into<PathBuf>,
        scheme: &'a SchemeFile,
        generated: &'a GeneratedFiles<'a>,
        binding: impl Into<PathBuf>,
    ) -> Self {
        Self {
            dir: dir.into(),
            scheme,
            generated,
            binding: binding.into(),
        }
    }

    pub fn write(&self) -> Result<(), Error> {
        fs::create_dir_all(&self.dir)?;
        fs::create_dir_all(self.dir.join("native"))?;

        self.generated.write_to(&self.dir)?;
        fs::write(self.dir.join("index.ts"), self.index_ts()?)?;
        fs::write(self.dir.join(PackageJson::FILE_NAME), self.package_json()?)?;
        fs::write(
            self.dir.join(TsConfigJson::FILE_NAME),
            TsConfigJson::new().render()?,
        )?;
        fs::copy(&self.binding, self.dir.join("native").join("bindings.node"))?;
        self.build()
    }

    fn index_ts(&self) -> Result<String, Error> {
        let blocks = BlocksFile::from(self.generated.model());
        let payloads = PayloadFile::from(self.generated.model());
        let packet = PacketFile::new(self.generated.model(), vec![&blocks, &payloads]);
        let api_block = ApiBlock;
        let api_payload = ApiPayload;
        let api_packet = ApiPacket;
        let api = ApiModule::new(
            vec![&api_block, &api_payload, &api_packet],
            vec![&blocks, &payloads, &packet],
        );
        let mut content = String::new();
        let mut tab = Tab::default();
        let mut writer = FormatterWriter::new(&mut content, &mut tab);
        api.write_ts(&mut writer)?;
        Ok(content)
    }

    fn package_json(&self) -> Result<String, Error> {
        PackageJson::new(self.scheme).render()
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
    pub const FILE_NAME: &'static str = "package.json";

    const DEV_DEPS: &'static [(&'static str, &'static str)] = &[("typescript", "^5.0.0")];

    pub fn new(scheme: &'a SchemeFile) -> Self {
        Self { scheme }
    }

    pub fn render(&self) -> Result<String, Error> {
        let dev_dependencies = Self::DEV_DEPS
            .iter()
            .map(|(name, version)| ((*name).to_owned(), json!(version)))
            .collect::<serde_json::Map<_, _>>();
        let package = json!({
            "name": self.scheme.package,
            "version": "0.1.0",
            "private": true,
            "main": "index.js",
            "types": "index.d.ts",
            "scripts": {
                "build": "tsc -p tsconfig.json"
            },
            "files": [
                "index.js",
                "index.d.ts",
                "blocks.js",
                "blocks.d.ts",
                "payloads.js",
                "payloads.d.ts",
                "packet.js",
                "packet.d.ts",
                "native/bindings.node"
            ],
            "devDependencies": dev_dependencies
        });

        Ok(format!("{}\n", serde_json::to_string_pretty(&package)?))
    }
}
