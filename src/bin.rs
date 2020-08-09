mod cli {
    /// All input which the cli can receive as Json has their Rust forms defined here.
    pub mod input;

    /// All output which the cli can output as Json has their Rust forms defined here.
    pub mod output;

    /// All startup options which the cli can receive as Json has their Rust forms defined here.
    pub mod startup;

    #[doc(hidden)]
    pub(super) mod main_loop;
}

pub use yy_boss::*;

fn main() {
    let args = cli::startup::parse_arguments();

    let boss_or = YypBoss::new(&args.yyp_path);
    if let Some(boss) = cli::startup::startup(boss_or) {
        cli::main_loop::main_loop(boss);

        println!("Program completed.");
    }
}
