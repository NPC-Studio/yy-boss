pub mod cli {
    pub mod input;
    pub mod main_loop;
    pub mod output;
    pub mod startup;
}

pub use yy_boss::*;

pub fn main() {
    let args = cli::startup::parse_arguments();

    let boss_or = YypBoss::new(&args.yyp_path);
    if let Some(boss) = cli::startup::startup(boss_or) {
        cli::main_loop::main_loop(boss);

        println!("Program completed.");
    }
}
