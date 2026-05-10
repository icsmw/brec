use crate::{BindingsCrate, Error, Model, NpmPackage, NpmTypeFiles};
use std::env;
use std::fs;
use std::path::{Path, PathBuf};

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
    NpmPackage::validate_dependencies(&package_dir, &model, cli.npm_deps.as_deref())?;
    let bindings = BindingsCrate::new(&bindings_dir, &model, &protocol_dir, cli.cargo_deps)?;
    bindings.write()?;
    let binding_artifact = bindings.build_release()?;

    NpmPackage::new(&package_dir, &type_files, binding_artifact, cli.npm_deps).write()?;

    println!(
        "generated Node package from {} into {}",
        scheme_path.display(),
        package_dir.display()
    );
    Ok(())
}

#[derive(Debug)]
struct Cli {
    scheme: Option<PathBuf>,
    out: Option<PathBuf>,
    bindings_out: Option<PathBuf>,
    protocol: Option<PathBuf>,
    cargo_deps: Option<PathBuf>,
    npm_deps: Option<PathBuf>,
}

impl Cli {
    fn parse(args: impl Iterator<Item = String>) -> Result<Self, Error> {
        let mut scheme = None;
        let mut out = None;
        let mut bindings_out = None;
        let mut protocol = None;
        let mut cargo_deps = None;
        let mut npm_deps = None;

        let mut args = args.peekable();
        while let Some(arg) = args.next() {
            match arg.as_str() {
                "-h" | "--help" => {
                    print_usage();
                    std::process::exit(0);
                }
                "--scheme" => {
                    let value = args
                        .next()
                        .ok_or_else(|| Error::Cli("missing value for --scheme".to_owned()))?;
                    scheme = Some(PathBuf::from(value));
                }
                "--out" => {
                    let value = args
                        .next()
                        .ok_or_else(|| Error::Cli("missing value for --out".to_owned()))?;
                    out = Some(PathBuf::from(value));
                }
                "--npm-out" => {
                    let value = args
                        .next()
                        .ok_or_else(|| Error::Cli("missing value for --npm-out".to_owned()))?;
                    out = Some(PathBuf::from(value));
                }
                "--bindings-out" => {
                    let value = args
                        .next()
                        .ok_or_else(|| Error::Cli("missing value for --bindings-out".to_owned()))?;
                    bindings_out = Some(PathBuf::from(value));
                }
                "--protocol" => {
                    let value = args
                        .next()
                        .ok_or_else(|| Error::Cli("missing value for --protocol".to_owned()))?;
                    protocol = Some(PathBuf::from(value));
                }
                "--cargo-deps" => {
                    let value = args
                        .next()
                        .ok_or_else(|| Error::Cli("missing value for --cargo-deps".to_owned()))?;
                    cargo_deps = Some(PathBuf::from(value));
                }
                "--npm-deps" => {
                    let value = args
                        .next()
                        .ok_or_else(|| Error::Cli("missing value for --npm-deps".to_owned()))?;
                    npm_deps = Some(PathBuf::from(value));
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
        })
    }
}

fn print_usage() {
    println!(
        "Usage: brec-node-types [--scheme <PATH>] [--out <DIR>] [--bindings-out <DIR>] [--protocol <DIR>] [--cargo-deps <PATH>] [--npm-deps <PATH>]

If --scheme is omitted, the CLI searches for brec.scheme.json in the current
directory. It first checks ./target/brec.scheme.json and then recursively scans
the working directory. If multiple files are found, the CLI fails and asks for
an explicit --scheme path.

By default, the generated bindings crate and npm package are written next to
the scheme file. --out and --npm-out are aliases for the npm package directory.
--cargo-deps and --npm-deps override built-in dependency versions by name."
    );
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
