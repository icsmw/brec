use crate::Error;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

pub struct BindingsCrate {
    dir: PathBuf,
    package: BindingsPackage,
}

struct BindingsPackage {
    protocol_dir: PathBuf,
    protocol_package: String,
    brec_dir: PathBuf,
    workspace_dependencies: WorkspaceDependencies,
}

struct WorkspaceDependencies {
    entries: Vec<String>,
}

impl BindingsCrate {
    pub fn new(dir: impl Into<PathBuf>, protocol_dir: impl Into<PathBuf>) -> Result<Self, Error> {
        let protocol_dir = protocol_dir.into();
        let protocol_package = read_package_name(&protocol_dir.join("Cargo.toml"))?;

        Ok(Self {
            dir: dir.into(),
            package: BindingsPackage {
                workspace_dependencies: WorkspaceDependencies::from_protocol(&protocol_dir)?,
                protocol_dir,
                protocol_package,
                brec_dir: brec_core_dir()?,
            },
        })
    }

    pub fn write(&self) -> Result<(), Error> {
        fs::create_dir_all(self.dir.join("src"))?;
        fs::write(self.dir.join("Cargo.toml"), self.package.cargo_toml()?)?;
        fs::write(self.dir.join("src").join("lib.rs"), self.package.lib_rs())?;
        Ok(())
    }

    pub fn build_release(&self) -> Result<PathBuf, Error> {
        let manifest = self.dir.join("Cargo.toml");
        let status = Command::new("cargo")
            .arg("build")
            .arg("--release")
            .arg("--manifest-path")
            .arg(&manifest)
            .status()?;

        if !status.success() {
            return Err(Error::CommandFailed {
                command: format!(
                    "cargo build --release --manifest-path {}",
                    manifest.display()
                ),
                status,
            });
        }

        self.find_release_artifact()
    }

    fn find_release_artifact(&self) -> Result<PathBuf, Error> {
        let release = self.dir.join("target").join("release");
        let candidates = [
            release.join(format!(
                "{}bindings{}",
                std::env::consts::DLL_PREFIX,
                std::env::consts::DLL_SUFFIX
            )),
            release.join("bindings.dll"),
        ];

        candidates
            .into_iter()
            .find(|path| path.is_file())
            .ok_or(Error::BindingArtifactNotFound(release))
    }
}

impl BindingsPackage {
    fn cargo_toml(&self) -> Result<String, Error> {
        Ok(format!(
            r#"[workspace]
{}

[package]
name = "bindings"
version = "0.1.0"
edition = "2024"

[lib]
name = "bindings"
crate-type = ["cdylib", "rlib"]

[dependencies]
serde = {{ version = "1.0", features = ["derive"] }}
serde_json = "1.0"
protocol = {{ package = {}, path = {} }}
brec = {{ path = {}, features = ["bincode"] }}
napi = {{ version = "3.8", features = ["serde-json", "napi6"] }}
napi-derive = "3.5"
"#,
            self.workspace_dependencies,
            toml_string(&self.protocol_package)?,
            toml_path(&self.protocol_dir)?,
            toml_path(&self.brec_dir)?,
        ))
    }

    fn lib_rs(&self) -> &'static str {
        r#"use napi::bindgen_prelude::{Buffer, Error, Result, Status};
use napi::{Env, Unknown};
use napi_derive::napi;
use protocol::{Block, Packet, Payload};
use serde::{Deserialize, Serialize};

fn to_napi_error(prefix: &'static str, err: impl std::fmt::Display) -> Error {
    Error::new(Status::GenericFailure, format!("{prefix}: {err}"))
}

#[napi]
pub fn decode_block<'env>(env: &'env Env, buf: Buffer) -> Result<Unknown<'env>> {
    Block::decode_napi(env, buf).map_err(|e| to_napi_error("Decode block", e))
}

#[napi]
pub fn encode_block(_env: Env, val: Unknown<'_>) -> Result<Buffer> {
    let mut buf: Vec<u8> = Vec::new();
    Block::encode_napi(val, &mut buf).map_err(|e| to_napi_error("Encode block", e))?;
    Ok(buf.into())
}

