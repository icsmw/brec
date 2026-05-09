use crate::*;
use serde_json::json;

pub struct TsConfigJson;

impl TsConfigJson {
    pub const FILE_NAME: &'static str = "tsconfig.json";

    pub fn new() -> Self {
        Self
    }

    pub fn render(&self) -> Result<String, Error> {
        let config = json!({
            "compilerOptions": {
                "declaration": true,
                "esModuleInterop": true,
                "forceConsistentCasingInFileNames": true,
                "module": "CommonJS",
                "moduleResolution": "Node",
                "noEmitOnError": true,
                "skipLibCheck": false,
                "strict": true,
                "target": "ES2022"
            },
            "include": [
                "*.ts"
            ]
        });

        Ok(format!("{}\n", serde_json::to_string_pretty(&config)?))
    }
}
