use crate::*;
use serde_json::json;

/// TypeScript compiler configuration for the generated npm package.
///
/// The package compiles only top-level generated `.ts` files and emits
/// declaration files.
pub struct TsConfigJson {
    target: WasmTarget,
}

impl TsConfigJson {
    pub fn new(target: WasmTarget) -> Self {
        Self { target }
    }
}

impl Default for TsConfigJson {
    fn default() -> Self {
        Self::new(WasmTarget::Node)
    }
}

impl FileName for TsConfigJson {
    const FILE_NAME: &'static str = "tsconfig.json";
}

impl SourceWritable for TsConfigJson {
    fn write(&self, writer: &mut SourceWriter) -> Result<(), Error> {
        let module = match self.target {
            WasmTarget::Node => "CommonJS",
            WasmTarget::Browser => "ES2022",
        };
        let config = json!({
            "compilerOptions": {
                "declaration": true,
                "esModuleInterop": true,
                "forceConsistentCasingInFileNames": true,
                "module": module,
                "noEmitOnError": true,
                "skipLibCheck": false,
                "strict": true,
                "target": "ES2022"
            },
            "include": [
                "*.ts"
            ]
        });

        writer.write(format!("{}\n", serde_json::to_string_pretty(&config)?))?;
        Ok(())
    }
}