#[napi]
pub fn decode_payload<'env>(env: &'env Env, buf: Buffer) -> Result<Unknown<'env>> {
    let mut ctx = ();
    Payload::decode_napi(env, buf, &mut ctx).map_err(|e| to_napi_error("Decode payload", e))
}

#[napi]
pub fn encode_payload(env: Env, val: Unknown<'_>) -> Result<Buffer> {
    let mut buf: Vec<u8> = Vec::new();
    let mut ctx = ();
    Payload::encode_napi(&env, val, &mut buf, &mut ctx)
        .map_err(|e| to_napi_error("Encode payload", e))?;
    Ok(buf.into())
}

#[derive(Deserialize, Serialize)]
struct JsPacket {
    blocks: Vec<Block>,
    payload: Option<Payload>,
}

#[napi]
pub fn decode_packet<'env>(env: &'env Env, buf: Buffer) -> Result<Unknown<'env>> {
    let mut ctx = ();
    Packet::decode_napi(env, buf, &mut ctx).map_err(|e| to_napi_error("Decode packet", e))
}

#[napi]
pub fn encode_packet(env: Env, blocks: Unknown<'_>, payload: Unknown<'_>) -> Result<Buffer> {
    let mut buf: Vec<u8> = Vec::new();
    let mut ctx = ();
    let blocks = env
        .from_js_value::<Vec<Block>, _>(blocks)
        .map_err(|e| to_napi_error("Deserialize blocks", e))?;
    let payload = env
        .from_js_value::<Option<Payload>, _>(payload)
        .map_err(|e| to_napi_error("Deserialize payload", e))?;
    let packet_js = env
        .to_js_value(&JsPacket { blocks, payload })
        .map_err(|e| to_napi_error("Serialize packet", e))?;
    Packet::encode_napi(&env, packet_js, &mut buf, &mut ctx)
        .map_err(|e| to_napi_error("Encode packet", e))?;
    Ok(buf.into())
}

#[napi]
pub fn encode_packet_object(env: Env, packet: Unknown<'_>) -> Result<Buffer> {
    let mut buf: Vec<u8> = Vec::new();
    let mut ctx = ();
    Packet::encode_napi(&env, packet, &mut buf, &mut ctx)
        .map_err(|e| to_napi_error("Encode packet object", e))?;
    Ok(buf.into())
}

#[napi]
pub fn encode_packet_from_json(env: Env, packet_json: String) -> Result<Buffer> {
    let mut buf: Vec<u8> = Vec::new();
    let mut ctx = ();
    let packet: JsPacket = serde_json::from_str(&packet_json)
        .map_err(|e| to_napi_error("Deserialize packet json", e))?;
    let packet_js = env
        .to_js_value(&packet)
        .map_err(|e| to_napi_error("Serialize packet", e))?;
    Packet::encode_napi(&env, packet_js, &mut buf, &mut ctx)
        .map_err(|e| to_napi_error("Encode packet", e))?;
    Ok(buf.into())
}
"#
    }
}

impl WorkspaceDependencies {
    fn from_protocol(protocol_dir: &Path) -> Result<Self, Error> {
        let manifest = protocol_dir.join("Cargo.toml");
        let content = fs::read_to_string(&manifest)?;
        let inherited = inherited_dependencies(&content);

        if inherited.is_empty() {
            return Ok(Self {
                entries: Vec::new(),
            });
        }

        let workspace_root = find_workspace_root(protocol_dir)
            .ok_or_else(|| Error::InvalidProtocolCrate(protocol_dir.to_path_buf()))?;
        let workspace_manifest = workspace_root.join("Cargo.toml");
        let workspace = fs::read_to_string(&workspace_manifest)?;
        let entries = workspace_dependency_entries(&workspace, &workspace_root, &inherited)?;

        Ok(Self { entries })
    }
}

impl std::fmt::Display for WorkspaceDependencies {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.entries.is_empty() {
            return Ok(());
        }

        writeln!(f, "[workspace.dependencies]")?;
        for entry in &self.entries {
            writeln!(f, "{entry}")?;
        }
        Ok(())
    }
}

