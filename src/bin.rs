#![allow(clippy::bool_comparison)]

mod cli {
    /// All input which the cli can receive as Json has their Rust forms defined here.
    pub mod input;

    /// All output which the cli can output as Json has their Rust forms defined here.
    pub mod output;

    /// All startup options which the cli can receive as Json has their Rust forms defined here.
    pub mod startup;

    #[doc(hidden)]
    pub(super) mod main_loop;

    #[doc(hidden)]
    pub(super) mod yy_cli;
}

pub use yy_boss::*;

fn main() {
    let args = cli::startup::parse_arguments();
    let yy_cli = cli::yy_cli::YyCli::new(args.working_directory);

    let boss_or = YypBoss::new(&args.yyp_path);
    if let Some(boss) = cli::startup::startup(boss_or, &yy_cli) {
        cli::main_loop::main_loop(boss, yy_cli);

        println!("Program completed.");
    }
}
