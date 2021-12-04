use std::process::exit;

use gumdrop::Options;
use indexmap::IndexMap;

use crate::{profile::Profile, spec::Spec};

#[derive(Debug, Options)]
struct Args {
    #[options(help = "show help information")]
    help: bool,

    #[options(command)]
    command: Option<Command>,
}

#[derive(Debug, Options)]
struct InfoArgs {
    #[options(help = "show help information")]
    help: bool,
    spec: Option<String>,
    profile: Option<String>,
}

#[derive(Debug, Options)]
enum Command {
    Info(InfoArgs),
}

impl Args {
    fn print_usage() {
        println!("cargo-pbuild -- Configuration profiles for Cargo\n<https://github.com/technocreatives/cargo-pbuild>\n\nUsage: cargo pbuild [OPTIONS] [SUBCOMMAND]\n");
        println!("{}\n", Args::usage());
        println!("Available commands:\n{}", Args::command_list().unwrap());
    }
}

impl InfoArgs {
    fn print_usage() {
        println!("cargo-pbuild info -- Show info about a profile or spec\n\nUsage: cargo pbuild info [OPTIONS]\n");
        println!("{}\n", InfoArgs::usage());
    }
}

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Error loading data.")]
    Load(#[from] LoadError),
}

#[derive(Debug, thiserror::Error)]
pub enum LoadError {
    #[error("IO error")]
    Io(#[from] std::io::Error),

    #[error("No main spec found")]
    MissingMainSpec,

    #[error("Spec error")]
    Spec(#[from] crate::spec::Error),

    #[error("Profile error")]
    Profile(#[from] crate::profile::Error),
}

fn load_data() -> Result<(IndexMap<String, Spec>, IndexMap<String, Profile>), LoadError> {
    let mut specs = IndexMap::new();
    for item in std::fs::read_dir("./profiles/specs")?.filter_map(Result::ok) {
        let p = item.path();
        if p.extension().and_then(|x| x.to_str()) == Some("toml") {
            let spec = Spec::parse_path(&p)?;
            specs.insert(
                p.file_stem()
                    .and_then(|x| x.to_str())
                    .map(|x| x.to_string())
                    .unwrap(),
                spec,
            );
        }
    }

    let main_spec = match specs.get("main") {
        Some(v) => v,
        None => return Err(LoadError::MissingMainSpec),
    };

    let mut profiles = IndexMap::new();

    for item in std::fs::read_dir("./profiles")?.filter_map(Result::ok) {
        let p = item.path();
        if p.extension().and_then(|x| x.to_str()) == Some("toml") {
            let profile = Profile::parse_path(main_spec, &p)?;
            profiles.insert(
                p.file_stem()
                    .and_then(|x| x.to_str())
                    .map(|x| x.to_string())
                    .unwrap(),
                profile,
            );
        }
    }

    Ok((specs, profiles))
}

pub fn run(args: Vec<String>) -> Result<(), Error> {
    let args = match Args::parse_args(&args, gumdrop::ParsingStyle::AllOptions) {
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

    let (specs, profiles) = load_data()?;

    let command = args.command.unwrap();
    match command {
        Command::Info(InfoArgs {
            help,
            spec,
            profile,
        }) => {
            if help {
                InfoArgs::print_usage();
                exit(0);
            }

            if let Some(spec_name) = spec {
                let spec = match specs.get(&spec_name) {
                    Some(v) => v,
                    None => {
                        eprintln!("No spec found with the name `{}`.", &spec_name);
                        exit(1);
                    }
                };

                println!("{}: {}", spec_name, spec);
            }

            if let Some(profile_name) = profile {
                let profile = match profiles.get(&profile_name) {
                    Some(v) => v,
                    None => {
                        eprintln!("No profile found with the name `{}`.", &profile_name);
                        exit(1);
                    }
                };

                println!("{}: {}", profile_name, profile);
            }
        }
    }

    Ok(())
}