fn read_package_name(manifest: &Path) -> Result<String, Error> {
    let content = fs::read_to_string(manifest)?;
    let mut in_package = false;

    for line in content.lines() {
        let line = line.trim();
        if line.starts_with('[') {
            in_package = line == "[package]";
            continue;
        }
        if !in_package || !line.starts_with("name") {
            continue;
        }
        let Some((_, value)) = line.split_once('=') else {
            continue;
        };
        let value = value.trim().trim_matches('"');
        if !value.is_empty() {
            return Ok(value.to_owned());
        }
    }

    let dir = manifest
        .parent()
        .map(Path::to_path_buf)
        .unwrap_or_else(|| manifest.to_path_buf());
    Err(Error::InvalidProtocolCrate(dir))
}

fn inherited_dependencies(manifest: &str) -> Vec<String> {
    let mut dependencies = Vec::new();
    let mut in_dependencies = false;

    for line in manifest.lines() {
        let line = line.trim();
        if line.starts_with('[') {
            in_dependencies = matches!(
                line,
                "[dependencies]" | "[build-dependencies]" | "[dev-dependencies]"
            );
            continue;
        }

        if !in_dependencies || !line.contains("workspace = true") {
            continue;
        }

        if let Some((name, _)) = line.split_once('=') {
            dependencies.push(name.trim().to_owned());
        }
    }

    dependencies.sort();
    dependencies.dedup();
    dependencies
}

fn find_workspace_root(start: &Path) -> Option<PathBuf> {
    start.ancestors().find_map(|dir| {
        let manifest = dir.join("Cargo.toml");
        let content = fs::read_to_string(manifest).ok()?;
        content.contains("[workspace]").then(|| dir.to_path_buf())
    })
}

fn workspace_dependency_entries(
    workspace: &str,
    workspace_root: &Path,
    inherited: &[String],
) -> Result<Vec<String>, Error> {
    let mut entries = Vec::new();
    let mut in_dependencies = false;

    for line in workspace.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with('[') {
            in_dependencies = trimmed == "[workspace.dependencies]";
            continue;
        }

        if !in_dependencies || trimmed.is_empty() || trimmed.starts_with('#') {
            continue;
        }

        let Some((name, _)) = trimmed.split_once('=') else {
            continue;
        };
        if inherited.iter().any(|inherited| inherited == name.trim()) {
            entries.push(rewrite_workspace_path(trimmed, workspace_root)?);
        }
    }

    let missing = inherited
        .iter()
        .filter(|name| {
            !entries.iter().any(|entry| {
                entry
                    .split_once('=')
                    .is_some_and(|(key, _)| key.trim() == *name)
            })
        })
        .cloned()
        .collect::<Vec<_>>();
    if !missing.is_empty() {
        return Err(Error::Cli(format!(
            "protocol inherits workspace dependencies not found in workspace root: {}",
            missing.join(", ")
        )));
    }

    Ok(entries)
}

fn rewrite_workspace_path(line: &str, workspace_root: &Path) -> Result<String, Error> {
    let Some(path_attr) = line.find("path = ") else {
        return Ok(line.to_owned());
    };
    let value_start = path_attr + "path = ".len();
    let Some(value) = line[value_start..].strip_prefix('"') else {
        return Ok(line.to_owned());
    };
    let Some(value_end) = value.find('"') else {
        return Ok(line.to_owned());
    };

    let path = &value[..value_end];
    if !path.starts_with('.') {
        return Ok(line.to_owned());
    }

    let absolute = match fs::canonicalize(workspace_root.join(path)) {
        Ok(path) => path,
        Err(_) => workspace_root.join(path),
    };
    let quoted = toml_path(&absolute)?;
    let quote_start = value_start;
    let quote_end = value_start + 1 + value_end + 1;

    Ok(format!(
        "{}{}{}",
        &line[..quote_start],
        quoted,
        &line[quote_end..]
    ))
}

fn brec_core_dir() -> Result<PathBuf, Error> {
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let root = manifest_dir
        .parent()
        .and_then(Path::parent)
        .and_then(Path::parent)
        .ok_or_else(|| Error::MissingParent(manifest_dir.clone()))?;
    Ok(root.join("lib").join("core"))
}

fn toml_path(path: &Path) -> Result<String, Error> {
    toml_string(&path.display().to_string())
}

fn toml_string(value: &str) -> Result<String, Error> {
    Ok(serde_json::to_string(value)?)
}
