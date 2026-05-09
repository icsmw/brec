use super::cargo_toml::CargoToml;
use crate::*;
use std::path::{Path, PathBuf};

pub(super) struct WorkspaceDependencies {
    entries: Vec<String>,
}

impl WorkspaceDependencies {
    pub(super) fn from_protocol(protocol_dir: &Path) -> Result<Self, Error> {
        let manifest = protocol_dir.join(CargoToml::FILE_NAME);
        let content = std::fs::read_to_string(&manifest)?;
        let inherited = inherited_dependencies(&content);

        if inherited.is_empty() {
            return Ok(Self {
                entries: Vec::new(),
            });
        }

        let workspace_root = find_workspace_root(protocol_dir)
            .ok_or_else(|| Error::InvalidProtocolCrate(protocol_dir.to_path_buf()))?;
        let workspace_manifest = workspace_root.join(CargoToml::FILE_NAME);
        let workspace = std::fs::read_to_string(&workspace_manifest)?;
        let entries = workspace_dependency_entries(&workspace, &workspace_root, &inherited)?;

        Ok(Self { entries })
    }
}

impl SourceWritable for WorkspaceDependencies {
    fn write(&self, writer: &mut SourceWriter) -> Result<(), Error> {
        if self.entries.is_empty() {
            return Ok(());
        }

        writer.ln("")?;
        writer.ln("[workspace.dependencies]")?;
        for entry in &self.entries {
            writer.ln(entry)?;
        }
        Ok(())
    }
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
        let manifest = dir.join(CargoToml::FILE_NAME);
        let content = std::fs::read_to_string(manifest).ok()?;
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

    let absolute = match std::fs::canonicalize(workspace_root.join(path)) {
        Ok(path) => path,
        Err(_) => workspace_root.join(path),
    };
    let quoted = CargoToml::toml_path(&absolute)?;
    let quote_start = value_start;
    let quote_end = value_start + 1 + value_end + 1;

    Ok(format!(
        "{}{}{}",
        &line[..quote_start],
        quoted,
        &line[quote_end..]
    ))
}
