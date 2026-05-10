use crate::{BindingsCrate, Error, Model, NpmPackage, NpmTypeFiles, WasmTarget};
use std::env;
use std::fs;
use std::path::{Path, PathBuf};

/// Runs the generator from process arguments.
///
/// The command is intentionally small: it reads the exported Brec scheme,
/// prepares a temporary Rust crate with wasm-bindgen bindings, builds it with
/// wasm-pack, and writes the npm package that consumes that artifact.
pub fn run() -> Result<(), Error> {
    let cli = Cli::parse(env::args().skip(1))?;
    let model = Model::try_from(cli.scheme)?;
    let scheme_path = model.scheme_path();
    let scheme_dir = model.scheme_parent_path()?;
    let protocol_dir = match cli.protocol {
        Some(path) => path,
        None => infer_protocol_dir(scheme_path)?,
    };
    let bindings_dir = cli
        .bindings_out
        .unwrap_or_else(|| scheme_dir.join(BindingsCrate::DIR_NAME));
    let package_dir = cli.out.unwrap_or_else(|| scheme_dir.join("npm"));

    fs::create_dir_all(&package_dir)?;

    let type_files = NpmTypeFiles::new(&model);
    NpmPackage::validate_dependencies(&package_dir, &model, cli.npm_deps.as_deref(), cli.target)?;
    let bindings = BindingsCrate::new(&bindings_dir, &model, &protocol_dir, cli.cargo_deps)?;
    bindings.write()?;
    bindings.build_package(&package_dir, cli.target)?;

    NpmPackage::new(&package_dir, &type_files, cli.npm_deps, cli.target).write()?;

    println!(
        "generated WASM package from {} into {}",
        scheme_path.display(),
        package_dir.display()
    );
    Ok(())
}

/// Parsed command line options.
///
/// Paths are kept as the user provided them until the concrete generator
/// object needs to resolve them against the scheme, protocol, or config file.
#[derive(Debug)]
struct Cli {
    scheme: Option<PathBuf>,
    out: Option<PathBuf>,
    bindings_out: Option<PathBuf>,
    protocol: Option<PathBuf>,
    cargo_deps: Option<PathBuf>,
    npm_deps: Option<PathBuf>,
    target: WasmTarget,
}

impl Cli {
    fn parse(args: impl Iterator<Item = String>) -> Result<Self, Error> {
        let mut scheme = None;
        let mut out = None;
        let mut bindings_out = None;
        let mut protocol = None;
        let mut cargo_deps = None;
        let mut npm_deps = None;
        let mut target = None;

        let mut args = args;
        while let Some(arg) = args.next() {
            match arg.as_str() {
                "-h" | "--help" => {
                    print_usage();
                    std::process::exit(0);
                }
                "--scheme" => {
                    let value = next_value(&mut args, "--scheme")?;
                    scheme = Some(PathBuf::from(value));
                }
                "--out" => {
                    let value = next_value(&mut args, "--out")?;
                    out = Some(PathBuf::from(value));
                }
                "--npm-out" => {
                    let value = next_value(&mut args, "--npm-out")?;
                    out = Some(PathBuf::from(value));
                }
                "--bindings-out" => {
                    let value = next_value(&mut args, "--bindings-out")?;
                    bindings_out = Some(PathBuf::from(value));
                }
                "--protocol" => {
                    let value = next_value(&mut args, "--protocol")?;
                    protocol = Some(PathBuf::from(value));
                }
                "--cargo-deps" => {
                    let value = next_value(&mut args, "--cargo-deps")?;
                    cargo_deps = Some(PathBuf::from(value));
                }
                "--npm-deps" => {
                    let value = next_value(&mut args, "--npm-deps")?;
                    npm_deps = Some(PathBuf::from(value));
                }
                "--target" => {
                    let value = next_value(&mut args, "--target")?;
                    target = Some(WasmTarget::parse(&value)?);
                }
                other => {
                    return Err(Error::Cli(format!("unknown argument: {other}")));
                }
            }
        }

        Ok(Self {
            scheme,
            out,
            bindings_out,
            protocol,
            cargo_deps,
            npm_deps,
            target: target
                .ok_or_else(|| Error::Cli("missing required --target node|browser".into()))?,
        })
    }
}

