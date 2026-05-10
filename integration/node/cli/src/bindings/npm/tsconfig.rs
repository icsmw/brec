use crate::*;
use serde_json::json;

/// TypeScript compiler configuration for the generated npm package.
///
/// The package compiles only top-level generated `.ts` files and emits
/// declaration files, because consumers import the package as normal CommonJS.
pub struct TsConfigJson;

impl TsConfigJson {
    pub fn new() -> Self {
        Self
    }
}

impl Default for TsConfigJson {
    fn default() -> Self {
        Self::new()
    }
}

impl FileName for TsConfigJson {
    const FILE_NAME: &'static str = "tsconfig.json";
}

impl SourceWritable for TsConfigJson {
    fn write(&self, writer: &mut SourceWriter) -> Result<(), Error> {
        let config = json!({
            "compilerOptions": {
                "declaration": true,
                "esModuleInterop": true,
                "forceConsistentCasingInFileNames": true,
                "module": "CommonJS",
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
