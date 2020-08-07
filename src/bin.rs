mod cli {
    mod input;
    pub use input::*;

    mod output;
    pub use output::*;

    pub mod main_loop;
}

pub use yy_boss::*;

pub fn main() {
    let input = cli::parse_inputs();

    match YypBoss::new(&input.yyp_path) {
        Ok(yyp_boss) => {
            cli::main_loop::main_loop(yyp_boss);
        }
        Err(e) => {
            eprintln!("!!Error!!: Could not load Yyp...{:?}", e);
        }
    }

    println!("Program completed.");
}
