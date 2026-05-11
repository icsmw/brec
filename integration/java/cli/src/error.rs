use std::path::PathBuf;
use std::process::ExitStatus;
use thiserror::Error;

pub const SCHEME_FILE_NAME: &str = "brec.scheme.json";

/// Error type for the Java bindings generator.
///
/// Variants are intentionally user-facing: this binary is normally run from
/// shell scripts and e2e jobs, so each error should explain which input,
/// generated path, or external command failed without requiring a backtrace.
#[derive(Debug, Error)]
pub enum Error {
    #[error("{0}")]
    Cli(String),
    #[error("formatting error")]
    Fmt(#[from] std::fmt::Error),
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),
    #[error("failed to parse scheme json: {0}")]
    Json(#[from] serde_json::Error),
    #[error("failed to parse dependency config: {0}")]
    Toml(#[from] toml::de::Error),
    #[error("no {SCHEME_FILE_NAME} found under {}", .0.display())]
    SchemeNotFound(PathBuf),
    #[error(
        "found multiple {SCHEME_FILE_NAME} files; pass --scheme explicitly:\n{}",
        .0.iter().map(|path| format!("  {}", path.display())).collect::<Vec<_>>().join("\n")
    )]
    MultipleSchemes(Vec<PathBuf>),
    #[error("invalid scheme: {0}")]
    InvalidScheme(String),
    #[error("invalid protocol crate at {}; expected Cargo.toml with [package] name", .0.display())]
    InvalidProtocolCrate(PathBuf),
    #[error("path has no parent directory: {}", .0.display())]
    MissingParent(PathBuf),
    #[error("command `{command}` failed with status {status}")]
    CommandFailed { command: String, status: ExitStatus },
    #[error("failed to find built bindings artifact under {}", .0.display())]
    BindingArtifactNotFound(PathBuf),
    #[error(
        "payload schema references types that are not described in brec.scheme.json.\nmark them with #[payload(include)] so they are exported into scheme.types:\n{}",
        .0.iter().map(|name| format!("  {name}")).collect::<Vec<_>>().join("\n")
    )]
    MissingIncludedTypes(Vec<String>),
}
