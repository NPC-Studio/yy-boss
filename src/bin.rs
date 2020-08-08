mod cli {
    mod input;
    pub use input::*;

    mod output;
    pub use output::*;

    mod startup;
    pub use startup::*;

    pub mod main_loop;
}

pub use yy_boss::*;

pub fn main() {
    let args = cli::parse_arguments();

    let boss_or = YypBoss::new(&args.yyp_path);
    if let Some(boss) = cli::startup(boss_or) {
        cli::main_loop::main_loop(boss);

        println!("Program completed.");
    }
}
