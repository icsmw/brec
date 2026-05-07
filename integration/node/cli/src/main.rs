mod bindings;
mod cli;
mod desc;
mod error;
mod model;
mod npm;

pub use bindings::*;
pub use desc::*;
pub use error::*;
pub use model::*;
pub use npm::NpmPackage;

fn main() {
    if let Err(err) = cli::run() {
        eprintln!("{err}");
        std::process::exit(1);
    }
}
