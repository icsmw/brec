mod bindings;
mod cli;
mod error;
mod model;
mod ts;

pub use bindings::*;
pub use error::*;
pub use model::*;
pub use ts::*;

fn main() {
    if let Err(err) = cli::run() {
        eprintln!("{err}");
        std::process::exit(1);
    }
}
