mod bindings;
mod cli;
mod error;
mod model;

pub use bindings::*;
pub use error::*;
pub use model::*;

fn main() {
    if let Err(err) = cli::run() {
        eprintln!("{err}");
        std::process::exit(1);
    }
}
