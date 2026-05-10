use crate::*;
use std::ffi::OsString;
use std::path::{Component, Path, PathBuf};

/// Dependency path preserving the user's intent from a config file.
///
/// Absolute dependency paths are written back as absolute paths. Relative paths
/// are resolved against the dependency config file and then rendered relative
/// to the generated output directory, which keeps generated manifests portable.
pub(crate) enum RelativePath {
    Absolute(PathBuf),
    Relative(PathBuf),
}

impl RelativePath {
    /// Resolves a dependency path from a TOML config file.
    ///
    /// Relative config paths stay relative when rendered into generated output,
    /// but they are first normalized to an absolute path so `..` and `.` are
    /// handled consistently.
    pub(crate) fn from_config(path: impl AsRef<Path>, config_dir: &Path) -> Result<Self, Error> {
        let path = path.as_ref();
        let resolved = if path.is_absolute() {
            path.to_path_buf()
        } else {
            config_dir.join(path)
        };
        let normalized = absolute_normalized(&resolved)?;

        if !normalized.is_dir() {
            return Err(Error::Cli(format!(
                "relative dependency path does not point to an existing directory: {}",
                normalized.display()
            )));
        }

        if path.is_absolute() {
            Ok(Self::Absolute(normalized))
        } else {
            Ok(Self::Relative(normalized))
        }
    }

    pub(crate) fn path(&self, output_dir: &Path) -> Result<PathBuf, Error> {
        match self {
            Self::Absolute(path) => Ok(path.clone()),
            Self::Relative(path) => relative_path(output_dir, path),
        }
    }
}

/// Computes a relative path from one directory to another without requiring
/// either path to exist.
pub(crate) fn relative_path(from_dir: &Path, to: &Path) -> Result<PathBuf, Error> {
    let from = absolute_normalized(from_dir)?;
    let to = absolute_normalized(to)?;
    let from_components = comparable_components(&from);
    let to_components = comparable_components(&to);
    let shared = from_components
        .iter()
        .zip(&to_components)
        .take_while(|(left, right)| left == right)
        .count();

    if shared == 0 {
        return Ok(to);
    }

    let mut relative = PathBuf::new();
    for _ in &from_components[shared..] {
        relative.push("..");
    }
    for component in &to_components[shared..] {
        relative.push(component);
    }

    if relative.as_os_str().is_empty() {
        relative.push(".");
    }
    Ok(relative)
}

fn absolute_normalized(path: &Path) -> Result<PathBuf, Error> {
    let path = if path.is_absolute() {
        path.to_path_buf()
    } else {
        std::env::current_dir()?.join(path)
    };

    let mut normalized = PathBuf::new();
    for component in path.components() {
        match component {
            Component::Prefix(prefix) => normalized.push(prefix.as_os_str()),
            Component::RootDir => normalized.push(Path::new(std::path::MAIN_SEPARATOR_STR)),
            Component::CurDir => {}
            Component::ParentDir => {
                normalized.pop();
            }
            Component::Normal(component) => normalized.push(component),
        }
    }
    Ok(normalized)
}

fn comparable_components(path: &Path) -> Vec<OsString> {
    path.components()
        .filter_map(|component| match component {
            Component::Prefix(prefix) => Some(prefix.as_os_str().to_os_string()),
            Component::RootDir => Some(OsString::from(std::path::MAIN_SEPARATOR_STR)),
            Component::Normal(component) => Some(component.to_os_string()),
            Component::CurDir | Component::ParentDir => None,
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn builds_relative_path_between_sibling_dirs() {
        let from = Path::new("/repo/generated/npm");
        let to = Path::new("/repo/protocol");

        assert_eq!(
            relative_path(from, to).expect("relative"),
            PathBuf::from("../../protocol")
        );
    }

    #[test]
    fn returns_dot_for_same_dir() {
        let dir = Path::new("/repo/protocol");

        assert_eq!(
            relative_path(dir, dir).expect("relative"),
            PathBuf::from(".")
        );
    }

    #[test]
    fn keeps_absolute_path_when_roots_do_not_match() {
        let from = Path::new("C:/repo/generated");
        let to = Path::new("D:/deps/protocol");

        if cfg!(windows) {
            assert_eq!(
                relative_path(from, to).expect("relative"),
                PathBuf::from("D:/deps/protocol")
            );
        }
    }
}