fn print_usage() {
    println!(
        "Usage: brec_wasm_cli --target node|browser [--scheme <PATH>] [--out <DIR>] [--bindings-out <DIR>] [--protocol <DIR>] [--cargo-deps <PATH>] [--npm-deps <PATH>]

If --scheme is omitted, the CLI searches for brec.scheme.json in the current
directory. It first checks ./target/brec.scheme.json and then recursively scans
the working directory. If multiple files are found, the CLI fails and asks for
an explicit --scheme path.

--target controls the wasm-pack target and generated TypeScript entrypoint.
It is required and must be either node or browser.

By default, the generated bindings crate and npm package are written next to
the scheme file. --out and --npm-out are aliases for the npm package directory.
--cargo-deps and --npm-deps override built-in dependency versions by name."
    );
}

fn next_value(args: &mut impl Iterator<Item = String>, flag: &str) -> Result<String, Error> {
    args.next()
        .ok_or_else(|| Error::Cli(format!("missing value for {flag}")))
}

fn infer_protocol_dir(scheme_path: &Path) -> Result<PathBuf, Error> {
    let scheme_dir = scheme_path
        .parent()
        .ok_or_else(|| Error::MissingParent(scheme_path.to_path_buf()))?;
    if scheme_dir.file_name().is_some_and(|name| name == "target") {
        return scheme_dir
            .parent()
            .map(Path::to_path_buf)
            .ok_or_else(|| Error::MissingParent(scheme_dir.to_path_buf()));
    }
    Ok(scheme_dir.to_path_buf())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_all_paths() {
        let cli = Cli::parse(
            [
                "--scheme",
                "target/brec.scheme.json",
                "--npm-out",
                "generated/npm",
                "--bindings-out",
                "bindings",
                "--protocol",
                "protocol",
                "--cargo-deps",
                "deps.cargo.toml",
                "--npm-deps",
                "deps.npm.toml",
                "--target",
                "browser",
            ]
            .into_iter()
            .map(str::to_owned),
        )
        .expect("cli");

        assert_eq!(cli.scheme, Some(PathBuf::from("target/brec.scheme.json")));
        assert_eq!(cli.out, Some(PathBuf::from("generated/npm")));
        assert_eq!(cli.bindings_out, Some(PathBuf::from("bindings")));
        assert_eq!(cli.protocol, Some(PathBuf::from("protocol")));
        assert_eq!(cli.cargo_deps, Some(PathBuf::from("deps.cargo.toml")));
        assert_eq!(cli.npm_deps, Some(PathBuf::from("deps.npm.toml")));
        assert_eq!(cli.target, WasmTarget::Browser);
    }

    #[test]
    fn reports_missing_flag_value() {
        let err = Cli::parse(["--scheme"].into_iter().map(str::to_owned))
            .expect_err("missing value should fail");

        assert!(err.to_string().contains("missing value for --scheme"));
    }

    #[test]
    fn requires_target() {
        let err = Cli::parse(
            [
                "--scheme",
                "target/brec.scheme.json",
                "--npm-out",
                "generated/npm",
            ]
            .into_iter()
            .map(str::to_owned),
        )
        .expect_err("missing target should fail");

        assert!(err.to_string().contains("missing required --target"));
    }

    #[test]
    fn rejects_unknown_target() {
        let err = Cli::parse(["--target", "deno"].into_iter().map(str::to_owned))
            .expect_err("unknown target should fail");

        assert!(err.to_string().contains("expected node or browser"));
    }

    #[test]
    fn infers_protocol_dir_next_to_non_target_scheme() {
        let path = Path::new("/repo/protocol/brec.scheme.json");

        assert_eq!(
            infer_protocol_dir(path).expect("protocol dir"),
            PathBuf::from("/repo/protocol")
        );
    }

    #[test]
    fn infers_protocol_dir_from_target_scheme() {
        let path = Path::new("/repo/protocol/target/brec.scheme.json");

        assert_eq!(
            infer_protocol_dir(path).expect("protocol dir"),
            PathBuf::from("/repo/protocol")
        );
    }
}
