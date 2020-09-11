#![allow(clippy::bool_comparison)]
use cli::output::{Output, Startup};

mod cli {
    /// All input which the cli can receive as Json has their Rust forms defined here.
    pub mod input;

    /// All output which the cli can output as Json has their Rust forms defined here.
    pub mod output;

    /// All startup options which the cli can receive as Json has their Rust forms defined here.
    pub mod startup;

    /// Logging options for using the Cli.
    pub mod logging;

    #[doc(hidden)]
    pub(super) mod main_loop;

    #[doc(hidden)]
    pub(super) mod yy_cli;
}

pub use yy_boss::*;

fn main() {
    log_panics::init();

    let args = match cli::startup::parse_arguments() {
        Ok(v) => v,
        Err(e) => match e.kind {
            clap::ErrorKind::HelpDisplayed | clap::ErrorKind::VersionDisplayed => {
                e.write_to(&mut std::io::stdout())
                    .expect("couldn't write to stdout");
                std::process::exit(0);
            }
            _ => {
                Output::Startup(Startup {
                    success: false,
                    error: Some(StartupError::BadCliArguments(e.to_string()).to_string()),
                })
                .print();
                return;
            }
        },
    };

    cli::logging::begin_logging(args.logging, &args.working_directory);
    log::info!("Starting loop...");
    let yy_cli = cli::yy_cli::YyCli::new(args.working_directory);

    let boss_or = YypBoss::new(&args.yyp_path);
    if let Some(boss) = cli::startup::startup(boss_or, &yy_cli) {
        cli::main_loop::main_loop(boss, yy_cli);

        println!("Program completed.");
    }
}
