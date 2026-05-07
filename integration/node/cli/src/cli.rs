use crate::{BindingsCrate, Error, GeneratedFiles, Model, NpmPackage, error::SCHEME_FILE_NAME};
use brec_scheme::SchemeFile;
use std::env;
use std::fs;
use std::path::{Path, PathBuf};

pub fn run() -> Result<(), Error> {
    let cli = Cli::parse(env::args().skip(1))?;
    let scheme_path = match cli.scheme {
        Some(path) => path,
        None => find_scheme_path(&env::current_dir()?)?,
    };
    let scheme_dir = scheme_path
        .parent()
        .ok_or_else(|| Error::MissingParent(scheme_path.clone()))?;
    let protocol_dir = match cli.protocol {
        Some(path) => path,
        None => infer_protocol_dir(&scheme_path)?,
    };
    let bindings_dir = cli
        .bindings_out
        .unwrap_or_else(|| scheme_dir.join("bindings"));
    let package_dir = cli.out.unwrap_or_else(|| scheme_dir.join("npm"));

    let content = fs::read_to_string(&scheme_path)?;
    let scheme: SchemeFile = serde_json::from_str(&content)?;
    let model = Model::try_from(&scheme)?;

    fs::create_dir_all(&package_dir)?;

    let generated = GeneratedFiles::new(&model);
    let bindings = BindingsCrate::new(&bindings_dir, &protocol_dir)?;
    bindings.write()?;
    let binding_artifact = bindings.build_release()?;

    NpmPackage::new(&package_dir, &scheme, &generated, binding_artifact).write()?;

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
}

impl Cli {
    fn parse(args: impl Iterator<Item = String>) -> Result<Self, Error> {
        let mut scheme = None;
        let mut out = None;
        let mut bindings_out = None;
        let mut protocol = None;

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
        })
    }
}

fn print_usage() {
    println!(
        "Usage: brec-node-types [--scheme <PATH>] [--out <DIR>] [--bindings-out <DIR>] [--protocol <DIR>]

If --scheme is omitted, the CLI searches for brec.scheme.json in the current
directory. It first checks ./target/brec.scheme.json and then recursively scans
the working directory. If multiple files are found, the CLI fails and asks for
an explicit --scheme path.

By default, the generated bindings crate and npm package are written next to
the scheme file. --out and --npm-out are aliases for the npm package directory."
    );
}

fn find_scheme_path(start: &Path) -> Result<PathBuf, Error> {
    let direct = start.join("target").join(SCHEME_FILE_NAME);
    if direct.is_file() {
        return Ok(direct);
    }

    let mut dirs = vec![start.to_path_buf()];
    let mut matches = Vec::new();

    while let Some(dir) = dirs.pop() {
        for entry in fs::read_dir(&dir)? {
            let entry = entry?;
            let path = entry.path();
            let fty = entry.file_type()?;
            if fty.is_dir() {
                dirs.push(path);
                continue;
            }
            if fty.is_file()
                && path
                    .file_name()
                    .is_some_and(|name| name == SCHEME_FILE_NAME)
            {
                matches.push(path);
            }
        }
    }

    match matches.len() {
        0 => Err(Error::SchemeNotFound(start.to_path_buf())),
        1 => Ok(matches.remove(0)),
        _ => Err(Error::MultipleSchemes(matches)),
    }
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
