use std::env;
use std::process::exit;

use color_eyre::Result;

fn main() -> Result<()> {
    color_eyre::install()?;

    if env::var("CARGO").is_err() {
        eprintln!("This binary may only be called via `cargo pbuild`.");
        exit(1);
    }

    let args = std::env::args().skip(2).collect::<Vec<_>>();

    Ok(cargo_pbuild::cli::run(args)?)
}
