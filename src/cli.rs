use std::process::exit;

use gumdrop::Options;

#[derive(Debug, Options)]
struct Args {
    #[options(help = "show help information")]
    help: bool,
}

impl Args {
    fn print_usage() {
        println!("cargo-pbuild -- Configuration profiles for Cargo\n<https://github.com/technocreatives/cargo-pbuild>\n\nUsage: cargo pbuild [OPTIONS] [SUBCOMMAND]\n");
        println!("{}\n", Args::usage());
        println!("Available commands:\n{}", Args::command_list().unwrap());
    }
}

#[derive(Debug, thiserror::Error)]
pub enum Error {}

pub fn run(args: Vec<String>) -> Result<(), Error> {
    let _args = match Args::parse_args(&args, gumdrop::ParsingStyle::AllOptions) {
        Ok(args) if args.help => {
            Args::print_usage();
            exit(0);
        }
        Ok(args) => args,
        Err(e) => {
            eprintln!("error: {}\n", e);
            Args::print_usage();
            exit(2);
        }
    };

    Ok(())
}
